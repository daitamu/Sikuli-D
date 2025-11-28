//! Pattern Editor Commands / パターンエディタコマンド
//!
//! Provides Tauri commands for pattern (image template) editing functionality.
//! パターン（画像テンプレート）編集機能のTauriコマンドを提供します。

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use sikulid::{Pattern, Screen};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

// ============================================================================
// Pattern State / パターン状態
// ============================================================================

/// State for pattern editor operations
/// パターンエディタ操作の状態
#[derive(Default)]
pub struct PatternState {
    /// Current pattern being edited / 現在編集中のパターン
    pub current_pattern: Mutex<Option<PatternConfig>>,
    /// Last test result / 最後のテスト結果
    pub last_test_result: Mutex<Option<TestResult>>,
}

// ============================================================================
// Data Structures / データ構造
// ============================================================================

/// Pattern configuration for image template
/// 画像テンプレートのパターン設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternConfig {
    /// Path to the image file / 画像ファイルのパス
    pub image_path: String,

    /// Similarity threshold (0.0 - 1.0) / 類似度閾値（0.0 - 1.0）
    pub similarity: f64,

    /// Target offset for click position (x, y) / クリック位置のターゲットオフセット（x, y）
    pub target_offset: (i32, i32),

    /// Optional mask image path / オプションのマスク画像パス
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_path: Option<String>,
}

/// Result of pattern test on screen
/// 画面上でのパターンテスト結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// Whether pattern was found / パターンが見つかったか
    pub found: bool,

    /// Message describing the result / 結果を説明するメッセージ
    pub message: String,

    /// Location where pattern was found (if any) / パターンが見つかった場所（あれば）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationInfo>,

    /// Match confidence score / マッチ信頼度スコア
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// Time taken for search in milliseconds / 検索にかかった時間（ミリ秒）
    pub search_time_ms: u64,
}

/// Location information for found pattern
/// 見つかったパターンの位置情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    /// X coordinate / X座標
    pub x: i32,

    /// Y coordinate / Y座標
    pub y: i32,

    /// Width / 幅
    pub width: u32,

    /// Height / 高さ
    pub height: u32,

    /// Target click position (with offset applied) / ターゲットクリック位置（オフセット適用後）
    pub target_x: i32,

    /// Target click position (with offset applied) / ターゲットクリック位置（オフセット適用後）
    pub target_y: i32,
}

/// Result of pattern operation
/// パターン操作の結果
#[derive(Debug, Serialize)]
pub struct PatternOperationResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<PatternConfig>,
}

// ============================================================================
// Pattern Editor Commands / パターンエディタコマンド
// ============================================================================

/// Create a new pattern configuration
/// 新しいパターン設定を作成
#[tauri::command]
pub fn create_pattern(
    image_path: String,
    state: State<'_, PatternState>,
) -> Result<PatternOperationResult, String> {
    info!("Creating new pattern: {}", image_path);

    // Validate image path exists
    let path = PathBuf::from(&image_path);
    if !path.exists() {
        error!("Image file not found: {}", image_path);
        return Ok(PatternOperationResult {
            success: false,
            message: format!("Image file not found: {}", image_path),
            pattern: None,
        });
    }

    if !path.is_file() {
        error!("Path is not a file: {}", image_path);
        return Ok(PatternOperationResult {
            success: false,
            message: format!("Path is not a file: {}", image_path),
            pattern: None,
        });
    }

    // Create default pattern configuration
    let pattern = PatternConfig {
        image_path: image_path.clone(),
        similarity: 0.8,       // Default similarity
        target_offset: (0, 0), // No offset by default
        mask_path: None,
    };

    // Store in state
    if let Ok(mut current) = state.current_pattern.lock() {
        *current = Some(pattern.clone());
    }

    info!("Pattern created successfully");

    Ok(PatternOperationResult {
        success: true,
        message: format!(
            "Pattern created from {}",
            path.file_name().unwrap_or_default().to_string_lossy()
        ),
        pattern: Some(pattern),
    })
}

