//! Settings and configuration management
//! 設定・構成管理モジュール
//!
//! Provides configuration management for the SikuliX IDE including
//! general settings, editor preferences, execution options, and profiles.
//! SikuliX IDEの設定管理機能を提供します。
//! 一般設定、エディタ設定、実行オプション、プロファイルを含みます。

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{Result, SikulixError};

/// Application theme
/// アプリケーションテーマ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Light theme / ライトテーマ
    Light,
    /// Dark theme / ダークテーマ
    #[default]
    Dark,
    /// Follow system setting / システム設定に従う
    System,
}

/// UI language setting
/// UI言語設定
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// English
    #[default]
    English,
    /// Japanese / 日本語
    Japanese,
}

/// General application settings
/// 一般アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    /// UI language / UI言語
    pub language: Language,
    /// Check for updates on startup / 起動時にアップデートを確認
    pub check_updates: bool,
    /// Application theme / アプリケーションテーマ
    pub theme: Theme,
    /// Recent files list size / 最近使ったファイル一覧のサイズ
    pub recent_files_limit: u32,
    /// Show welcome screen on startup / 起動時にウェルカム画面を表示
    pub show_welcome: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            language: Language::English,
            check_updates: true,
            theme: Theme::Dark,
            recent_files_limit: 10,
            show_welcome: true,
        }
    }
}

/// Editor settings
/// エディタ設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorSettings {
    /// Font size in pixels / フォントサイズ（ピクセル）
    pub font_size: u32,
    /// Font family / フォントファミリー
    pub font_family: String,
    /// Tab size (spaces) / タブサイズ（スペース数）
    pub tab_size: u32,
    /// Use spaces instead of tabs / タブの代わりにスペースを使用
    pub use_spaces: bool,
    /// Word wrap mode / 折り返し設定
    pub word_wrap: bool,
    /// Show line numbers / 行番号を表示
    pub line_numbers: bool,
    /// Highlight current line / 現在の行をハイライト
    pub highlight_current_line: bool,
    /// Auto-close brackets / 括弧の自動補完
    pub auto_close_brackets: bool,
    /// Enable code completion / コード補完を有効化
    pub code_completion: bool,
    /// Minimap visible / ミニマップを表示
    pub minimap: bool,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            font_size: 14,
            font_family: "Consolas, Monaco, monospace".to_string(),
            tab_size: 4,
            use_spaces: true,
            word_wrap: true,
            line_numbers: true,
            highlight_current_line: true,
            auto_close_brackets: true,
            code_completion: true,
            minimap: false,
        }
    }
}

/// Script execution settings
/// スクリプト実行設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSettings {
    /// Custom Python path (empty = auto-detect) / カスタムPythonパス（空 = 自動検出）
    pub python_path: String,
    /// Default script timeout in seconds / デフォルトスクリプトタイムアウト（秒）
    pub default_timeout: u32,
    /// Auto-save before running / 実行前に自動保存
    pub auto_save_before_run: bool,
    /// Clear output panel before running / 実行前に出力パネルをクリア
    pub clear_output_on_run: bool,
    /// Show execution time / 実行時間を表示
    pub show_execution_time: bool,
}

impl Default for ExecutionSettings {
    fn default() -> Self {
        Self {
            python_path: String::new(),
            default_timeout: 300,
            auto_save_before_run: true,
            clear_output_on_run: true,
            show_execution_time: true,
        }
    }
}

/// Image recognition settings
/// 画像認識設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRecognitionSettings {
    /// Default similarity threshold (0.0 - 1.0) / デフォルト類似度閾値
    pub default_similarity: f64,
    /// Highlight matches on screen / 画面上でマッチをハイライト
    pub highlight_matches: bool,
    /// Highlight duration in milliseconds / ハイライト時間（ミリ秒）
    pub highlight_duration_ms: u32,
    /// Auto-wait timeout for find operations / find操作の自動待機タイムアウト（秒）
    pub auto_wait_timeout: f64,
    /// Cache template images / テンプレート画像をキャッシュ
    pub cache_templates: bool,
}

