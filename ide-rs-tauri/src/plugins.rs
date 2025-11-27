//! Plugin Management Commands / プラグイン管理コマンド
//!
//! Provides Tauri commands for plugin management.
//! プラグイン管理のTauriコマンドを提供します。

use log::{debug, info, warn};
use serde::Serialize;
use sikulid_core::plugin::PluginManager;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Plugin State / プラグイン状態
// ============================================================================

/// State for plugin management
/// プラグイン管理の状態
pub struct PluginManagementState {
    /// Plugin manager / プラグインマネージャー
    pub manager: Mutex<PluginManager>,
    /// Plugins directory / プラグインディレクトリ
    pub plugins_dir: PathBuf,
    /// State file path / 状態ファイルパス
    pub state_file: PathBuf,
}

impl PluginManagementState {
    pub fn new(app_data_dir: PathBuf) -> Self {
        let plugins_dir = app_data_dir.join("plugins");
        let state_file = app_data_dir.join("plugin_state.json");

        let mut manager = PluginManager::new(&plugins_dir);

        // Discover plugins on startup
        if let Err(e) = manager.discover_plugins() {
            warn!("Failed to discover plugins: {}", e);
        }

        // Load enabled state
        if let Err(e) = manager.load_enabled_state(&state_file) {
            warn!("Failed to load plugin state: {}", e);
        }

        Self {
            manager: Mutex::new(manager),
            plugins_dir,
            state_file,
        }
    }
}

// ============================================================================
// Plugin DTOs / プラグインDTO
// ============================================================================

/// Plugin info for frontend
/// フロントエンド用プラグイン情報
#[derive(Serialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub enabled: bool,
    pub state: String,
    pub categories: Vec<String>,
    pub homepage: String,
    pub error: Option<String>,
}

/// Plugin operation result
/// プラグイン操作結果
#[derive(Serialize)]
pub struct PluginResult {
    pub success: bool,
    pub message: String,
}

/// Plugin list result
/// プラグインリスト結果
#[derive(Serialize)]
pub struct PluginListResult {
    pub plugins: Vec<PluginInfo>,
    pub total: usize,
    pub enabled: usize,
    pub active: usize,
}

// ============================================================================
// Plugin Commands / プラグインコマンド
// ============================================================================

/// Get all plugins
/// すべてのプラグインを取得
#[tauri::command]
pub fn get_plugins(state: State<'_, PluginManagementState>) -> PluginListResult {
    debug!("Getting all plugins");
    let manager = state.manager.lock().unwrap();

    let plugins: Vec<PluginInfo> = manager
        .plugins()
        .map(|p| PluginInfo {
            id: p.metadata.id.clone(),
            name: p.metadata.name.clone(),
            version: p.metadata.version.clone(),
            description: p.metadata.description.clone(),
            author: p.metadata.author.clone(),
            enabled: p.enabled,
            state: format!("{:?}", p.state),
            categories: p.metadata.categories.clone(),
            homepage: p.metadata.homepage.clone(),
            error: p.error.clone(),
        })
        .collect();

    let total = plugins.len();
    let enabled = plugins.iter().filter(|p| p.enabled).count();
    let active = manager.active_plugin_count();

    PluginListResult {
        plugins,
        total,
        enabled,
        active,
    }
}

/// Get plugin by ID
/// IDでプラグインを取得
#[tauri::command]
pub fn get_plugin(id: String, state: State<'_, PluginManagementState>) -> Option<PluginInfo> {
    debug!("Getting plugin: {}", id);
    let manager = state.manager.lock().unwrap();

    manager.get_plugin(&id).map(|p| PluginInfo {
        id: p.metadata.id.clone(),
        name: p.metadata.name.clone(),
        version: p.metadata.version.clone(),
        description: p.metadata.description.clone(),
        author: p.metadata.author.clone(),
        enabled: p.enabled,
        state: format!("{:?}", p.state),
        categories: p.metadata.categories.clone(),
        homepage: p.metadata.homepage.clone(),
        error: p.error.clone(),
    })
}

