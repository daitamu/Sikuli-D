//! Settings Commands / 設定コマンド
//!
//! Provides Tauri commands for settings management.
//! 設定管理のTauriコマンドを提供します。

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use sikulid_core::settings::{
    AppSettings, HotkeyBinding, HotkeySettings, Language, ProfileManager, Theme,
};
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Settings State / 設定状態
// ============================================================================

/// State for settings management
/// 設定管理の状態
#[derive(Default)]
pub struct SettingsState {
    /// Current application settings / 現在のアプリケーション設定
    pub settings: Mutex<AppSettings>,
    /// Profile manager / プロファイルマネージャー
    pub profile_manager: Mutex<ProfileManager>,
}

impl SettingsState {
    pub fn new() -> Self {
        Self {
            settings: Mutex::new(AppSettings::load_or_default()),
            profile_manager: Mutex::new(ProfileManager::load_or_default()),
        }
    }
}

// ============================================================================
// Settings DTOs / 設定DTO
// ============================================================================

/// Settings result
/// 設定結果
#[derive(Serialize)]
pub struct SettingsResult {
    pub success: bool,
    pub message: String,
}

/// Complete settings for frontend
/// フロントエンド用の完全な設定
#[derive(Serialize, Deserialize)]
pub struct FrontendSettings {
    pub general: GeneralSettingsDto,
    pub editor: EditorSettingsDto,
    pub execution: ExecutionSettingsDto,
    pub image_recognition: ImageRecognitionSettingsDto,
    pub ocr: OcrSettingsDto,
    pub hotkeys: Vec<HotkeyBindingDto>,
}

#[derive(Serialize, Deserialize)]
pub struct GeneralSettingsDto {
    pub language: String,
    pub check_updates: bool,
    pub theme: String,
    pub recent_files_limit: u32,
    pub show_welcome: bool,
}

#[derive(Serialize, Deserialize)]
pub struct EditorSettingsDto {
    pub font_size: u32,
    pub font_family: String,
    pub tab_size: u32,
    pub use_spaces: bool,
    pub word_wrap: bool,
    pub line_numbers: bool,
    pub highlight_current_line: bool,
    pub auto_close_brackets: bool,
    pub code_completion: bool,
    pub minimap: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ExecutionSettingsDto {
    pub python_path: String,
    pub default_timeout: u32,
    pub auto_save_before_run: bool,
    pub clear_output_on_run: bool,
    pub show_execution_time: bool,
}

#[derive(Serialize, Deserialize)]
pub struct ImageRecognitionSettingsDto {
    pub default_similarity: f64,
    pub highlight_matches: bool,
    pub highlight_duration_ms: u32,
    pub auto_wait_timeout: f64,
    pub cache_templates: bool,
}

#[derive(Serialize, Deserialize)]
pub struct OcrSettingsDto {
    pub language: String,
    pub tessdata_path: String,
    pub enable_cache: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HotkeyBindingDto {
    pub key: String,
    pub action: String,
    pub enabled: bool,
}

// ============================================================================
// Conversion Functions / 変換関数
// ============================================================================

impl From<&AppSettings> for FrontendSettings {
    fn from(settings: &AppSettings) -> Self {
        Self {
            general: GeneralSettingsDto {
                language: match settings.general.language {
                    Language::English => "en".to_string(),
                    Language::Japanese => "ja".to_string(),
                },
                check_updates: settings.general.check_updates,
                theme: match settings.general.theme {
                    Theme::Light => "light".to_string(),
                    Theme::Dark => "dark".to_string(),
                    Theme::System => "system".to_string(),
                },
                recent_files_limit: settings.general.recent_files_limit,
                show_welcome: settings.general.show_welcome,
            },
            editor: EditorSettingsDto {
                font_size: settings.editor.font_size,
                font_family: settings.editor.font_family.clone(),
                tab_size: settings.editor.tab_size,
                use_spaces: settings.editor.use_spaces,
                word_wrap: settings.editor.word_wrap,
                line_numbers: settings.editor.line_numbers,
                highlight_current_line: settings.editor.highlight_current_line,
                auto_close_brackets: settings.editor.auto_close_brackets,
                code_completion: settings.editor.code_completion,
                minimap: settings.editor.minimap,
            },
            execution: ExecutionSettingsDto {
                python_path: settings.execution.python_path.clone(),
                default_timeout: settings.execution.default_timeout,
                auto_save_before_run: settings.execution.auto_save_before_run,
                clear_output_on_run: settings.execution.clear_output_on_run,
                show_execution_time: settings.execution.show_execution_time,
            },
            image_recognition: ImageRecognitionSettingsDto {
                default_similarity: settings.image_recognition.default_similarity,
                highlight_matches: settings.image_recognition.highlight_matches,
                highlight_duration_ms: settings.image_recognition.highlight_duration_ms,
                auto_wait_timeout: settings.image_recognition.auto_wait_timeout,
                cache_templates: settings.image_recognition.cache_templates,
            },
            ocr: OcrSettingsDto {
                language: settings.ocr.language.clone(),
                tessdata_path: settings.ocr.tessdata_path.clone(),
                enable_cache: settings.ocr.enable_cache,
            },
            hotkeys: settings
                .hotkeys
                .bindings
                .iter()
                .map(|b| HotkeyBindingDto {
                    key: b.key.clone(),
                    action: b.action.clone(),
                    enabled: b.enabled,
                })
                .collect(),
        }
    }
}

fn apply_frontend_settings(settings: &mut AppSettings, frontend: &FrontendSettings) {
    // General
    settings.general.language = match frontend.general.language.as_str() {
        "ja" => Language::Japanese,
        _ => Language::English,
    };
    settings.general.check_updates = frontend.general.check_updates;
    settings.general.theme = match frontend.general.theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => Theme::System,
    };
    settings.general.recent_files_limit = frontend.general.recent_files_limit;
    settings.general.show_welcome = frontend.general.show_welcome;