impl Default for ImageRecognitionSettings {
    fn default() -> Self {
        Self {
            default_similarity: 0.7,
            highlight_matches: true,
            highlight_duration_ms: 1000,
            auto_wait_timeout: 3.0,
            cache_templates: true,
        }
    }
}

/// OCR settings
/// OCR設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrSettings {
    /// OCR language code (e.g., "eng", "jpn", "eng+jpn") / OCR言語コード
    pub language: String,
    /// Custom tessdata path (empty = default) / カスタムtessdataパス
    pub tessdata_path: String,
    /// Enable OCR result caching / OCR結果キャッシュを有効化
    pub enable_cache: bool,
}

impl Default for OcrSettings {
    fn default() -> Self {
        Self {
            language: "eng".to_string(),
            tessdata_path: String::new(),
            enable_cache: true,
        }
    }
}

/// Hotkey binding
/// ホットキーバインディング
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyBinding {
    /// Key combination (e.g., "Ctrl+S", "F5") / キーの組み合わせ
    pub key: String,
    /// Action to perform / 実行するアクション
    pub action: String,
    /// Whether this binding is enabled / このバインディングが有効かどうか
    pub enabled: bool,
}

/// Hotkey conflict information
/// ホットキー競合情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConflict {
    /// The conflicting key combination / 競合するキーの組み合わせ
    pub key: String,
    /// Actions that share this key / このキーを共有するアクション
    pub actions: Vec<String>,
}

/// Reserved system hotkey information
/// システム予約済みホットキー情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHotkey {
    /// Key combination / キーの組み合わせ
    pub key: String,
    /// Description of system function / システム機能の説明
    pub description: String,
}

/// Hotkey settings
/// ホットキー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeySettings {
    /// List of hotkey bindings / ホットキーバインディングのリスト
    pub bindings: Vec<HotkeyBinding>,
}

impl HotkeySettings {
    /// Normalize a key string for comparison
    /// 比較用にキー文字列を正規化
    fn normalize_key(key: &str) -> String {
        // Remove spaces and convert to lowercase for comparison
        // Sort modifiers for consistent ordering: Alt, Ctrl, Meta, Shift
        let key = key.trim();
        let parts: Vec<&str> = key.split('+').map(|s| s.trim()).collect();

        if parts.len() == 1 {
            return parts[0].to_uppercase();
        }

        let mut modifiers: Vec<String> = Vec::new();
        let mut main_key = String::new();

        for part in parts {
            let upper = part.to_uppercase();
            match upper.as_str() {
                "CTRL" | "CONTROL" => modifiers.push("Ctrl".to_string()),
                "ALT" => modifiers.push("Alt".to_string()),
                "SHIFT" => modifiers.push("Shift".to_string()),
                "META" | "WIN" | "CMD" | "COMMAND" => modifiers.push("Meta".to_string()),
                _ => main_key = upper,
            }
        }

        modifiers.sort();
        if !main_key.is_empty() {
            modifiers.push(main_key);
        }

        modifiers.join("+")
    }

    /// Detect conflicts between hotkey bindings
    /// ホットキーバインディング間の競合を検出
    ///
    /// Returns a list of conflicts where the same key is assigned to multiple actions.
    /// 同じキーが複数のアクションに割り当てられている競合のリストを返します。
    pub fn detect_conflicts(&self) -> Vec<HotkeyConflict> {
        let mut key_actions: HashMap<String, Vec<String>> = HashMap::new();

        // Group enabled bindings by normalized key
        for binding in &self.bindings {
            if binding.enabled {
                let normalized = Self::normalize_key(&binding.key);
                key_actions
                    .entry(normalized)
                    .or_default()
                    .push(binding.action.clone());
            }
        }

        // Find keys with multiple actions
        key_actions
            .into_iter()
            .filter(|(_, actions)| actions.len() > 1)
            .map(|(key, actions)| HotkeyConflict { key, actions })
            .collect()
    }

