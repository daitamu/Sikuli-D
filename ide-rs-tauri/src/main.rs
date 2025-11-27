//! SikuliX IDE - Tauri Prototype
//! SikuliX IDE - Tauri プロトタイプ

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod capture;
mod debug;
mod execution;
mod image_library;
mod ocr;
mod pattern;
mod plugins;
mod settings;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sikulid::{PythonVersion, SyntaxAnalyzer};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

// ============================================================================
// Application State / アプリケーション状態
// ============================================================================

/// Application state for managing editor state
/// エディタ状態を管理するアプリケーション状態
#[derive(Default)]
struct AppState {
    /// Currently open file path / 現在開いているファイルパス
    current_file: Mutex<Option<PathBuf>>,
    /// Recent files list / 最近使用したファイルリスト
    recent_files: Mutex<Vec<PathBuf>>,
    /// Whether content has been modified / 内容が変更されたかどうか
    is_modified: Mutex<bool>,
}

/// Persistent settings saved to disk
/// ディスクに保存される永続設定
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct PersistentSettings {
    /// Recent files list / 最近使用したファイルリスト
    recent_files: Vec<String>,
    /// Last opened directory / 最後に開いたディレクトリ
    last_directory: Option<String>,
    /// Window width / ウィンドウ幅
    window_width: Option<u32>,
    /// Window height / ウィンドウ高さ
    window_height: Option<u32>,
}

// ============================================================================
// Data Transfer Objects / データ転送オブジェクト
// ============================================================================

/// File information returned to frontend
/// フロントエンドに返すファイル情報
#[derive(Serialize, Deserialize, Clone)]
struct FileInfo {
    path: String,
    name: String,
    content: String,
}

/// Result type for file operations
/// ファイル操作の結果型
#[derive(Serialize)]
struct FileOperationResult {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    file_info: Option<FileInfo>,
}

// ============================================================================
// Settings Persistence / 設定永続化
// ============================================================================

fn get_settings_path(app: &AppHandle) -> PathBuf {
    let app_dir = app
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    app_dir.join("settings.json")
}

fn load_settings(app: &AppHandle) -> PersistentSettings {
    let path = get_settings_path(app);
    debug!("Loading settings from: {:?}", path);

    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => {
                    info!("Settings loaded successfully");
                    return settings;
                }
                Err(e) => {
                    warn!("Failed to parse settings file: {}", e);
                }
            },
            Err(e) => {
                warn!("Failed to read settings file: {}", e);
            }
        }
    } else {
        debug!("Settings file does not exist, using defaults");
    }

    PersistentSettings::default()
}

fn save_settings(app: &AppHandle, settings: &PersistentSettings) {
    let path = get_settings_path(app);
    debug!("Saving settings to: {:?}", path);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create settings directory: {}", e);
                return;
            }
        }
    }

    match serde_json::to_string_pretty(settings) {
        Ok(content) => {
            if let Err(e) = fs::write(&path, content) {
                error!("Failed to write settings file: {}", e);
            } else {
                info!("Settings saved successfully");
            }
        }
        Err(e) => {
            error!("Failed to serialize settings: {}", e);
        }
    }
}

// ============================================================================
// Main Entry Point / メインエントリポイント
// ============================================================================