    // Editor
    settings.editor.font_size = frontend.editor.font_size;
    settings.editor.font_family = frontend.editor.font_family.clone();
    settings.editor.tab_size = frontend.editor.tab_size;
    settings.editor.use_spaces = frontend.editor.use_spaces;
    settings.editor.word_wrap = frontend.editor.word_wrap;
    settings.editor.line_numbers = frontend.editor.line_numbers;
    settings.editor.highlight_current_line = frontend.editor.highlight_current_line;
    settings.editor.auto_close_brackets = frontend.editor.auto_close_brackets;
    settings.editor.code_completion = frontend.editor.code_completion;
    settings.editor.minimap = frontend.editor.minimap;

    // Execution
    settings.execution.python_path = frontend.execution.python_path.clone();
    settings.execution.default_timeout = frontend.execution.default_timeout;
    settings.execution.auto_save_before_run = frontend.execution.auto_save_before_run;
    settings.execution.clear_output_on_run = frontend.execution.clear_output_on_run;
    settings.execution.show_execution_time = frontend.execution.show_execution_time;

    // Image Recognition
    settings.image_recognition.default_similarity = frontend.image_recognition.default_similarity;
    settings.image_recognition.highlight_matches = frontend.image_recognition.highlight_matches;
    settings.image_recognition.highlight_duration_ms =
        frontend.image_recognition.highlight_duration_ms;
    settings.image_recognition.auto_wait_timeout = frontend.image_recognition.auto_wait_timeout;
    settings.image_recognition.cache_templates = frontend.image_recognition.cache_templates;

    // OCR
    settings.ocr.language = frontend.ocr.language.clone();
    settings.ocr.tessdata_path = frontend.ocr.tessdata_path.clone();
    settings.ocr.enable_cache = frontend.ocr.enable_cache;