    /// Check if a specific key combination has conflicts
    /// 特定のキーの組み合わせに競合があるかチェック
    pub fn has_conflict(&self, key: &str) -> bool {
        let normalized = Self::normalize_key(key);
        let count = self
            .bindings
            .iter()
            .filter(|b| b.enabled && Self::normalize_key(&b.key) == normalized)
            .count();
        count > 1
    }

    /// Get common system reserved hotkeys
    /// 一般的なシステム予約ホットキーを取得
    pub fn system_reserved_hotkeys() -> Vec<SystemHotkey> {
        vec![
            SystemHotkey {
                key: "Alt+F4".to_string(),
                description: "Close window".to_string(),
            },
            SystemHotkey {
                key: "Alt+Tab".to_string(),
                description: "Switch window".to_string(),
            },
            SystemHotkey {
                key: "Ctrl+Alt+Delete".to_string(),
                description: "Security options".to_string(),
            },
            SystemHotkey {
                key: "Meta+D".to_string(),
                description: "Show desktop".to_string(),
            },
            SystemHotkey {
                key: "Meta+E".to_string(),
                description: "File explorer".to_string(),
            },
            SystemHotkey {
                key: "Meta+L".to_string(),
                description: "Lock screen".to_string(),
            },
            SystemHotkey {
                key: "Meta+R".to_string(),
                description: "Run dialog".to_string(),
            },
            SystemHotkey {
                key: "PrintScreen".to_string(),
                description: "Screenshot".to_string(),
            },
        ]
    }

    /// Check if a key conflicts with system reserved hotkeys
    /// キーがシステム予約ホットキーと競合するかチェック
    pub fn conflicts_with_system(&self, key: &str) -> Option<SystemHotkey> {
        let normalized = Self::normalize_key(key);
        Self::system_reserved_hotkeys()
            .into_iter()
            .find(|sys| Self::normalize_key(&sys.key) == normalized)
    }

    /// Set or update a hotkey binding
    /// ホットキーバインディングを設定または更新
    ///
    /// If an action with the same name exists, it updates the key.
    /// Otherwise, it adds a new binding.
    /// 同じ名前のアクションが存在する場合はキーを更新します。
    /// そうでなければ、新しいバインディングを追加します。
    pub fn set_hotkey(&mut self, action: &str, key: &str) -> Result<()> {
        // Validate key format
        if key.trim().is_empty() {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Key cannot be empty",
            )));
        }

        // Check if action already exists
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action == action) {
            info!(
                "Updating hotkey for action '{}': {} -> {}",
                action, binding.key, key
            );
            binding.key = key.to_string();
        } else {
            info!("Adding new hotkey: {} -> {}", key, action);
            self.bindings.push(HotkeyBinding {
                key: key.to_string(),
                action: action.to_string(),
                enabled: true,
            });
        }

        Ok(())
    }

    /// Remove a hotkey binding by action name
    /// アクション名でホットキーバインディングを削除
    pub fn remove_hotkey(&mut self, action: &str) -> bool {
        let initial_len = self.bindings.len();
        self.bindings.retain(|b| b.action != action);
        let removed = self.bindings.len() < initial_len;
        if removed {
            info!("Removed hotkey for action: {}", action);
        }
        removed
    }

    /// Enable or disable a hotkey binding
    /// ホットキーバインディングを有効化または無効化
    pub fn set_enabled(&mut self, action: &str, enabled: bool) -> bool {
        if let Some(binding) = self.bindings.iter_mut().find(|b| b.action == action) {
            binding.enabled = enabled;
            info!(
                "Hotkey for '{}' {}",
                action,
                if enabled { "enabled" } else { "disabled" }
            );
            true
        } else {
            false
        }
    }

    /// Get hotkey for a specific action
    /// 特定のアクションのホットキーを取得
    pub fn get_hotkey(&self, action: &str) -> Option<&HotkeyBinding> {
        self.bindings.iter().find(|b| b.action == action)
    }

    /// Get action for a specific key (first match, enabled only)
    /// 特定のキーのアクションを取得（最初のマッチ、有効なもののみ）
    pub fn get_action(&self, key: &str) -> Option<&str> {
        let normalized = Self::normalize_key(key);
        self.bindings
            .iter()
            .find(|b| b.enabled && Self::normalize_key(&b.key) == normalized)
            .map(|b| b.action.as_str())
    }

    /// Validate all hotkey bindings and return any issues
    /// すべてのホットキーバインディングを検証し、問題を返す
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();

        // Check for internal conflicts
        let conflicts = self.detect_conflicts();
        for conflict in conflicts {
            issues.push(format!(
                "Key '{}' is assigned to multiple actions: {}",
                conflict.key,
                conflict.actions.join(", ")
            ));
        }

        // Check for system conflicts
        for binding in &self.bindings {
            if binding.enabled {
                if let Some(sys) = self.conflicts_with_system(&binding.key) {
                    issues.push(format!(
                        "Key '{}' conflicts with system hotkey: {}",
                        binding.key, sys.description
                    ));
                }
            }
        }

        issues
    }
}

