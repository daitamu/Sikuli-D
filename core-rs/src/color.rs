//! Color and screen capture utility functions
//! 色とスクリーンキャプチャユーティリティ関数

use crate::{Color, Region, Result, SikulixError};
use crate::screen::Screen;
use std::path::Path;

/// Get the color of a pixel at the specified screen coordinates
/// 指定された画面座標のピクセルの色を取得
///
/// Captures a 1x1 region at the specified position and returns its color.
/// 指定された位置の1x1領域をキャプチャして色を返します。
///
/// # Arguments / 引数
/// * `x` - X coordinate on screen / 画面上のX座標
/// * `y` - Y coordinate on screen / 画面上のY座標
///
/// # Returns / 戻り値
/// Returns the Color at the specified position
/// 指定位置のColorを返します
///
/// # Errors / エラー
/// Returns an error if screen capture fails or coordinates are out of bounds
/// スクリーンキャプチャ失敗または座標が範囲外の場合エラーを返します
///
/// # Example / 使用例
/// ```no_run
/// use sikulix_core::get_color;
///
/// let color = get_color(100, 200).unwrap();
/// println!("Color at (100, 200): {}", color.to_hex());
/// assert_eq!(color.r, 255); // Example assertion
/// ```
pub fn get_color(x: i32, y: i32) -> Result<Color> {
    // Capture a 1x1 pixel region
    let region = Region::new(x, y, 1, 1);
    let screen = Screen::primary();
    let img = screen.capture_region(&region)?;

    // Get the pixel color (RGBA)
    let rgba = img.as_rgba8().ok_or_else(|| {
        SikulixError::ScreenCaptureError("Failed to convert image to RGBA".to_string())
    })?;

    if let Some(pixel) = rgba.get_pixel_checked(0, 0) {
        Ok(Color::new(pixel[0], pixel[1], pixel[2], pixel[3]))
    } else {
        Err(SikulixError::ScreenCaptureError(
            "Failed to get pixel color".to_string(),
        ))
    }
}

/// Save a screenshot of the entire primary screen to a file
/// プライマリスクリーン全体のスクリーンショットをファイルに保存
///
/// Supports PNG and JPEG formats based on file extension.
/// ファイル拡張子に基づいてPNGとJPEG形式をサポートします。
///
/// # Arguments / 引数
/// * `path` - File path to save the screenshot (e.g., "screenshot.png")
///   スクリーンショットを保存するファイルパス（例: "screenshot.png"）
///
/// # Returns / 戻り値
/// Returns the saved file path
/// 保存されたファイルパスを返します
///
/// # Errors / エラー
/// Returns an error if screen capture or file save fails
/// スクリーンキャプチャまたはファイル保存が失敗した場合エラーを返します
///
/// # Example / 使用例
/// ```no_run
/// use sikulix_core::save_screen_capture;
///
/// // Save as PNG
/// let path = save_screen_capture("full_screen.png").unwrap();
/// println!("Screenshot saved to: {}", path);
///
/// // Save as JPEG
/// let path = save_screen_capture("screenshot.jpg").unwrap();
/// ```
pub fn save_screen_capture(path: &str) -> Result<String> {
    let screen = Screen::primary();
    let img = screen.capture()?;

    // Determine format from extension
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    // Save with appropriate format
    match extension.as_str() {
        "png" => img.save(path).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save PNG: {}", e),
            ))
        })?,
        "jpg" | "jpeg" => img.save(path).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save JPEG: {}", e),
            ))
        })?,
        _ => {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported image format: {}", extension),
            )))
        }
    }

    Ok(path.to_string())
}