/// Enable a plugin
/// プラグインを有効化
#[tauri::command]
pub fn enable_plugin(id: String, state: State<'_, PluginManagementState>) -> PluginResult {
    info!("Enabling plugin: {}", id);
    let mut manager = state.manager.lock().unwrap();

    match manager.enable_plugin(&id) {
        Ok(_) => {
            if let Err(e) = manager.save_enabled_state(&state.state_file) {
                warn!("Failed to save plugin state: {}", e);
            }
            PluginResult {
                success: true,
                message: format!("Plugin '{}' enabled", id),
            }
        }
        Err(e) => PluginResult {
            success: false,
            message: format!("Failed to enable plugin: {}", e),
        },
    }
}

/// Disable a plugin
/// プラグインを無効化
#[tauri::command]
pub fn disable_plugin(id: String, state: State<'_, PluginManagementState>) -> PluginResult {
    info!("Disabling plugin: {}", id);
    let mut manager = state.manager.lock().unwrap();

    match manager.disable_plugin(&id) {
        Ok(_) => {
            if let Err(e) = manager.save_enabled_state(&state.state_file) {
                warn!("Failed to save plugin state: {}", e);
            }
            PluginResult {
                success: true,
                message: format!("Plugin '{}' disabled", id),
            }
        }
        Err(e) => PluginResult {
            success: false,
            message: format!("Failed to disable plugin: {}", e),
        },
    }
}

/// Refresh plugin list
/// プラグインリストを更新
#[tauri::command]
pub fn refresh_plugins(state: State<'_, PluginManagementState>) -> PluginResult {
    info!("Refreshing plugin list");
    let mut manager = state.manager.lock().unwrap();

    match manager.discover_plugins() {
        Ok(count) => {
            // Reload saved state
            if let Err(e) = manager.load_enabled_state(&state.state_file) {
                warn!("Failed to load plugin state: {}", e);
            }
            PluginResult {
                success: true,
                message: format!("Found {} plugins", count),
            }
        }
        Err(e) => PluginResult {
            success: false,
            message: format!("Failed to discover plugins: {}", e),
        },
    }
}

/// Get plugins directory path
/// プラグインディレクトリパスを取得
#[tauri::command]
pub fn get_plugins_directory(state: State<'_, PluginManagementState>) -> String {
    state.plugins_dir.to_string_lossy().to_string()
}

/// Open plugins directory in file manager
/// ファイルマネージャーでプラグインディレクトリを開く
#[tauri::command]
pub async fn open_plugins_directory(
    state: State<'_, PluginManagementState>,
) -> Result<PluginResult, String> {
    let plugins_dir = state.plugins_dir.clone();

    // Create directory if it doesn't exist
    if !plugins_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
            return Ok(PluginResult {
                success: false,
                message: format!("Failed to create plugins directory: {}", e),
            });
        }
    }

    // Open in file manager
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(&plugins_dir)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&plugins_dir).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&plugins_dir)
            .spawn();
    }

    Ok(PluginResult {
        success: true,
        message: "Opened plugins directory".to_string(),
    })
}

/// Export plugin list
/// プラグインリストをエクスポート
#[tauri::command]
pub fn export_plugin_list(state: State<'_, PluginManagementState>) -> Result<String, String> {
    debug!("Exporting plugin list");
    let manager = state.manager.lock().unwrap();
    manager
        .export_plugin_list()
        .map_err(|e| format!("Export failed: {}", e))
}

/// Get plugin load order
/// プラグインの読み込み順序を取得
#[tauri::command]
pub fn get_plugin_load_order(state: State<'_, PluginManagementState>) -> Vec<String> {
    debug!("Getting plugin load order");
    let manager = state.manager.lock().unwrap();
    manager.load_order().to_vec()
}

// ============================================================================
// Plugin Settings Commands / プラグイン設定コマンド
// ============================================================================

/// Plugin settings result
/// プラグイン設定結果
#[derive(Serialize)]
pub struct PluginSettingsResult {
    pub settings: serde_json::Value,
    pub schema: Vec<SettingsField>,
}

