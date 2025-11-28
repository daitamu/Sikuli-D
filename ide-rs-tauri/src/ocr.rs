//! OCR (Optical Character Recognition) Integration Module
//! OCR（光学文字認識）統合モジュール
//!
//! This module provides OCR functionality by calling the sikulix CLI.
//! このモジュールはsikulix CLIを呼び出すことでOCR機能を提供します。

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::Mutex;

// ============================================================================
// Data Types / データ型
// ============================================================================

/// Region coordinates for OCR
/// OCR用の領域座標
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionDto {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// OCR recognition options
/// OCR認識オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrOptions {
    /// OCR language (e.g., "eng", "jpn")
    /// OCR言語（例："eng", "jpn"）
    #[serde(default = "default_language")]
    pub language: String,

    /// Minimum confidence threshold (0.0-1.0)
    /// 最小信頼度閾値（0.0-1.0）
    #[serde(default = "default_confidence")]
    pub min_confidence: f64,

    /// Page Segmentation Mode (PSM)
    /// ページ分割モード（PSM）
    #[serde(default)]
    pub psm: Option<u32>,
}

fn default_language() -> String {
    "eng".to_string()
}

fn default_confidence() -> f64 {
    0.7
}

impl Default for OcrOptions {
    fn default() -> Self {
        Self {
            language: default_language(),
            min_confidence: default_confidence(),
            psm: None,
        }
    }
}

/// OCR recognition result
/// OCR認識結果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// Recognized text
    /// 認識されたテキスト
    pub text: String,

    /// Overall confidence score (0.0-1.0)
    /// 全体の信頼度スコア（0.0-1.0）
    pub confidence: f64,

    /// Individual words with their bounding boxes
    /// 境界ボックス付きの個別単語
    pub words: Vec<OcrWord>,
}

/// Single word in OCR result with bounding box
/// 境界ボックス付きのOCR結果内の単一単語
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrWord {
    /// Word text
    /// 単語テキスト
    pub text: String,

    /// Word region
    /// 単語領域
    pub region: RegionDto,

    /// Word confidence (0.0-1.0)
    /// 単語信頼度（0.0-1.0）
    pub confidence: f64,
}

/// OCR language information
/// OCR言語情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrLanguage {
    /// Language code (e.g., "eng", "jpn")
    /// 言語コード（例："eng", "jpn"）
    pub code: String,

    /// Display name
    /// 表示名
    pub name: String,

    /// Whether it's installed
    /// インストール済みかどうか
    pub installed: bool,
}

// ============================================================================
// State Management / 状態管理
// ============================================================================

/// OCR state for managing settings
/// 設定管理用のOCR状態
#[derive(Default)]
pub struct OcrState {
    /// Current OCR language
    /// 現在のOCR言語
    current_language: Mutex<String>,

    /// Available languages cache
    /// 利用可能な言語のキャッシュ
    #[allow(dead_code)]
    languages_cache: Mutex<Option<Vec<OcrLanguage>>>,
}

impl OcrState {
    pub fn new() -> Self {
        Self {
            current_language: Mutex::new("eng".to_string()),
            languages_cache: Mutex::new(None),
        }
    }
}

// ============================================================================
// Helper Functions / ヘルパー関数
// ============================================================================

/// Check if sikulix CLI is available
/// sikulix CLIが利用可能か確認
fn check_sikulix_available() -> Result<bool, String> {
    debug!("Checking sikulix CLI availability");

    match Command::new("sikulix")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) => {
            let available = status.success();
            debug!("sikulix CLI available: {}", available);
            Ok(available)
        }
        Err(e) => {
            warn!("Failed to check sikulix CLI: {}", e);
            Ok(false)
        }
    }
}

/// Parse OCR output from sikulix CLI
/// sikulix CLIからのOCR出力を解析
fn parse_ocr_output(output: &str) -> Result<OcrResult, String> {
    debug!("Parsing OCR output: {} bytes", output.len());

    // Try to parse as JSON first
    // まずJSONとして解析を試行
    if let Ok(result) = serde_json::from_str::<OcrResult>(output) {
        debug!("Parsed JSON OCR result: {} words", result.words.len());
        return Ok(result);
    }

    // Fallback: treat as plain text
    // フォールバック：プレーンテキストとして扱う
    warn!("Could not parse as JSON, treating as plain text");
    Ok(OcrResult {
        text: output.trim().to_string(),
        confidence: 0.0,
        words: vec![],
    })
}

