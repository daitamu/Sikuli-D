//! Screen Capture Commands / 画面キャプチャコマンド
//!
//! Provides Tauri commands for screen capture functionality.
//! 画面キャプチャ機能のTauriコマンドを提供します。

use image::ImageFormat;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sikulid_core::{Region, Screen};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder};

// ============================================================================
// Capture State / キャプチャ状態
// ============================================================================

/// State for screen capture operations
/// 画面キャプチャ操作の状態
#[derive(Default)]
pub struct CaptureState {
    /// Path to the last captured image / 最後にキャプチャした画像のパス
    pub last_capture_path: Mutex<Option<PathBuf>>,
    /// Whether capture mode is active / キャプチャモードがアクティブかどうか
    pub capture_active: Mutex<bool>,
}

/// Result of a capture operation
/// キャプチャ操作の結果
#[derive(Serialize, Deserialize)]
pub struct CaptureResult {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Region coordinates from frontend
/// フロントエンドからの領域座標
#[derive(Deserialize)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// ============================================================================
// Capture Commands / キャプチャコマンド
// ============================================================================

/// Start capture overlay window
/// キャプチャオーバーレイウィンドウを開始
#[tauri::command]
pub async fn start_capture(app: AppHandle, state: State<'_, CaptureState>) -> Result<(), String> {
    info!("Starting capture overlay");

    // Check if capture is already active
    if let Ok(active) = state.capture_active.lock() {
        if *active {
            return Err("Capture is already active".to_string());
        }
    }

    // Mark capture as active
    if let Ok(mut active) = state.capture_active.lock() {
        *active = true;
    }

    // Create capture overlay window
    let _capture_window = WebviewWindowBuilder::new(
        &app,
        "capture-overlay",
        WebviewUrl::App("capture.html".into()),
    )
    .title("Screen Capture")
    .fullscreen(true)
    .transparent(true)
    .always_on_top(true)
    .decorations(false)
    .skip_taskbar(true)
    .build()
    .map_err(|e| {
        error!("Failed to create capture window: {}", e);
        if let Ok(mut active) = state.capture_active.lock() {
            *active = false;
        }
        format!("Failed to create capture window: {}", e)
    })?;

    debug!("Capture overlay window created");

    Ok(())
}

/// Capture a specific region of the screen
/// 画面の特定領域をキャプチャ
#[tauri::command]
pub async fn capture_region(
    app: AppHandle,
    region: CaptureRegion,
    state: State<'_, CaptureState>,
) -> Result<CaptureResult, String> {
    info!(
        "Capturing region: x={}, y={}, w={}, h={}",
        region.x, region.y, region.width, region.height
    );

    // Close capture overlay first
    if let Some(window) = app.get_webview_window("capture-overlay") {
        let _ = window.close();
    }

    // Mark capture as inactive
    if let Ok(mut active) = state.capture_active.lock() {
        *active = false;
    }

    // Small delay to ensure overlay is closed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Capture the region
    let screen = Screen::primary();
    let capture_region = Region::new(region.x, region.y, region.width, region.height);

    match screen.capture_region(&capture_region) {
        Ok(image) => {
            // Generate unique filename
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("capture_{}.png", timestamp);

            // Get captures directory
            let captures_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("captures");

            // Create directory if needed
            if !captures_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&captures_dir) {
                    error!("Failed to create captures directory: {}", e);
                    return Ok(CaptureResult {
                        success: false,
                        message: format!("Failed to create captures directory: {}", e),
                        path: None,
                    });
                }
            }

            let capture_path = captures_dir.join(&filename);

            // Save image
            match image.save_with_format(&capture_path, ImageFormat::Png) {
                Ok(_) => {
                    info!("Capture saved: {:?}", capture_path);

                    // Update last capture path
                    if let Ok(mut last_path) = state.last_capture_path.lock() {
                        *last_path = Some(capture_path.clone());
                    }

                    Ok(CaptureResult {
                        success: true,
                        message: format!("Captured region saved to {}", filename),
                        path: Some(capture_path.to_string_lossy().to_string()),
                    })
                }
                Err(e) => {
                    error!("Failed to save capture: {}", e);
                    Ok(CaptureResult {
                        success: false,
                        message: format!("Failed to save capture: {}", e),
                        path: None,
                    })
                }
            }
        }
        Err(e) => {
            error!("Failed to capture region: {}", e);
            Ok(CaptureResult {
                success: false,
                message: format!("Failed to capture region: {}", e),
                path: None,
            })
        }
    }
}