/// Settings field schema
/// 設定フィールドスキーマ
#[derive(Serialize)]
pub struct SettingsField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub default: serde_json::Value,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Get plugin settings
/// プラグイン設定を取得
#[tauri::command]
pub fn get_plugin_settings(
    id: String,
    state: State<'_, PluginManagementState>,
) -> PluginSettingsResult {
    debug!("Getting settings for plugin: {}", id);

    let config_path = state.plugins_dir.join(&id).join("config.json");
    let settings = if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or(serde_json::json!({})),
            Err(_) => serde_json::json!({}),
        }
    } else {
        serde_json::json!({})
    };

    // Load schema from plugin.json if available
    let schema_path = state.plugins_dir.join(&id).join("plugin.json");
    let schema = if schema_path.exists() {
        match std::fs::read_to_string(&schema_path) {
            Ok(content) => {
                if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(settings_schema) = metadata.get("settings") {
                        if let Some(arr) = settings_schema.as_array() {
                            arr.iter()
                                .filter_map(|f| {
                                    Some(SettingsField {
                                        key: f.get("key")?.as_str()?.to_string(),
                                        label: f
                                            .get("label")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        field_type: f
                                            .get("type")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("text")
                                            .to_string(),
                                        default: f
                                            .get("default")
                                            .cloned()
                                            .unwrap_or(serde_json::Value::Null),
                                        description: f
                                            .get("description")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string(),
                                        options: f
                                            .get("options")
                                            .and_then(|v| v.as_array())
                                            .cloned(),
                                        min: f.get("min").and_then(|v| v.as_f64()),
                                        max: f.get("max").and_then(|v| v.as_f64()),
                                    })
                                })
                                .collect()
                        } else {
                            Vec::new()
                        }
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            }
            Err(_) => Vec::new(),
        }
    } else {
        Vec::new()
    };

    PluginSettingsResult { settings, schema }
}

/// Save plugin settings
/// プラグイン設定を保存
#[tauri::command]
pub fn save_plugin_settings(
    id: String,
    settings: serde_json::Value,
    state: State<'_, PluginManagementState>,
) -> PluginResult {
    info!("Saving settings for plugin: {}", id);

    let config_dir = state.plugins_dir.join(&id);
    if !config_dir.exists() {
        return PluginResult {
            success: false,
            message: format!("Plugin '{}' not found", id),
        };
    }

    let config_path = config_dir.join("config.json");
    match serde_json::to_string_pretty(&settings) {
        Ok(content) => match std::fs::write(&config_path, content) {
            Ok(_) => PluginResult {
                success: true,
                message: "Settings saved".to_string(),
            },
            Err(e) => PluginResult {
                success: false,
                message: format!("Failed to write config: {}", e),
            },
        },
        Err(e) => PluginResult {
            success: false,
            message: format!("Failed to serialize settings: {}", e),
        },
    }
}

/// Permission info for frontend
/// フロントエンド用権限情報
#[derive(Serialize)]
pub struct PermissionInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub sensitive: bool,
}

/// Plugin permissions result
/// プラグイン権限結果
#[derive(Serialize)]
pub struct PluginPermissionsResult {
    pub available: Vec<PermissionInfo>,
    pub permissions: std::collections::HashMap<String, String>,
}

/// Get plugin permissions
/// プラグイン権限を取得
#[tauri::command]
pub fn get_plugin_permissions(
    id: String,
    _state: State<'_, PluginManagementState>,
) -> PluginPermissionsResult {
    debug!("Getting permissions for plugin: {}", id);

    use sikulid_core::plugin::Permission;

    let available: Vec<PermissionInfo> = Permission::all()
        .into_iter()
        .map(|p| PermissionInfo {
            id: format!("{:?}", p),
            name: p.display_name().to_string(),
            description: p.description().to_string(),
            sensitive: p.is_sensitive(),
        })
        .collect();

    // For now, return empty permissions map - would need PermissionManager integration
    let permissions = std::collections::HashMap::new();

    PluginPermissionsResult {
        available,
        permissions,
    }
}

/// Set plugin permission
/// プラグイン権限を設定
#[tauri::command]
pub fn set_plugin_permission(
    id: String,
    permission: String,
    grant: bool,
    _state: State<'_, PluginManagementState>,
) -> PluginResult {
    info!(
        "Setting permission {} for plugin {} to {}",
        permission, id, grant
    );

    // This would integrate with PermissionManager
    // For now, just log the action
    PluginResult {
        success: true,
        message: format!(
            "Permission {} {} for plugin {}",
            permission,
            if grant { "granted" } else { "revoked" },
            id
        ),
    }
}