/// Get known OCR languages
/// 既知のOCR言語を取得
fn get_known_languages() -> Vec<OcrLanguage> {
    vec![
        OcrLanguage {
            code: "eng".to_string(),
            name: "English".to_string(),
            installed: true,
        },
        OcrLanguage {
            code: "jpn".to_string(),
            name: "Japanese".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "chi_sim".to_string(),
            name: "Chinese (Simplified)".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "chi_tra".to_string(),
            name: "Chinese (Traditional)".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "kor".to_string(),
            name: "Korean".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "fra".to_string(),
            name: "French".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "deu".to_string(),
            name: "German".to_string(),
            installed: false,
        },
        OcrLanguage {
            code: "spa".to_string(),
            name: "Spanish".to_string(),
            installed: false,
        },
    ]
}

// ============================================================================
// Tauri Commands / Tauriコマンド
// ============================================================================

/// Perform OCR recognition on a screen region
/// 画面領域でOCR認識を実行
#[tauri::command]
pub async fn ocr_recognize(region: RegionDto) -> Result<OcrResult, String> {
    info!(
        "OCR recognize requested: region ({}, {}, {}, {})",
        region.x, region.y, region.width, region.height
    );

    // Use default options
    // デフォルトオプションを使用
    ocr_recognize_with_options(region, OcrOptions::default()).await
}

/// Perform OCR recognition with custom options
/// カスタムオプション付きでOCR認識を実行
#[tauri::command]
pub async fn ocr_recognize_with_options(
    region: RegionDto,
    options: OcrOptions,
) -> Result<OcrResult, String> {
    info!(
        "OCR recognize with options: region ({}, {}, {}, {}), lang: {}, confidence: {}",
        region.x, region.y, region.width, region.height, options.language, options.min_confidence
    );

    // Check if sikulix is available
    // sikulixが利用可能か確認
    if !check_sikulix_available()? {
        error!("sikulix CLI not available");
        return Err("sikulix CLI not found. Please ensure it's installed and in PATH.".to_string());
    }

    // Build sikulix command
    // sikulixコマンドを構築
    let mut cmd = Command::new("sikulix");
    cmd.arg("ocr")
        .arg("--region")
        .arg(format!(
            "{},{},{},{}",
            region.x, region.y, region.width, region.height
        ))
        .arg("--language")
        .arg(&options.language)
        .arg("--min-confidence")
        .arg(options.min_confidence.to_string())
        .arg("--output")
        .arg("json");

    // Add PSM if specified
    // PSMが指定されている場合は追加
    if let Some(psm) = options.psm {
        cmd.arg("--psm").arg(psm.to_string());
    }

    debug!("Executing OCR command: {:?}", cmd);

    // Execute command
    // コマンドを実行
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug!("OCR command succeeded: {} bytes output", stdout.len());

                match parse_ocr_output(&stdout) {
                    Ok(result) => {
                        info!(
                            "OCR completed: {} characters, confidence: {:.2}",
                            result.text.len(),
                            result.confidence
                        );
                        Ok(result)
                    }
                    Err(e) => {
                        error!("Failed to parse OCR output: {}", e);
                        Err(format!("Failed to parse OCR output: {}", e))
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                error!("OCR command failed: {}", stderr);
                Err(format!("OCR command failed: {}", stderr))
            }
        }
        Err(e) => {
            error!("Failed to execute OCR command: {}", e);
            Err(format!("Failed to execute OCR command: {}", e))
        }
    }
}

/// Get list of available OCR languages
/// 利用可能なOCR言語のリストを取得
#[tauri::command]
pub fn get_available_languages() -> Vec<OcrLanguage> {
    debug!("Get available OCR languages requested");

    // TODO: Query actual installed languages from Tesseract
    // TODO: Tesseractから実際にインストールされている言語を問い合わせ
    let languages = get_known_languages();

    info!("Returning {} known languages", languages.len());
    languages
}