fn main() {
    // Initialize logger
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Starting SikuliX IDE v{}", env!("CARGO_PKG_VERSION"));
    info!("Core library version: {}", sikulid::VERSION);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(AppState::default())
        .manage(capture::CaptureState::default())
        .manage(debug::DebugPanelState::new())
        .manage(execution::ScriptProcessState::new())
        .manage(pattern::PatternState::default())
        .manage(settings::SettingsState::new())
        .manage(ocr::OcrState::new())
        .setup(|app| {
            info!("Application setup starting");

            // Initialize plugin management
            let app_data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            app.manage(plugins::PluginManagementState::new(app_data_dir));

            // Load persistent settings and restore recent files
            let settings = load_settings(app.handle());
            let state = app.state::<AppState>();

            if let Ok(mut recent) = state.recent_files.lock() {
                *recent = settings
                    .recent_files
                    .iter()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
                    .collect();
                info!("Restored {} recent files", recent.len());
            }

            info!("Application setup completed");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Python analysis
            analyze_python_version,
            analyze_script_content,
            get_core_version,
            // Script execution
            execution::run_script,
            execution::run_script_streaming,
            execution::stop_script,
            // File operations
            read_file,
            write_file,
            get_current_file,
            set_current_file,
            clear_current_file,
            // State management
            set_modified,
            is_modified,
            // Recent files
            get_recent_files,
            add_recent_file,
            clear_recent_files,
            // Settings persistence
            save_app_settings,
            load_app_settings,
            // Screen capture
            capture::start_capture,
            capture::capture_region,
            capture::capture_full_screen,
            capture::cancel_capture,
            capture::get_last_capture_path,
            capture::is_capture_active,
            // Pattern editor
            pattern::create_pattern,
            pattern::update_pattern,
            pattern::test_pattern,
            pattern::preview_target_offset,
            pattern::generate_pattern_code,
            pattern::get_current_pattern,
            pattern::get_last_test_result,
            pattern::clear_pattern,
            // Settings
            settings::get_settings,
            settings::save_settings,
            settings::reset_settings,
            settings::get_theme,
            settings::set_theme,
            settings::get_hotkeys,
            settings::update_hotkey,
            settings::toggle_hotkey,
            settings::reset_hotkeys,
            settings::get_profiles,
            settings::switch_profile,
            settings::create_profile,
            settings::delete_profile,
            // Plugins
            plugins::get_plugins,
            plugins::get_plugin,
            plugins::enable_plugin,
            plugins::disable_plugin,
            plugins::refresh_plugins,
            plugins::get_plugins_directory,
            plugins::open_plugins_directory,
            plugins::export_plugin_list,
            plugins::get_plugin_load_order,
            // Plugin settings
            plugins::get_plugin_settings,
            plugins::save_plugin_settings,
            plugins::get_plugin_permissions,
            plugins::set_plugin_permission,
            // Plugin install/uninstall
            plugins::select_plugin_file,
            plugins::install_plugin_from_file,
            plugins::install_plugin_from_url,
            plugins::uninstall_plugin,
            // OCR
            ocr::ocr_recognize,
            ocr::ocr_recognize_with_options,
            ocr::get_available_languages,
            ocr::set_ocr_language,
            ocr::get_ocr_language,
            ocr::check_ocr_available,
            ocr::get_ocr_info,
            plugins::open_plugin_directory,
            // Script execution
            execution::run_script,
            execution::run_script_streaming,
            execution::stop_script,
            execution::get_running_processes,
            execution::stop_all_scripts,
            // Debug panel
            debug::debug_init_session,
            debug::debug_end_session,
            debug::debug_add_breakpoint,
            debug::debug_remove_breakpoint,
            debug::debug_toggle_breakpoint,
            debug::debug_list_breakpoints,
            debug::debug_clear_breakpoints,
            debug::debug_pause,
            debug::debug_resume,
            debug::debug_step_over,
            debug::debug_step_into,
            debug::debug_step_out,
            debug::debug_stop,
            debug::debug_get_state,
            debug::debug_get_variables,
            debug::debug_get_call_stack,
            debug::debug_get_current_position,
            debug::debug_evaluate_expression,
            // Image library
            image_library::list_images_command,
            image_library::get_image_thumbnail_command,
            image_library::delete_image_command,
            image_library::rename_image_command,
            image_library::find_unused_images_command,
            image_library::import_images_command
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                info!("Window close requested, saving settings");
                let app = window.app_handle();
                let state = app.state::<AppState>();

                // Save recent files before closing
                let settings = {
                    if let Ok(recent) = state.recent_files.lock() {
                        PersistentSettings {
                            recent_files: recent
                                .iter()
                                .map(|p| p.to_string_lossy().to_string())
                                .collect(),
                            ..Default::default()
                        }
                    } else {
                        PersistentSettings::default()
                    }
                };
                save_settings(app, &settings);
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ============================================================================
// Python Analysis Commands / Python解析コマンド
// ============================================================================

/// Analyze Python version from script content
/// スクリプト内容からPythonバージョンを解析
#[tauri::command]
fn analyze_python_version(content: &str) -> String {
    debug!(
        "Analyzing Python version for {} bytes of content",
        content.len()
    );

    if content.is_empty() {
        debug!("Empty content, returning unknown");
        return "unknown".to_string();
    }

    let version = SyntaxAnalyzer::detect_version(content);
    let result = match version {
        PythonVersion::Python2 => "python2",
        PythonVersion::Python3 => "python3",
        PythonVersion::Mixed => "mixed",
        PythonVersion::Unknown => "unknown",
    };

    debug!("Detected Python version: {}", result);
    result.to_string()
}

/// Analyze script content and detect Python version (placeholder)
/// スクリプト内容を分析しPythonバージョンを検出（プレースホルダー）
#[tauri::command]
fn analyze_script_content(content: &str) -> String {
    info!("Analyze script content requested ({} bytes)", content.len());

    if content.is_empty() {
        warn!("Empty script content");
        return "No script to run.".to_string();
    }

    let version = SyntaxAnalyzer::detect_version(content);
    let version_str = match version {
        PythonVersion::Python2 => "Python 2 (will be converted)",
        PythonVersion::Python3 => "Python 3",
        PythonVersion::Mixed => "Mixed (needs review)",
        PythonVersion::Unknown => "Unknown (treating as Python 3)",
    };

    info!("Script version detected: {}", version_str);

    format!(
        "=== Script Execution ===\nDetected: {}\n\n[Execution not yet implemented]",
        version_str
    )
}

/// Get core library version
/// コアライブラリのバージョンを取得
#[tauri::command]
fn get_core_version() -> String {
    debug!("Core version requested: {}", sikulid::VERSION);
    sikulid::VERSION.to_string()
}

// ============================================================================
// File Operations / ファイル操作
// ============================================================================

/// Read file content from path
/// パスからファイル内容を読み込み
#[tauri::command]
fn read_file(path: String, state: State<AppState>) -> FileOperationResult {
    info!("Reading file: {}", path);
    let path_buf = PathBuf::from(&path);

    // Validate path exists
    if !path_buf.exists() {
        error!("File not found: {}", path);
        return FileOperationResult {
            success: false,
            message: format!("File not found: {}", path),
            file_info: None,
        };
    }

    // Validate it's a file
    if !path_buf.is_file() {
        error!("Not a file: {}", path);
        return FileOperationResult {
            success: false,
            message: format!("Not a file: {}", path),
            file_info: None,
        };
    }

    // Read file content
    match fs::read_to_string(&path_buf) {
        Ok(content) => {
            let content_len = content.len();
            debug!("File read successfully: {} bytes", content_len);

            // Update current file
            if let Ok(mut current) = state.current_file.lock() {
                *current = Some(path_buf.clone());
                debug!("Current file updated");
            }

            // Reset modified flag
            if let Ok(mut modified) = state.is_modified.lock() {
                *modified = false;
            }

            let name = path_buf
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string());

            info!("File loaded: {} ({} bytes)", name, content_len);

            FileOperationResult {
                success: true,
                message: format!("File loaded: {}", name),
                file_info: Some(FileInfo {
                    path: path.clone(),
                    name,
                    content,
                }),
            }
        }
        Err(e) => {
            error!("Failed to read file: {}", e);
            FileOperationResult {
                success: false,
                message: format!("Failed to read file: {}", e),
                file_info: None,
            }
        }
    }
}

/// Write content to file
/// ファイルに内容を書き込み
#[tauri::command]
fn write_file(path: String, content: String, state: State<AppState>) -> FileOperationResult {
    info!("Writing file: {} ({} bytes)", path, content.len());
    let path_buf = PathBuf::from(&path);

    // Ensure parent directory exists
    if let Some(parent) = path_buf.parent() {
        if !parent.exists() {
            debug!("Creating parent directory: {:?}", parent);
            if let Err(e) = fs::create_dir_all(parent) {
                error!("Failed to create directory: {}", e);
                return FileOperationResult {
                    success: false,
                    message: format!("Failed to create directory: {}", e),
                    file_info: None,
                };
            }
        }
    }

    // Write to temporary file first (crash safety)
    let temp_path = path_buf.with_extension("tmp");
    debug!("Writing to temporary file: {:?}", temp_path);

    match fs::write(&temp_path, &content) {
        Ok(_) => {
            // Rename temp file to target
            debug!("Renaming temporary file to target");
            match fs::rename(&temp_path, &path_buf) {
                Ok(_) => {
                    // Update current file
                    if let Ok(mut current) = state.current_file.lock() {
                        *current = Some(path_buf.clone());
                    }

                    // Reset modified flag
                    if let Ok(mut modified) = state.is_modified.lock() {
                        *modified = false;
                    }

                    let name = path_buf
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "Untitled".to_string());

                    info!("File saved: {}", name);

                    FileOperationResult {
                        success: true,
                        message: format!("File saved: {}", name),
                        file_info: Some(FileInfo {
                            path,
                            name,
                            content,
                        }),
                    }
                }
                Err(e) => {
                    error!("Failed to rename temporary file: {}", e);
                    // Clean up temp file
                    let _ = fs::remove_file(&temp_path);
                    FileOperationResult {
                        success: false,
                        message: format!("Failed to save file: {}", e),
                        file_info: None,
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to write temporary file: {}", e);
            FileOperationResult {
                success: false,
                message: format!("Failed to write file: {}", e),
                file_info: None,
            }
        }
    }
}

/// Get current file path
/// 現在のファイルパスを取得
#[tauri::command]
fn get_current_file(state: State<AppState>) -> Option<String> {
    let result = state
        .current_file
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .map(|p| p.to_string_lossy().to_string());

    debug!("Get current file: {:?}", result);
    result
}

/// Set current file path
/// 現在のファイルパスを設定
#[tauri::command]
fn set_current_file(path: String, state: State<AppState>) {
    debug!("Setting current file: {}", path);
    if let Ok(mut current) = state.current_file.lock() {
        *current = Some(PathBuf::from(path));
    }
}

/// Clear current file (for new file)
/// 現在のファイルをクリア（新規ファイル用）
#[tauri::command]
fn clear_current_file(state: State<AppState>) {
    debug!("Clearing current file");
    if let Ok(mut current) = state.current_file.lock() {
        *current = None;
    }
    if let Ok(mut modified) = state.is_modified.lock() {
        *modified = false;
    }
}

// ============================================================================
// State Management / 状態管理
// ============================================================================

/// Set modified flag
/// 変更フラグを設定
#[tauri::command]
fn set_modified(modified: bool, state: State<AppState>) {
    debug!("Setting modified flag: {}", modified);
    if let Ok(mut flag) = state.is_modified.lock() {
        *flag = modified;
    }
}

/// Check if content is modified
/// 内容が変更されているか確認
#[tauri::command]
fn is_modified(state: State<AppState>) -> bool {
    let result = state.is_modified.lock().ok().map(|g| *g).unwrap_or(false);
    debug!("Is modified: {}", result);
    result
}

// ============================================================================
// Recent Files Management / 最近使用ファイル管理
// ============================================================================

const MAX_RECENT_FILES: usize = 10;

/// Get recent files list
/// 最近使用したファイルリストを取得
#[tauri::command]
fn get_recent_files(state: State<AppState>) -> Vec<String> {
    let result: Vec<String> = state
        .recent_files
        .lock()
        .ok()
        .map(|guard| {
            guard
                .iter()
                .filter(|p| p.exists()) // Filter out non-existent files
                .map(|p| p.to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();

    debug!("Get recent files: {} files", result.len());
    result
}

/// Add file to recent files list
/// 最近使用したファイルリストに追加
#[tauri::command]
fn add_recent_file(path: String, state: State<AppState>) {
    debug!("Adding to recent files: {}", path);
    let path_buf = PathBuf::from(&path);

    if let Ok(mut recent) = state.recent_files.lock() {
        // Remove if already exists (to move to front)
        recent.retain(|p| p != &path_buf);

        // Add to front
        recent.insert(0, path_buf);

        // Trim to max size
        recent.truncate(MAX_RECENT_FILES);

        debug!("Recent files count: {}", recent.len());
    }
}

/// Clear recent files list
/// 最近使用したファイルリストをクリア
#[tauri::command]
fn clear_recent_files(state: State<AppState>) {
    info!("Clearing recent files");
    if let Ok(mut recent) = state.recent_files.lock() {
        recent.clear();
    }
}

// ============================================================================
// Settings Persistence Commands / 設定永続化コマンド
// ============================================================================

/// Save application settings
/// アプリケーション設定を保存
#[tauri::command]
fn save_app_settings(app: AppHandle, state: State<AppState>) {
    info!("Saving application settings");
    if let Ok(recent) = state.recent_files.lock() {
        let settings = PersistentSettings {
            recent_files: recent
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect(),
            ..Default::default()
        };
        save_settings(&app, &settings);
    }
}

/// Load application settings
/// アプリケーション設定を読み込み
#[tauri::command]
fn load_app_settings(app: AppHandle, state: State<AppState>) {
    info!("Loading application settings");
    let settings = load_settings(&app);

    if let Ok(mut recent) = state.recent_files.lock() {
        *recent = settings
            .recent_files
            .iter()
            .map(PathBuf::from)
            .filter(|p| p.exists())
            .collect();
        info!("Loaded {} recent files", recent.len());
    }
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_python_version_empty() {
        assert_eq!(analyze_python_version(""), "unknown");
    }

    #[test]
    fn test_analyze_python_version_python2() {
        let code = "print 'hello'";
        assert_eq!(analyze_python_version(code), "python2");
    }

    #[test]
    fn test_analyze_python_version_python3() {
        // Use definitive Python 3 syntax (f-string)
        // print('hello') is ambiguous - could be Python 2 with __future__
        let code = "print(f'hello {name}')";
        assert_eq!(analyze_python_version(code), "python3");
    }

    #[test]
    fn test_file_operation_result_serialization() {
        let result = FileOperationResult {
            success: true,
            message: "OK".to_string(),
            file_info: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(!json.contains("file_info")); // skip_serializing_if works
    }

    #[test]
    fn test_persistent_settings_default() {
        let settings = PersistentSettings::default();
        assert!(settings.recent_files.is_empty());
        assert!(settings.last_directory.is_none());
    }

    #[test]
    fn test_persistent_settings_serialization() {
        let settings = PersistentSettings {
            recent_files: vec!["file1.py".to_string(), "file2.py".to_string()],
            last_directory: Some("/home/user".to_string()),
            window_width: Some(1024),
            window_height: Some(768),
        };
        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("file1.py"));
        assert!(json.contains("/home/user"));

        let parsed: PersistentSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.recent_files.len(), 2);
        assert_eq!(parsed.window_width, Some(1024));
    }
}