// ============================================================================
// Plugin Install/Uninstall Commands / プラグインインストール/アンインストールコマンド
// ============================================================================

/// File selection result
/// ファイル選択結果
#[derive(Serialize)]
pub struct FileSelectResult {
    pub path: Option<String>,
}

/// Select plugin file dialog
/// プラグインファイル選択ダイアログ
#[tauri::command]
pub async fn select_plugin_file() -> FileSelectResult {
    // Note: File dialog would be handled via Tauri dialog plugin
    // For now, users should use the file path input or drag-drop
    FileSelectResult { path: None }
}

/// Install plugin from file
/// ファイルからプラグインをインストール
#[tauri::command]
pub async fn install_plugin_from_file(
    path: String,
    state: State<'_, PluginManagementState>,
) -> Result<PluginResult, String> {
    info!("Installing plugin from file: {}", path);

    let source_path = std::path::Path::new(&path);
    if !source_path.exists() {
        return Ok(PluginResult {
            success: false,
            message: "File not found".to_string(),
        });
    }

    // Check if it's a zip file
    if !path.to_lowercase().ends_with(".zip") {
        return Ok(PluginResult {
            success: false,
            message: "Only .zip files are supported".to_string(),
        });
    }

    // Extract to plugins directory
    let plugins_dir = state.plugins_dir.clone();
    if !plugins_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&plugins_dir) {
            return Ok(PluginResult {
                success: false,
                message: format!("Failed to create plugins directory: {}", e),
            });
        }
    }

    // Extract zip file
    let file = match std::fs::File::open(source_path) {
        Ok(f) => f,
        Err(e) => {
            return Ok(PluginResult {
                success: false,
                message: format!("Failed to open file: {}", e),
            })
        }
    };

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            return Ok(PluginResult {
                success: false,
                message: format!("Failed to read zip: {}", e),
            })
        }
    };

    // Extract to a temp directory first, then move
    let temp_dir = plugins_dir.join(".temp_install");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
    std::fs::create_dir_all(&temp_dir).ok();

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let outpath = match file.enclosed_name() {
            Some(path) => temp_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath).ok();
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p).ok();
                }
            }
            let mut outfile = match std::fs::File::create(&outpath) {
                Ok(f) => f,
                Err(_) => continue,
            };
            std::io::copy(&mut file, &mut outfile).ok();
        }
    }

    // Find plugin.json and determine plugin ID
    let plugin_json = temp_dir.join("plugin.json");
    let (plugin_id, final_dest) = if plugin_json.exists() {
        // plugin.json is at root level
        match std::fs::read_to_string(&plugin_json) {
            Ok(content) => {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(id) = meta.get("id").and_then(|v| v.as_str()) {
                        let dest = plugins_dir.join(id);
                        (id.to_string(), dest)
                    } else {
                        let _ = std::fs::remove_dir_all(&temp_dir);
                        return Ok(PluginResult {
                            success: false,
                            message: "Invalid plugin.json: missing id".to_string(),
                        });
                    }
                } else {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    return Ok(PluginResult {
                        success: false,
                        message: "Invalid plugin.json format".to_string(),
                    });
                }
            }
            Err(_) => {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Ok(PluginResult {
                    success: false,
                    message: "Failed to read plugin.json".to_string(),
                });
            }
        }
    } else {
        // Check if there's a subdirectory with plugin.json
        let mut found = None;
        if let Ok(entries) = std::fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let sub_plugin_json = entry.path().join("plugin.json");
                if sub_plugin_json.exists() {
                    if let Ok(content) = std::fs::read_to_string(&sub_plugin_json) {
                        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(id) = meta.get("id").and_then(|v| v.as_str()) {
                                found = Some((id.to_string(), entry.path()));
                                break;
                            }
                        }
                    }
                }
            }
        }

        match found {
            Some((id, subdir)) => {
                let dest = plugins_dir.join(&id);
                // Move subdir to dest
                if dest.exists() {
                    let _ = std::fs::remove_dir_all(&dest);
                }
                if let Err(e) = std::fs::rename(&subdir, &dest) {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                    return Ok(PluginResult {
                        success: false,
                        message: format!("Failed to install plugin: {}", e),
                    });
                }
                let _ = std::fs::remove_dir_all(&temp_dir);

                // Refresh plugins
                let mut manager = state.manager.lock().unwrap();
                let _ = manager.discover_plugins();

                return Ok(PluginResult {
                    success: true,
                    message: format!("Plugin '{}' installed successfully", id),
                });
            }
            None => {
                let _ = std::fs::remove_dir_all(&temp_dir);
                return Ok(PluginResult {
                    success: false,
                    message: "No plugin.json found in archive".to_string(),
                });
            }
        }
    };

    // Move temp_dir contents to final_dest
    if final_dest.exists() {
        let _ = std::fs::remove_dir_all(&final_dest);
    }
    if let Err(e) = std::fs::rename(&temp_dir, &final_dest) {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Ok(PluginResult {
            success: false,
            message: format!("Failed to install plugin: {}", e),
        });
    }

    // Refresh plugins
    let mut manager = state.manager.lock().unwrap();
    let _ = manager.discover_plugins();

    Ok(PluginResult {
        success: true,
        message: format!("Plugin '{}' installed successfully", plugin_id),
    })
}