/// Update pattern configuration
/// パターン設定を更新
#[tauri::command]
pub fn update_pattern(config: PatternConfig, state: State<'_, PatternState>) -> Result<(), String> {
    info!("Updating pattern: {}", config.image_path);

    // Validate similarity is in valid range
    if config.similarity < 0.0 || config.similarity > 1.0 {
        error!("Invalid similarity value: {}", config.similarity);
        return Err(format!(
            "Similarity must be between 0.0 and 1.0, got: {}",
            config.similarity
        ));
    }

    // Validate image path exists
    let path = PathBuf::from(&config.image_path);
    if !path.exists() {
        error!("Image file not found: {}", config.image_path);
        return Err(format!("Image file not found: {}", config.image_path));
    }

    // Validate mask path if provided
    if let Some(ref mask_path) = config.mask_path {
        let mask = PathBuf::from(mask_path);
        if !mask.exists() {
            error!("Mask file not found: {}", mask_path);
            return Err(format!("Mask file not found: {}", mask_path));
        }
    }

    // Update state
    if let Ok(mut current) = state.current_pattern.lock() {
        *current = Some(config.clone());
    }

    debug!("Pattern updated successfully");

    Ok(())
}

/// Test pattern on current screen
/// 現在の画面でパターンをテスト
#[tauri::command]
pub async fn test_pattern(
    config: PatternConfig,
    state: State<'_, PatternState>,
) -> Result<TestResult, String> {
    info!("Testing pattern: {}", config.image_path);

    let start = std::time::Instant::now();

    // Create pattern from config
    let mut pattern = Pattern::from_file(&config.image_path)
        .map_err(|e| format!("Failed to load pattern image: {}", e))?;

    // Apply similarity setting
    pattern = pattern.similar(config.similarity);

    // Apply target offset
    if config.target_offset != (0, 0) {
        pattern = pattern.target_offset(config.target_offset.0, config.target_offset.1);
    }

    // Get primary screen
    let screen = Screen::primary();

    // Try to find the pattern
    let result = match screen.find(&pattern) {
        Ok(match_result) => {
            let search_time_ms = start.elapsed().as_millis() as u64;

            // Calculate target position with offset
            let target_x = match_result.region.x + config.target_offset.0;
            let target_y = match_result.region.y + config.target_offset.1;

            let test_result = TestResult {
                found: true,
                message: format!(
                    "Pattern found at ({}, {})",
                    match_result.region.x, match_result.region.y
                ),
                location: Some(LocationInfo {
                    x: match_result.region.x,
                    y: match_result.region.y,
                    width: match_result.region.width,
                    height: match_result.region.height,
                    target_x,
                    target_y,
                }),
                confidence: Some(match_result.score),
                search_time_ms,
            };

            info!("Pattern found: {:?}", test_result.location);
            test_result
        }
        Err(e) => {
            let search_time_ms = start.elapsed().as_millis() as u64;

            warn!("Pattern not found: {}", e);

            TestResult {
                found: false,
                message: format!(
                    "Pattern not found on screen (searched for {}ms)",
                    search_time_ms
                ),
                location: None,
                confidence: None,
                search_time_ms,
            }
        }
    };

    // Store test result
    if let Ok(mut last_result) = state.last_test_result.lock() {
        *last_result = Some(result.clone());
    }

    Ok(result)
}

/// Preview target offset position
/// ターゲットオフセット位置をプレビュー
#[tauri::command]
pub fn preview_target_offset(config: PatternConfig) -> Result<String, String> {
    info!("Previewing target offset for: {}", config.image_path);

    // Return information about the offset
    let offset_info = if config.target_offset == (0, 0) {
        "Click will occur at the center of the found pattern".to_string()
    } else {
        format!(
            "Click will occur at offset ({}, {}) from pattern center",
            config.target_offset.0, config.target_offset.1
        )
    };

    debug!("Offset preview: {}", offset_info);

    Ok(offset_info)
}