/// Set current OCR language
/// 現在のOCR言語を設定
#[tauri::command]
pub fn set_ocr_language(lang: String, state: tauri::State<OcrState>) -> Result<(), String> {
    info!("Setting OCR language to: {}", lang);

    // Validate language code
    // 言語コードを検証
    let known_langs = get_known_languages();
    if !known_langs.iter().any(|l| l.code == lang) {
        warn!("Unknown language code: {}", lang);
        return Err(format!("Unknown language code: {}", lang));
    }

    // Update state
    // 状態を更新
    match state.current_language.lock() {
        Ok(mut current) => {
            *current = lang.clone();
            info!("OCR language set to: {}", lang);
            Ok(())
        }
        Err(e) => {
            error!("Failed to lock current_language: {}", e);
            Err("Failed to update language".to_string())
        }
    }
}

/// Get current OCR language
/// 現在のOCR言語を取得
#[tauri::command]
pub fn get_ocr_language(state: tauri::State<OcrState>) -> String {
    debug!("Get current OCR language requested");

    match state.current_language.lock() {
        Ok(current) => {
            let lang = current.clone();
            debug!("Current OCR language: {}", lang);
            lang
        }
        Err(e) => {
            error!("Failed to lock current_language: {}", e);
            "eng".to_string()
        }
    }
}

/// Check OCR availability
/// OCR可用性を確認
#[tauri::command]
pub fn check_ocr_available() -> bool {
    debug!("Check OCR availability requested");

    match check_sikulix_available() {
        Ok(available) => {
            info!("OCR available: {}", available);
            available
        }
        Err(e) => {
            error!("Error checking OCR availability: {}", e);
            false
        }
    }
}

/// Get OCR engine information
/// OCRエンジン情報を取得
#[tauri::command]
pub fn get_ocr_info() -> HashMap<String, String> {
    debug!("Get OCR info requested");

    let mut info = HashMap::new();
    info.insert("engine".to_string(), "Tesseract".to_string());
    info.insert("version".to_string(), "5.x".to_string());
    info.insert("provider".to_string(), "sikulix CLI".to_string());

    debug!("Returning OCR info: {:?}", info);
    info
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_dto_serialization() {
        let region = RegionDto {
            x: 100,
            y: 200,
            width: 300,
            height: 400,
        };

        let json = serde_json::to_string(&region).unwrap();
        assert!(json.contains("\"x\":100"));
        assert!(json.contains("\"y\":200"));
        assert!(json.contains("\"width\":300"));
        assert!(json.contains("\"height\":400"));
    }

    #[test]
    fn test_ocr_options_default() {
        let options = OcrOptions::default();
        assert_eq!(options.language, "eng");
        assert_eq!(options.min_confidence, 0.7);
        assert!(options.psm.is_none());
    }

    #[test]
    fn test_ocr_result_serialization() {
        let result = OcrResult {
            text: "Hello World".to_string(),
            confidence: 0.95,
            words: vec![
                OcrWord {
                    text: "Hello".to_string(),
                    region: RegionDto {
                        x: 0,
                        y: 0,
                        width: 50,
                        height: 20,
                    },
                    confidence: 0.96,
                },
                OcrWord {
                    text: "World".to_string(),
                    region: RegionDto {
                        x: 60,
                        y: 0,
                        width: 50,
                        height: 20,
                    },
                    confidence: 0.94,
                },
            ],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("Hello World"));
        assert!(json.contains("0.95"));
    }

    #[test]
    fn test_parse_ocr_output_json() {
        let json_output = r#"{
            "text": "Test",
            "confidence": 0.9,
            "words": [
                {
                    "text": "Test",
                    "region": {"x": 0, "y": 0, "width": 100, "height": 30},
                    "confidence": 0.9
                }
            ]
        }"#;

        let result = parse_ocr_output(json_output).unwrap();
        assert_eq!(result.text, "Test");
        assert_eq!(result.confidence, 0.9);
        assert_eq!(result.words.len(), 1);
    }

    #[test]
    fn test_parse_ocr_output_plain_text() {
        let plain_output = "Plain text output";
        let result = parse_ocr_output(plain_output).unwrap();
        assert_eq!(result.text, "Plain text output");
        assert_eq!(result.confidence, 0.0);
        assert_eq!(result.words.len(), 0);
    }

    #[test]
    fn test_get_known_languages() {
        let languages = get_known_languages();
        assert!(!languages.is_empty());

        // Check English is present
        assert!(languages.iter().any(|l| l.code == "eng"));

        // Check Japanese is present
        assert!(languages.iter().any(|l| l.code == "jpn"));
    }

    #[test]
    fn test_ocr_state_default() {
        let state = OcrState::new();
        let current = state.current_language.lock().unwrap();
        assert_eq!(*current, "eng");
    }
}