impl Default for HotkeySettings {
    fn default() -> Self {
        Self {
            bindings: vec![
                HotkeyBinding {
                    key: "F5".to_string(),
                    action: "run_script".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "F6".to_string(),
                    action: "stop_script".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "F9".to_string(),
                    action: "toggle_breakpoint".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+S".to_string(),
                    action: "save_file".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+O".to_string(),
                    action: "open_file".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+N".to_string(),
                    action: "new_file".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+Shift+C".to_string(),
                    action: "capture_screen".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+F".to_string(),
                    action: "find".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+H".to_string(),
                    action: "replace".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+Z".to_string(),
                    action: "undo".to_string(),
                    enabled: true,
                },
                HotkeyBinding {
                    key: "Ctrl+Y".to_string(),
                    action: "redo".to_string(),
                    enabled: true,
                },
            ],
        }
    }
}

/// Complete application settings
/// アプリケーション設定全体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// General settings / 一般設定
    pub general: GeneralSettings,
    /// Editor settings / エディタ設定
    pub editor: EditorSettings,
    /// Execution settings / 実行設定
    pub execution: ExecutionSettings,
    /// Image recognition settings / 画像認識設定
    pub image_recognition: ImageRecognitionSettings,
    /// OCR settings / OCR設定
    pub ocr: OcrSettings,
    /// Hotkey settings / ホットキー設定
    pub hotkeys: HotkeySettings,
}

impl AppSettings {
    /// Create new default settings
    /// 新しいデフォルト設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Load settings from a file
    /// ファイルから設定を読み込み
    pub fn load_from_file(path: &Path) -> Result<Self> {
        info!("Loading settings from: {}", path.display());
        let content = fs::read_to_string(path).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Failed to read settings file: {}", e),
            ))
        })?;

        let settings: Self = serde_json::from_str(&content).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse settings: {}", e),
            ))
        })?;

        debug!("Settings loaded successfully");
        Ok(settings)
    }

    /// Save settings to a file
    /// 設定をファイルに保存
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        info!("Saving settings to: {}", path.display());

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize settings: {}", e),
            ))
        })?;

        fs::write(path, content)?;
        debug!("Settings saved successfully");
        Ok(())
    }

    /// Get default settings file path
    /// デフォルト設定ファイルパスを取得
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sikulix")
            .join("settings.json")
    }

    /// Load settings from default location, or create default if not found
    /// デフォルト場所から設定を読み込み、見つからなければデフォルトを作成
    pub fn load_or_default() -> Self {
        let path = Self::default_path();
        match Self::load_from_file(&path) {
            Ok(settings) => settings,
            Err(e) => {
                warn!("Could not load settings: {}. Using defaults.", e);
                Self::default()
            }
        }
    }

    /// Save settings to default location
    /// 設定をデフォルト場所に保存
    pub fn save(&self) -> Result<()> {
        self.save_to_file(&Self::default_path())
    }

    /// Reset all settings to defaults
    /// すべての設定をデフォルトにリセット
    pub fn reset(&mut self) {
        info!("Resetting all settings to defaults");
        *self = Self::default();
    }

    /// Reset a specific section to defaults
    /// 特定のセクションをデフォルトにリセット
    pub fn reset_section(&mut self, section: &str) {
        info!("Resetting {} settings to defaults", section);
        match section {
            "general" => self.general = GeneralSettings::default(),
            "editor" => self.editor = EditorSettings::default(),
            "execution" => self.execution = ExecutionSettings::default(),
            "image_recognition" => self.image_recognition = ImageRecognitionSettings::default(),
            "ocr" => self.ocr = OcrSettings::default(),
            "hotkeys" => self.hotkeys = HotkeySettings::default(),
            _ => warn!("Unknown settings section: {}", section),
        }
    }
}