/// Generate Python code from pattern configuration
/// パターン設定からPythonコードを生成
#[tauri::command]
pub fn generate_pattern_code(config: PatternConfig) -> Result<String, String> {
    info!("Generating Python code for pattern: {}", config.image_path);

    let path = PathBuf::from(&config.image_path);
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| "Invalid image filename".to_string())?;

    // Build pattern code
    let mut code = format!("Pattern(\"{}\")", filename);

    // Add similarity if not default
    if (config.similarity - 0.8).abs() > 0.001 {
        code.push_str(&format!(".similar({:.2})", config.similarity));
    }

    // Add target offset if not (0, 0)
    if config.target_offset != (0, 0) {
        code.push_str(&format!(
            ".targetOffset({}, {})",
            config.target_offset.0, config.target_offset.1
        ));
    }

    debug!("Generated code: {}", code);

    Ok(code)
}

/// Get current pattern configuration
/// 現在のパターン設定を取得
#[tauri::command]
pub fn get_current_pattern(state: State<'_, PatternState>) -> Option<PatternConfig> {
    state
        .current_pattern
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Get last test result
/// 最後のテスト結果を取得
#[tauri::command]
pub fn get_last_test_result(state: State<'_, PatternState>) -> Option<TestResult> {
    state
        .last_test_result
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

/// Clear current pattern
/// 現在のパターンをクリア
#[tauri::command]
pub fn clear_pattern(state: State<'_, PatternState>) {
    info!("Clearing current pattern");

    if let Ok(mut current) = state.current_pattern.lock() {
        *current = None;
    }

    if let Ok(mut last_result) = state.last_test_result.lock() {
        *last_result = None;
    }
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_config_default() {
        let config = PatternConfig {
            image_path: "test.png".to_string(),
            similarity: 0.8,
            target_offset: (0, 0),
            mask_path: None,
        };

        assert_eq!(config.similarity, 0.8);
        assert_eq!(config.target_offset, (0, 0));
        assert!(config.mask_path.is_none());
    }

    #[test]
    fn test_pattern_config_serialization() {
        let config = PatternConfig {
            image_path: "test.png".to_string(),
            similarity: 0.75,
            target_offset: (10, -5),
            mask_path: Some("mask.png".to_string()),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("0.75"));
        assert!(json.contains("mask.png"));

        let parsed: PatternConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.similarity, 0.75);
        assert_eq!(parsed.target_offset, (10, -5));
    }

    #[test]
    fn test_test_result_serialization() {
        let result = TestResult {
            found: true,
            message: "Found".to_string(),
            location: Some(LocationInfo {
                x: 100,
                y: 200,
                width: 50,
                height: 30,
                target_x: 110,
                target_y: 195,
            }),
            confidence: Some(0.95),
            search_time_ms: 150,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"found\":true"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_generate_pattern_code_default() {
        let config = PatternConfig {
            image_path: "button.png".to_string(),
            similarity: 0.8,
            target_offset: (0, 0),
            mask_path: None,
        };

        let code = generate_pattern_code(config).unwrap();
        assert_eq!(code, "Pattern(\"button.png\")");
    }

    #[test]
    fn test_generate_pattern_code_with_similarity() {
        let config = PatternConfig {
            image_path: "button.png".to_string(),
            similarity: 0.75,
            target_offset: (0, 0),
            mask_path: None,
        };

        let code = generate_pattern_code(config).unwrap();
        assert_eq!(code, "Pattern(\"button.png\").similar(0.75)");
    }

    #[test]
    fn test_generate_pattern_code_with_offset() {
        let config = PatternConfig {
            image_path: "button.png".to_string(),
            similarity: 0.8,
            target_offset: (10, -5),
            mask_path: None,
        };

        let code = generate_pattern_code(config).unwrap();
        assert_eq!(code, "Pattern(\"button.png\").targetOffset(10, -5)");
    }

    #[test]
    fn test_generate_pattern_code_full() {
        let config = PatternConfig {
            image_path: "icon.png".to_string(),
            similarity: 0.9,
            target_offset: (20, 15),
            mask_path: None,
        };

        let code = generate_pattern_code(config).unwrap();
        assert_eq!(
            code,
            "Pattern(\"icon.png\").similar(0.90).targetOffset(20, 15)"
        );
    }
}