    // Hotkeys
    settings.hotkeys.bindings = frontend
        .hotkeys
        .iter()
        .map(|b| HotkeyBinding {
            key: b.key.clone(),
            action: b.action.clone(),
            enabled: b.enabled,
        })
        .collect();
}

// ============================================================================
// Settings Commands / 設定コマンド
// ============================================================================

/// Get all settings
/// すべての設定を取得
#[tauri::command]
pub fn get_settings(state: State<'_, SettingsState>) -> FrontendSettings {
    debug!("Getting all settings");
    let settings = state.settings.lock().unwrap();
    FrontendSettings::from(&*settings)
}

/// Save all settings
/// すべての設定を保存
#[tauri::command]
pub fn save_settings(
    frontend: FrontendSettings,
    state: State<'_, SettingsState>,
) -> SettingsResult {
    info!("Saving settings");
    let mut settings = state.settings.lock().unwrap();
    apply_frontend_settings(&mut settings, &frontend);

    match settings.save() {
        Ok(_) => {
            info!("Settings saved successfully");
            SettingsResult {
                success: true,
                message: "Settings saved".to_string(),
            }
        }
        Err(e) => {
            warn!("Failed to save settings: {}", e);
            SettingsResult {
                success: false,
                message: format!("Failed to save settings: {}", e),
            }
        }
    }
}

/// Reset settings to defaults
/// 設定をデフォルトにリセット
#[tauri::command]
pub fn reset_settings(section: Option<String>, state: State<'_, SettingsState>) -> SettingsResult {
    let mut settings = state.settings.lock().unwrap();

    if let Some(section) = section {
        info!("Resetting section: {}", section);
        settings.reset_section(&section);
    } else {
        info!("Resetting all settings");
        settings.reset();
    }

    match settings.save() {
        Ok(_) => SettingsResult {
            success: true,
            message: "Settings reset".to_string(),
        },
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to save after reset: {}", e),
        },
    }
}

/// Get current theme
/// 現在のテーマを取得
#[tauri::command]
pub fn get_theme(state: State<'_, SettingsState>) -> String {
    let settings = state.settings.lock().unwrap();
    match settings.general.theme {
        Theme::Light => "light".to_string(),
        Theme::Dark => "dark".to_string(),
        Theme::System => "system".to_string(),
    }
}

/// Set theme
/// テーマを設定
#[tauri::command]
pub fn set_theme(theme: String, state: State<'_, SettingsState>) -> SettingsResult {
    info!("Setting theme: {}", theme);
    let mut settings = state.settings.lock().unwrap();
    settings.general.theme = match theme.as_str() {
        "light" => Theme::Light,
        "dark" => Theme::Dark,
        _ => Theme::System,
    };

    match settings.save() {
        Ok(_) => SettingsResult {
            success: true,
            message: "Theme updated".to_string(),
        },
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to save theme: {}", e),
        },
    }
}

// ============================================================================
// Hotkey Commands / ホットキーコマンド
// ============================================================================

/// Get all hotkeys
/// すべてのホットキーを取得
#[tauri::command]
pub fn get_hotkeys(state: State<'_, SettingsState>) -> Vec<HotkeyBindingDto> {
    debug!("Getting hotkeys");
    let settings = state.settings.lock().unwrap();
    settings
        .hotkeys
        .bindings
        .iter()
        .map(|b| HotkeyBindingDto {
            key: b.key.clone(),
            action: b.action.clone(),
            enabled: b.enabled,
        })
        .collect()
}

/// Update a hotkey
/// ホットキーを更新
#[tauri::command]
pub fn update_hotkey(
    action: String,
    new_key: String,
    state: State<'_, SettingsState>,
) -> SettingsResult {
    info!("Updating hotkey for {}: {}", action, new_key);
    let mut settings = state.settings.lock().unwrap();

    // Check for conflicts
    let conflict = settings
        .hotkeys
        .bindings
        .iter()
        .find(|b| b.key == new_key && b.action != action && b.enabled);

    if let Some(conflict) = conflict {
        return SettingsResult {
            success: false,
            message: format!(
                "Hotkey '{}' is already used by '{}'",
                new_key, conflict.action
            ),
        };
    }

    // Update the hotkey
    if let Some(binding) = settings
        .hotkeys
        .bindings
        .iter_mut()
        .find(|b| b.action == action)
    {
        binding.key = new_key;
        match settings.save() {
            Ok(_) => SettingsResult {
                success: true,
                message: "Hotkey updated".to_string(),
            },
            Err(e) => SettingsResult {
                success: false,
                message: format!("Failed to save: {}", e),
            },
        }
    } else {
        SettingsResult {
            success: false,
            message: format!("Action '{}' not found", action),
        }
    }
}

/// Toggle hotkey enabled state
/// ホットキーの有効/無効を切り替え
#[tauri::command]
pub fn toggle_hotkey(
    action: String,
    enabled: bool,
    state: State<'_, SettingsState>,
) -> SettingsResult {
    info!("Toggling hotkey {}: {}", action, enabled);
    let mut settings = state.settings.lock().unwrap();

    if let Some(binding) = settings
        .hotkeys
        .bindings
        .iter_mut()
        .find(|b| b.action == action)
    {
        binding.enabled = enabled;
        match settings.save() {
            Ok(_) => SettingsResult {
                success: true,
                message: "Hotkey toggled".to_string(),
            },
            Err(e) => SettingsResult {
                success: false,
                message: format!("Failed to save: {}", e),
            },
        }
    } else {
        SettingsResult {
            success: false,
            message: format!("Action '{}' not found", action),
        }
    }
}