/// Settings profile
/// 設定プロファイル
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Profile name / プロファイル名
    pub name: String,
    /// Profile description / プロファイル説明
    pub description: String,
    /// Settings for this profile / このプロファイルの設定
    pub settings: AppSettings,
    /// Whether this is the default profile / デフォルトプロファイルかどうか
    pub is_default: bool,
}

impl Profile {
    /// Create a new profile with default settings
    /// デフォルト設定で新しいプロファイルを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            settings: AppSettings::default(),
            is_default: false,
        }
    }

    /// Create a profile from existing settings
    /// 既存の設定からプロファイルを作成
    pub fn from_settings(name: &str, settings: AppSettings) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            settings,
            is_default: false,
        }
    }
}

/// Profile manager for handling multiple settings profiles
/// 複数の設定プロファイルを管理するプロファイルマネージャー
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileManager {
    /// All available profiles / 利用可能なすべてのプロファイル
    pub profiles: HashMap<String, Profile>,
    /// Currently active profile name / 現在アクティブなプロファイル名
    pub active_profile: String,
}

impl Default for ProfileManager {
    fn default() -> Self {
        let mut profiles = HashMap::new();
        let mut default_profile = Profile::new("Default");
        default_profile.description = "Default settings profile".to_string();
        default_profile.is_default = true;
        profiles.insert("Default".to_string(), default_profile);

        Self {
            profiles,
            active_profile: "Default".to_string(),
        }
    }
}

impl ProfileManager {
    /// Create a new profile manager with default profile
    /// デフォルトプロファイル付きの新しいプロファイルマネージャーを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the active profile
    /// アクティブなプロファイルを取得
    pub fn active(&self) -> Option<&Profile> {
        self.profiles.get(&self.active_profile)
    }

    /// Get mutable reference to active profile
    /// アクティブなプロファイルへの可変参照を取得
    pub fn active_mut(&mut self) -> Option<&mut Profile> {
        self.profiles.get_mut(&self.active_profile)
    }

    /// Get active settings (returns owned copy, use active() for reference)
    /// アクティブな設定を取得（所有権コピーを返します。参照はactive()を使用）
    pub fn active_settings(&self) -> AppSettings {
        self.active()
            .map(|p| p.settings.clone())
            .unwrap_or_default()
    }