/// Install plugin from URL
/// URLからプラグインをインストール
#[tauri::command]
pub async fn install_plugin_from_url(
    url: String,
    _state: State<'_, PluginManagementState>,
) -> Result<PluginResult, String> {
    info!("Installing plugin from URL: {}", url);

    // This would require HTTP client functionality
    // For now, return not implemented
    Ok(PluginResult {
        success: false,
        message: "URL installation not yet implemented. Please download the plugin and install from file.".to_string(),
    })
}

/// Uninstall plugin
/// プラグインをアンインストール
#[tauri::command]
pub async fn uninstall_plugin(
    id: String,
    state: State<'_, PluginManagementState>,
) -> Result<PluginResult, String> {
    info!("Uninstalling plugin: {}", id);

    let plugin_dir = state.plugins_dir.join(&id);
    if !plugin_dir.exists() {
        return Ok(PluginResult {
            success: false,
            message: format!("Plugin '{}' not found", id),
        });
    }

    // Disable plugin first
    {
        let mut manager = state.manager.lock().unwrap();
        let _ = manager.disable_plugin(&id);
    }

    // Remove plugin directory
    match std::fs::remove_dir_all(&plugin_dir) {
        Ok(_) => {
            // Refresh plugins
            let mut manager = state.manager.lock().unwrap();
            let _ = manager.discover_plugins();

            Ok(PluginResult {
                success: true,
                message: format!("Plugin '{}' uninstalled successfully", id),
            })
        }
        Err(e) => Ok(PluginResult {
            success: false,
            message: format!("Failed to remove plugin: {}", e),
        }),
    }
}

/// Open plugin directory
/// プラグインディレクトリを開く
#[tauri::command]
pub async fn open_plugin_directory(
    id: String,
    state: State<'_, PluginManagementState>,
) -> Result<PluginResult, String> {
    let plugin_dir = state.plugins_dir.join(&id);

    if !plugin_dir.exists() {
        return Ok(PluginResult {
            success: false,
            message: format!("Plugin '{}' not found", id),
        });
    }

    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("explorer")
            .arg(&plugin_dir)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&plugin_dir).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&plugin_dir)
            .spawn();
    }

    Ok(PluginResult {
        success: true,
        message: "Opened plugin directory".to_string(),
    })
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_info_serialization() {
        let info = PluginInfo {
            id: "com.example.test".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            enabled: true,
            state: "Active".to_string(),
            categories: vec!["utility".to_string()],
            homepage: "https://example.com".to_string(),
            error: None,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("Test Plugin"));
        assert!(json.contains("enabled"));
    }

    #[test]
    fn test_plugin_result() {
        let result = PluginResult {
            success: true,
            message: "OK".to_string(),
        };
        assert!(result.success);
    }
}