/// Reset hotkeys to defaults
/// ホットキーをデフォルトにリセット
#[tauri::command]
pub fn reset_hotkeys(state: State<'_, SettingsState>) -> SettingsResult {
    info!("Resetting hotkeys to defaults");
    let mut settings = state.settings.lock().unwrap();
    settings.hotkeys = HotkeySettings::default();

    match settings.save() {
        Ok(_) => SettingsResult {
            success: true,
            message: "Hotkeys reset to defaults".to_string(),
        },
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to save: {}", e),
        },
    }
}

// ============================================================================
// Profile Commands / プロファイルコマンド
// ============================================================================

/// Profile info for frontend
/// フロントエンド用プロファイル情報
#[derive(Serialize)]
pub struct ProfileInfo {
    pub name: String,
    pub description: String,
    pub is_default: bool,
    pub is_active: bool,
}

/// Get all profiles
/// すべてのプロファイルを取得
#[tauri::command]
pub fn get_profiles(state: State<'_, SettingsState>) -> Vec<ProfileInfo> {
    debug!("Getting profiles");
    let manager = state.profile_manager.lock().unwrap();
    let active = manager.active_profile.clone();

    manager
        .profiles
        .values()
        .map(|p| ProfileInfo {
            name: p.name.clone(),
            description: p.description.clone(),
            is_default: p.is_default,
            is_active: p.name == active,
        })
        .collect()
}

/// Switch to a profile
/// プロファイルに切り替え
#[tauri::command]
pub fn switch_profile(name: String, state: State<'_, SettingsState>) -> SettingsResult {
    info!("Switching to profile: {}", name);
    let mut manager = state.profile_manager.lock().unwrap();

    match manager.switch_profile(&name) {
        Ok(_) => {
            // Update current settings from the new profile
            if let Some(profile) = manager.active() {
                let mut settings = state.settings.lock().unwrap();
                *settings = profile.settings.clone();
            }

            if let Err(e) = manager.save() {
                warn!("Failed to save profile manager: {}", e);
            }

            SettingsResult {
                success: true,
                message: format!("Switched to profile '{}'", name),
            }
        }
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to switch profile: {}", e),
        },
    }
}

/// Create a new profile
/// 新しいプロファイルを作成
#[tauri::command]
pub fn create_profile(name: String, state: State<'_, SettingsState>) -> SettingsResult {
    info!("Creating profile: {}", name);
    let mut manager = state.profile_manager.lock().unwrap();

    match manager.create_profile(&name) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                warn!("Failed to save profile manager: {}", e);
            }
            SettingsResult {
                success: true,
                message: format!("Profile '{}' created", name),
            }
        }
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to create profile: {}", e),
        },
    }
}

/// Delete a profile
/// プロファイルを削除
#[tauri::command]
pub fn delete_profile(name: String, state: State<'_, SettingsState>) -> SettingsResult {
    info!("Deleting profile: {}", name);
    let mut manager = state.profile_manager.lock().unwrap();

    match manager.delete_profile(&name) {
        Ok(_) => {
            if let Err(e) = manager.save() {
                warn!("Failed to save profile manager: {}", e);
            }
            SettingsResult {
                success: true,
                message: format!("Profile '{}' deleted", name),
            }
        }
        Err(e) => SettingsResult {
            success: false,
            message: format!("Failed to delete profile: {}", e),
        },
    }
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontend_settings_conversion() {
        let settings = AppSettings::default();
        let frontend = FrontendSettings::from(&settings);

        assert_eq!(frontend.general.theme, "dark");
        assert_eq!(frontend.editor.font_size, 14);
        assert!(frontend.execution.auto_save_before_run);
    }

    #[test]
    fn test_hotkey_dto() {
        let dto = HotkeyBindingDto {
            key: "F5".to_string(),
            action: "run_script".to_string(),
            enabled: true,
        };
        assert_eq!(dto.key, "F5");
        assert!(dto.enabled);
    }
}