/// Capture the full screen
/// 画面全体をキャプチャ
#[tauri::command]
pub async fn capture_full_screen(
    app: AppHandle,
    state: State<'_, CaptureState>,
) -> Result<CaptureResult, String> {
    info!("Capturing full screen");

    // Close capture overlay first
    if let Some(window) = app.get_webview_window("capture-overlay") {
        let _ = window.close();
    }

    // Mark capture as inactive
    if let Ok(mut active) = state.capture_active.lock() {
        *active = false;
    }

    // Small delay to ensure overlay is closed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Capture full screen
    let screen = Screen::primary();

    match screen.capture() {
        Ok(image) => {
            // Generate unique filename
            let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
            let filename = format!("fullscreen_{}.png", timestamp);

            // Get captures directory
            let captures_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("captures");

            // Create directory if needed
            if !captures_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&captures_dir) {
                    error!("Failed to create captures directory: {}", e);
                    return Ok(CaptureResult {
                        success: false,
                        message: format!("Failed to create captures directory: {}", e),
                        path: None,
                    });
                }
            }

            let capture_path = captures_dir.join(&filename);

            // Save image
            match image.save_with_format(&capture_path, ImageFormat::Png) {
                Ok(_) => {
                    info!("Full screen capture saved: {:?}", capture_path);

                    // Update last capture path
                    if let Ok(mut last_path) = state.last_capture_path.lock() {
                        *last_path = Some(capture_path.clone());
                    }

                    Ok(CaptureResult {
                        success: true,
                        message: format!("Full screen saved to {}", filename),
                        path: Some(capture_path.to_string_lossy().to_string()),
                    })
                }
                Err(e) => {
                    error!("Failed to save capture: {}", e);
                    Ok(CaptureResult {
                        success: false,
                        message: format!("Failed to save capture: {}", e),
                        path: None,
                    })
                }
            }
        }
        Err(e) => {
            error!("Failed to capture full screen: {}", e);
            Ok(CaptureResult {
                success: false,
                message: format!("Failed to capture full screen: {}", e),
                path: None,
            })
        }
    }
}

/// Cancel capture and close overlay
/// キャプチャをキャンセルしてオーバーレイを閉じる
#[tauri::command]
pub async fn cancel_capture(app: AppHandle, state: State<'_, CaptureState>) -> Result<(), String> {
    info!("Cancelling capture");

    // Close capture overlay
    if let Some(window) = app.get_webview_window("capture-overlay") {
        window
            .close()
            .map_err(|e| format!("Failed to close capture window: {}", e))?;
    }

    // Mark capture as inactive
    if let Ok(mut active) = state.capture_active.lock() {
        *active = false;
    }

    Ok(())
}

/// Get path to last captured image
/// 最後にキャプチャした画像のパスを取得
#[tauri::command]
pub fn get_last_capture_path(state: State<'_, CaptureState>) -> Option<String> {
    state
        .last_capture_path
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
        .map(|p| p.to_string_lossy().to_string())
}

/// Check if capture mode is active
/// キャプチャモードがアクティブかどうかを確認
#[tauri::command]
pub fn is_capture_active(state: State<'_, CaptureState>) -> bool {
    state
        .capture_active
        .lock()
        .ok()
        .map(|g| *g)
        .unwrap_or(false)
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_state_default() {
        let state = CaptureState::default();
        assert!(state.last_capture_path.lock().unwrap().is_none());
        assert!(!*state.capture_active.lock().unwrap());
    }

    #[test]
    fn test_capture_result_serialization() {
        let result = CaptureResult {
            success: true,
            message: "OK".to_string(),
            path: Some("/path/to/image.png".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("image.png"));
    }

    #[test]
    fn test_capture_result_without_path() {
        let result = CaptureResult {
            success: false,
            message: "Error".to_string(),
            path: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("path")); // skip_serializing_if works
    }
}