/// Save a screenshot of a specific region to a file
/// 特定領域のスクリーンショットをファイルに保存
///
/// Supports PNG and JPEG formats based on file extension.
/// ファイル拡張子に基づいてPNGとJPEG形式をサポートします。
///
/// # Arguments / 引数
/// * `region` - The region to capture / キャプチャする領域
/// * `path` - File path to save the screenshot / スクリーンショットを保存するファイルパス
///
/// # Returns / 戻り値
/// Returns the saved file path
/// 保存されたファイルパスを返します
///
/// # Errors / エラー
/// Returns an error if screen capture or file save fails
/// スクリーンキャプチャまたはファイル保存が失敗した場合エラーを返します
///
/// # Example / 使用例
/// ```no_run
/// use sikulix_core::{save_region_capture, Region};
///
/// // Capture top-left 200x200 pixels
/// let region = Region::new(0, 0, 200, 200);
/// let path = save_region_capture(&region, "region.png").unwrap();
/// println!("Region screenshot saved to: {}", path);
/// ```
pub fn save_region_capture(region: &Region, path: &str) -> Result<String> {
    let screen = Screen::primary();
    let img = screen.capture_region(region)?;

    // Determine format from extension
    let path_obj = Path::new(path);
    let extension = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();

    // Save with appropriate format
    match extension.as_str() {
        "png" => img.save(path).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save PNG: {}", e),
            ))
        })?,
        "jpg" | "jpeg" => img.save(path).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save JPEG: {}", e),
            ))
        })?,
        _ => {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unsupported image format: {}", extension),
            )))
        }
    }

    Ok(path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // get_color() Tests / get_color()テスト
    // ========================================================================

    #[test]
    #[ignore = "Requires screen capture - run with: cargo test -- --ignored"]
    fn test_get_color_basic() {
        // Test getting a color from screen
        // 画面から色を取得するテスト
        let result = get_color(100, 100);
        assert!(result.is_ok(), "get_color should succeed");

        let color = result.unwrap();
        // Color should have valid RGBA values
        assert!(color.a == 255, "Alpha should be 255 for opaque pixel");
    }

    #[test]
    #[ignore = "Requires screen capture - run with: cargo test -- --ignored"]
    fn test_get_color_multiple_positions() {
        // Test getting colors from multiple positions
        // 複数の位置から色を取得するテスト
        let positions = [(10, 10), (100, 200), (500, 300)];

        for (x, y) in positions.iter() {
            let result = get_color(*x, *y);
            assert!(
                result.is_ok(),
                "get_color should succeed for position ({}, {})",
                x,
                y
            );
        }
    }

    // ========================================================================
    // save_screen_capture() Tests / save_screen_capture()テスト
    // ========================================================================

    #[test]
    #[ignore = "Requires screen capture and file I/O - run with: cargo test -- --ignored"]
    fn test_save_screen_capture_png() {
        // Test saving screenshot as PNG
        // PNGとしてスクリーンショットを保存するテスト
        let temp_path = "test_screen.png";

        let result = save_screen_capture(temp_path);
        assert!(result.is_ok(), "save_screen_capture should succeed");

        let saved_path = result.unwrap();
        assert_eq!(saved_path, temp_path);

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    #[ignore = "Requires screen capture and file I/O - run with: cargo test -- --ignored"]
    fn test_save_screen_capture_jpeg() {
        // Test saving screenshot as JPEG
        // JPEGとしてスクリーンショットを保存するテスト
        let temp_path = "test_screen.jpg";

        let result = save_screen_capture(temp_path);
        assert!(result.is_ok(), "save_screen_capture should succeed");

        let saved_path = result.unwrap();
        assert_eq!(saved_path, temp_path);

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    fn test_save_screen_capture_invalid_format() {
        // Test saving with unsupported format
        // サポートされていない形式で保存するテスト
        let temp_path = "test_screen.bmp";

        let result = save_screen_capture(temp_path);
        // Should fail with unsupported format error
        assert!(result.is_err(), "save_screen_capture should fail for .bmp");
    }

    // ========================================================================
    // save_region_capture() Tests / save_region_capture()テスト
    // ========================================================================

    #[test]
    #[ignore = "Requires screen capture and file I/O - run with: cargo test -- --ignored"]
    fn test_save_region_capture_png() {
        // Test saving region screenshot as PNG
        // 領域スクリーンショットをPNGとして保存するテスト
        let region = Region::new(0, 0, 200, 200);
        let temp_path = "test_region.png";

        let result = save_region_capture(&region, temp_path);
        assert!(result.is_ok(), "save_region_capture should succeed");

        let saved_path = result.unwrap();
        assert_eq!(saved_path, temp_path);

        // Verify file exists and has reasonable size
        if let Ok(metadata) = std::fs::metadata(temp_path) {
            assert!(metadata.len() > 0, "Saved file should not be empty");
        }

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    #[ignore = "Requires screen capture and file I/O - run with: cargo test -- --ignored"]
    fn test_save_region_capture_small_region() {
        // Test saving a small region (1x1 pixel)
        // 小さな領域（1x1ピクセル）を保存するテスト
        let region = Region::new(100, 100, 1, 1);
        let temp_path = "test_pixel.png";

        let result = save_region_capture(&region, temp_path);
        assert!(result.is_ok(), "save_region_capture should succeed for 1x1");

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }

    #[test]
    #[ignore = "Requires screen capture and file I/O - run with: cargo test -- --ignored"]
    fn test_save_region_capture_large_region() {
        // Test saving a large region
        // 大きな領域を保存するテスト
        let region = Region::new(0, 0, 1024, 768);
        let temp_path = "test_large.jpg";

        let result = save_region_capture(&region, temp_path);
        assert!(result.is_ok(), "save_region_capture should succeed for large region");

        // Clean up
        let _ = std::fs::remove_file(temp_path);
    }
}