    /// Switch to a different profile
    /// 別のプロファイルに切り替え
    pub fn switch_profile(&mut self, name: &str) -> Result<()> {
        if self.profiles.contains_key(name) {
            info!("Switching to profile: {}", name);
            self.active_profile = name.to_string();
            Ok(())
        } else {
            Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Profile not found: {}", name),
            )))
        }
    }

    /// Create a new profile
    /// 新しいプロファイルを作成
    pub fn create_profile(&mut self, name: &str) -> Result<()> {
        if self.profiles.contains_key(name) {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!("Profile already exists: {}", name),
            )));
        }

        info!("Creating new profile: {}", name);
        let profile = Profile::new(name);
        self.profiles.insert(name.to_string(), profile);
        Ok(())
    }

    /// Delete a profile
    /// プロファイルを削除
    pub fn delete_profile(&mut self, name: &str) -> Result<()> {
        if let Some(profile) = self.profiles.get(name) {
            if profile.is_default {
                return Err(SikulixError::IoError(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "Cannot delete the default profile",
                )));
            }
        }

        if name == self.active_profile {
            // Switch to default profile first
            self.active_profile = "Default".to_string();
        }

        info!("Deleting profile: {}", name);
        self.profiles.remove(name);
        Ok(())
    }

    /// Duplicate a profile
    /// プロファイルを複製
    pub fn duplicate_profile(&mut self, source: &str, new_name: &str) -> Result<()> {
        let source_profile = self.profiles.get(source).ok_or_else(|| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source profile not found: {}", source),
            ))
        })?;

        let mut new_profile = source_profile.clone();
        new_profile.name = new_name.to_string();
        new_profile.is_default = false;

        info!("Duplicating profile {} as {}", source, new_name);
        self.profiles.insert(new_name.to_string(), new_profile);
        Ok(())
    }

    /// Export a profile to JSON string
    /// プロファイルをJSON文字列にエクスポート
    pub fn export_profile(&self, name: &str) -> Result<String> {
        let profile = self.profiles.get(name).ok_or_else(|| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Profile not found: {}", name),
            ))
        })?;

        serde_json::to_string_pretty(profile).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to export profile: {}", e),
            ))
        })
    }

    /// Import a profile from JSON string
    /// JSON文字列からプロファイルをインポート
    pub fn import_profile(&mut self, json: &str) -> Result<()> {
        let profile: Profile = serde_json::from_str(json).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse profile: {}", e),
            ))
        })?;

        info!("Importing profile: {}", profile.name);
        self.profiles.insert(profile.name.clone(), profile);
        Ok(())
    }

    /// Get list of all profile names
    /// すべてのプロファイル名のリストを取得
    pub fn list_profiles(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }

    /// Save profiles to default location
    /// プロファイルをデフォルト場所に保存
    pub fn save(&self) -> Result<()> {
        let path = Self::default_path();
        info!("Saving profiles to: {}", path.display());

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize profiles: {}", e),
            ))
        })?;

        fs::write(path, content)?;
        Ok(())
    }

    /// Load profiles from default location
    /// デフォルト場所からプロファイルを読み込み
    pub fn load() -> Result<Self> {
        let path = Self::default_path();
        info!("Loading profiles from: {}", path.display());

        let content = fs::read_to_string(&path)?;
        let manager: Self = serde_json::from_str(&content).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse profiles: {}", e),
            ))
        })?;

        Ok(manager)
    }

    /// Load profiles or create default
    /// プロファイルを読み込むか、デフォルトを作成
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    /// Get default profiles file path
    /// デフォルトプロファイルファイルパスを取得
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("sikulix")
            .join("profiles.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = AppSettings::default();
        assert_eq!(settings.general.theme, Theme::Dark);
        assert_eq!(settings.editor.font_size, 14);
        assert!(settings.execution.auto_save_before_run);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = AppSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let loaded: AppSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.general.theme, settings.general.theme);
    }

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("Test");
        assert_eq!(profile.name, "Test");
        assert!(!profile.is_default);
    }

    #[test]
    fn test_profile_manager() {
        let mut manager = ProfileManager::new();
        assert!(manager.active().is_some());
        assert!(manager.create_profile("Custom").is_ok());
        assert!(manager.switch_profile("Custom").is_ok());
        assert_eq!(manager.active_profile, "Custom");
    }

    #[test]
    fn test_profile_duplicate() {
        let mut manager = ProfileManager::new();
        manager.duplicate_profile("Default", "Copy").unwrap();
        assert!(manager.profiles.contains_key("Copy"));
    }

    #[test]
    fn test_hotkey_defaults() {
        let settings = HotkeySettings::default();
        assert!(!settings.bindings.is_empty());

        let run_key = settings.bindings.iter().find(|b| b.action == "run_script");
        assert!(run_key.is_some());
        assert_eq!(run_key.unwrap().key, "F5");
    }

    #[test]
    fn test_hotkey_conflict_detection() {
        let mut settings = HotkeySettings::default();

        // Add a conflicting binding (same key as F5)
        settings.bindings.push(HotkeyBinding {
            key: "F5".to_string(),
            action: "debug_script".to_string(),
            enabled: true,
        });

        let conflicts = settings.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].key, "F5");
        assert!(conflicts[0].actions.contains(&"run_script".to_string()));
        assert!(conflicts[0].actions.contains(&"debug_script".to_string()));
    }

    #[test]
    fn test_hotkey_no_conflict_when_disabled() {
        let mut settings = HotkeySettings::default();

        // Add a binding with same key but disabled
        settings.bindings.push(HotkeyBinding {
            key: "F5".to_string(),
            action: "debug_script".to_string(),
            enabled: false,
        });

        let conflicts = settings.detect_conflicts();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_hotkey_normalize_key() {
        let mut settings = HotkeySettings::default();

        // Test that Ctrl+S and ctrl + s are normalized to the same key
        settings.bindings.push(HotkeyBinding {
            key: "ctrl + s".to_string(),
            action: "save_as".to_string(),
            enabled: true,
        });

        let conflicts = settings.detect_conflicts();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].actions.contains(&"save_file".to_string()));
        assert!(conflicts[0].actions.contains(&"save_as".to_string()));
    }

    #[test]
    fn test_hotkey_set_and_remove() {
        let mut settings = HotkeySettings::default();

        // Add a new hotkey
        settings
            .set_hotkey("custom_action", "Ctrl+Shift+X")
            .unwrap();
        assert!(settings.get_hotkey("custom_action").is_some());
        assert_eq!(
            settings.get_hotkey("custom_action").unwrap().key,
            "Ctrl+Shift+X"
        );

        // Update existing hotkey
        settings.set_hotkey("custom_action", "Ctrl+Alt+X").unwrap();
        assert_eq!(
            settings.get_hotkey("custom_action").unwrap().key,
            "Ctrl+Alt+X"
        );

        // Remove hotkey
        assert!(settings.remove_hotkey("custom_action"));
        assert!(settings.get_hotkey("custom_action").is_none());
    }

    #[test]
    fn test_hotkey_enable_disable() {
        let mut settings = HotkeySettings::default();

        // Disable a hotkey
        assert!(settings.set_enabled("run_script", false));
        assert!(!settings.get_hotkey("run_script").unwrap().enabled);

        // Enable it again
        assert!(settings.set_enabled("run_script", true));
        assert!(settings.get_hotkey("run_script").unwrap().enabled);
    }

    #[test]
    fn test_hotkey_system_conflict() {
        let settings = HotkeySettings::default();

        // Check Alt+F4 (system hotkey)
        let conflict = settings.conflicts_with_system("Alt+F4");
        assert!(conflict.is_some());
        assert_eq!(conflict.unwrap().description, "Close window");

        // Check F5 (not a system hotkey)
        assert!(settings.conflicts_with_system("F5").is_none());
    }

    #[test]
    fn test_hotkey_get_action() {
        let settings = HotkeySettings::default();

        // Find action by key
        assert_eq!(settings.get_action("F5"), Some("run_script"));
        assert_eq!(settings.get_action("Ctrl+S"), Some("save_file"));
        assert_eq!(settings.get_action("ctrl+s"), Some("save_file")); // normalized

        // Non-existent key
        assert!(settings.get_action("Ctrl+Q").is_none());
    }

    #[test]
    fn test_hotkey_validate() {
        let mut settings = HotkeySettings::default();

        // Add a conflict
        settings.bindings.push(HotkeyBinding {
            key: "F5".to_string(),
            action: "debug_script".to_string(),
            enabled: true,
        });

        // Add a system conflict
        settings.bindings.push(HotkeyBinding {
            key: "Alt+F4".to_string(),
            action: "close_app".to_string(),
            enabled: true,
        });

        let issues = settings.validate();
        assert!(issues.len() >= 2);
        assert!(issues.iter().any(|i| i.contains("F5")));
        assert!(issues.iter().any(|i| i.contains("Alt+F4")));
    }
}
