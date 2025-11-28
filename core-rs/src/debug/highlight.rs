//! Visual Highlight Overlay for debugging
//! デバッグ用のビジュアルハイライトオーバーレイ
//!
//! Provides platform-specific visual overlays to highlight regions on screen.
//! プラットフォーム固有のビジュアルオーバーレイを提供し、画面上の領域をハイライト表示します。

use crate::{Color, Region, Result};
#[cfg(any(
    target_os = "windows",
    not(any(target_os = "windows", target_os = "macos", target_os = "linux"))
))]
use crate::SikulixError;
#[cfg(target_os = "windows")]
use log::debug;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use log::{info, warn};
#[cfg(target_os = "windows")]
use std::thread;
#[cfg(target_os = "windows")]
use std::time::Duration;

/// Highlight overlay configuration
/// ハイライトオーバーレイ設定
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Border color (RGB)
    /// 境界線の色（RGB）
    pub color: (u8, u8, u8),
    /// Border width in pixels
    /// 境界線の幅（ピクセル）
    pub border_width: u32,
    /// Display duration in milliseconds (0 = manual close)
    /// 表示時間（ミリ秒、0 = 手動クローズ）
    pub duration_ms: u64,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            color: (255, 0, 0), // Red / 赤
            border_width: 3,
            duration_ms: 2000, // 2 seconds / 2秒
        }
    }
}

impl HighlightConfig {
    /// Create a new highlight configuration
    /// 新しいハイライト設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the highlight color
    /// ハイライト色を設定
    pub fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color = (r, g, b);
        self
    }

    /// Set the border width
    /// 境界線の幅を設定
    pub fn with_border_width(mut self, width: u32) -> Self {
        self.border_width = width;
        self
    }

    /// Set the display duration
    /// 表示時間を設定
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Create from a Color struct
    /// Color構造体から作成
    pub fn from_color(color: Color) -> Self {
        Self {
            color: (color.r, color.g, color.b),
            ..Default::default()
        }
    }
}

// =============================================================================
// Windows Implementation / Windows実装
// =============================================================================

#[cfg(target_os = "windows")]
mod windows_impl {
    use super::*;
    use windows::core::w;
    use windows::Win32::Foundation::*;
    use windows::Win32::Graphics::Gdi::*;
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::*;

    /// Show highlight overlay on Windows
    /// Windowsでハイライトオーバーレイを表示
    pub fn show_highlight(region: &Region, config: &HighlightConfig) -> Result<()> {
        info!(
            "Showing Windows highlight overlay at ({}, {}) size {}x{}",
            region.x, region.y, region.width, region.height
        );

        unsafe {
            // Register window class
            // ウィンドウクラスを登録
            let class_name = w!("SikuliDHighlightOverlay");
            let hinstance = GetModuleHandleW(None).map_err(|e| {
                SikulixError::PlatformError(format!("Failed to get module handle: {}", e))
            })?;

            let wc = WNDCLASSW {
                lpfnWndProc: Some(overlay_wnd_proc),
                hInstance: hinstance.into(),
                lpszClassName: class_name,
                style: CS_HREDRAW | CS_VREDRAW,
                hCursor: LoadCursorW(HINSTANCE::default(), IDC_ARROW).unwrap_or_default(),
                hbrBackground: HBRUSH::default(),
                ..Default::default()
            };

            let atom = RegisterClassW(&wc);
            if atom == 0 {
                // Class may already be registered, which is fine
                // クラスは既に登録されている可能性があり、それは問題ありません
                debug!("Window class already registered or registration failed");
            }

            // Create layered, topmost, transparent window
            // レイヤード、最前面、透明なウィンドウを作成
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("Highlight Overlay"),
                WS_POPUP,
                region.x,
                region.y,
                region.width as i32,
                region.height as i32,
                HWND::default(),
                HMENU::default(),
                hinstance,
                None,
            )
            .map_err(|e| {
                SikulixError::PlatformError(format!("Failed to create overlay window: {}", e))
            })?;

            if hwnd.is_invalid() || hwnd.0.is_null() {
                return Err(SikulixError::PlatformError(
                    "Failed to create overlay window".to_string(),
                ));
            }

            debug!("Created overlay window: {:?}", hwnd);

            // Set window transparency
            // ウィンドウの透明度を設定
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA).map_err(|e| {
                let _ = DestroyWindow(hwnd);
                SikulixError::PlatformError(format!("Failed to set window attributes: {}", e))
            })?;

            // Show window without activating
            // アクティブ化せずにウィンドウを表示
            let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            let _ = UpdateWindow(hwnd);

            debug!("Window shown, drawing border");

            // Draw the border
            // 境界線を描画
            let hdc = GetDC(hwnd);
            if !hdc.is_invalid() {
                draw_border(hdc, region, config);
                let _ = ReleaseDC(hwnd, hdc);
            }

            debug!("Border drawn");

            // Handle duration
            // 時間を処理
            if config.duration_ms > 0 {
                // Auto-close after duration using a timer
                // タイマーを使用して指定時間後に自動クローズ
                let duration = Duration::from_millis(config.duration_ms);
                let hwnd_ptr = hwnd.0 as usize; // Convert to usize for Send
                thread::spawn(move || {
                    thread::sleep(duration);
                    // Reconstruct HWND from pointer
                    // ポインタからHWNDを再構築
                    let hwnd = HWND(hwnd_ptr as *mut std::ffi::c_void);
                    // Close the window (outer unsafe block covers this)
                    // ウィンドウを閉じる（外側のunsafeブロックでカバー）
                    let _ = DestroyWindow(hwnd);
                });
            }

            Ok(())
        }
    }

    /// Draw border rectangle on the device context
    /// デバイスコンテキストに境界線矩形を描画
    unsafe fn draw_border(hdc: HDC, region: &Region, config: &HighlightConfig) {
        // Create pen with specified color and width
        // 指定された色と幅でペンを作成
        let color_ref = COLORREF(
            config.color.0 as u32
                | ((config.color.1 as u32) << 8)
                | ((config.color.2 as u32) << 16),
        );

        let pen = CreatePen(PS_SOLID, config.border_width as i32, color_ref);
        if pen.is_invalid() {
            warn!("Failed to create pen for border");
            return;
        }

        let old_pen = SelectObject(hdc, pen);

        // Use null brush (transparent fill)
        // ヌルブラシを使用（透明な塗りつぶし）
        let brush = GetStockObject(NULL_BRUSH);
        let old_brush = SelectObject(hdc, brush);

        // Draw rectangle
        // 矩形を描画
        let _ = Rectangle(hdc, 0, 0, region.width as i32, region.height as i32);

        // Restore old objects and cleanup
        // 古いオブジェクトを復元してクリーンアップ
        SelectObject(hdc, old_brush);
        SelectObject(hdc, old_pen);
        let _ = DeleteObject(pen);
    }

    /// Window procedure for the overlay window
    /// オーバーレイウィンドウのウィンドウプロシージャ
    unsafe extern "system" fn overlay_wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let _hdc = BeginPaint(hwnd, &mut ps);
                // Drawing is already done in show_highlight
                // 描画は既にshow_highlightで完了しています
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                debug!("Overlay window destroyed");
                LRESULT(0)
            }
            WM_CLOSE => {
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// =============================================================================
// macOS Implementation (Stub) / macOS実装（スタブ）
// =============================================================================

#[cfg(target_os = "macos")]
mod macos_impl {
    use super::*;

    pub fn show_highlight(region: &Region, config: &HighlightConfig) -> Result<()> {
        // TODO: Implement using NSWindow with CALayer
        warn!(
            "macOS highlight overlay not yet implemented, logging instead: {:?} @ {:?}",
            config, region
        );
        info!(
            "HIGHLIGHT: Region({}, {}, {}x{}) for {}ms",
            region.x, region.y, region.width, region.height, config.duration_ms
        );
        Ok(())
    }
}

// =============================================================================
// Linux Implementation / Linux実装
// =============================================================================

#[cfg(target_os = "linux")]
mod linux_impl {
    use super::*;

    pub fn show_highlight(region: &Region, config: &HighlightConfig) -> Result<()> {
        // Delegate to the Linux-specific implementation
        // Linux固有の実装に委譲
        crate::debug::highlight_linux::show_highlight(
            region,
            &crate::debug::highlight_linux::HighlightConfig {
                color: config.color,
                border_width: config.border_width,
                duration_ms: config.duration_ms,
            },
        )
    }
}

// =============================================================================
// Public API / 公開API
// =============================================================================

/// Show a highlight overlay for a region
/// 領域のハイライトオーバーレイを表示
///
/// # Arguments / 引数
///
/// * `region` - Region to highlight / ハイライトする領域
/// * `duration_ms` - Duration in milliseconds / 時間（ミリ秒）
/// * `color` - Border color / 境界線の色
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::{Region, Color};
/// use sikulid::debug::highlight::highlight;
///
/// let region = Region::new(100, 100, 200, 150);
/// let color = Color::rgb(255, 0, 0); // Red
/// highlight(&region, 2000, color).unwrap();
/// ```
pub fn highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()> {
    let config = HighlightConfig {
        color: (color.r, color.g, color.b),
        duration_ms,
        ..Default::default()
    };

    show_highlight_with_config(region, &config)
}

/// Show a highlight overlay with custom configuration
/// カスタム設定でハイライトオーバーレイを表示
///
/// # Arguments / 引数
///
/// * `region` - Region to highlight / ハイライトする領域
/// * `config` - Highlight configuration / ハイライト設定
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::Region;
/// use sikulid::debug::highlight::{show_highlight_with_config, HighlightConfig};
///
/// let region = Region::new(100, 100, 200, 150);
/// let config = HighlightConfig::new()
///     .with_color(0, 255, 0)  // Green
///     .with_border_width(5)
///     .with_duration_ms(3000);
/// show_highlight_with_config(&region, &config).unwrap();
/// ```
pub fn show_highlight_with_config(region: &Region, config: &HighlightConfig) -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows_impl::show_highlight(region, config);

    #[cfg(target_os = "macos")]
    return macos_impl::show_highlight(region, config);

    #[cfg(target_os = "linux")]
    return linux_impl::show_highlight(region, config);

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        warn!(
            "Highlight overlay not supported on this platform: {:?} @ {:?}",
            config, region
        );
        Err(SikulixError::PlatformError(
            "Highlight overlay not supported on this platform".to_string(),
        ))
    }
}

/// Highlight a match result
/// マッチ結果をハイライト
///
/// # Arguments / 引数
///
/// * `match_result` - Match result to highlight / ハイライトするマッチ結果
/// * `duration_ms` - Duration in milliseconds / 時間（ミリ秒）
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::{Region, Match, Color};
/// use sikulid::debug::highlight::highlight_match;
///
/// let region = Region::new(100, 100, 200, 150);
/// let match_result = Match::new(region, 0.95);
/// highlight_match(&match_result, 2000).unwrap();
/// ```
pub fn highlight_match(match_result: &crate::Match, duration_ms: u64) -> Result<()> {
    let color = Color::rgb(255, 0, 0); // Red for matches / マッチには赤
    highlight(&match_result.region, duration_ms, color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_config_default() {
        let config = HighlightConfig::default();
        assert_eq!(config.color, (255, 0, 0));
        assert_eq!(config.border_width, 3);
        assert_eq!(config.duration_ms, 2000);
    }

    #[test]
    fn test_highlight_config_builder() {
        let config = HighlightConfig::new()
            .with_color(0, 255, 0)
            .with_border_width(5)
            .with_duration_ms(3000);

        assert_eq!(config.color, (0, 255, 0));
        assert_eq!(config.border_width, 5);
        assert_eq!(config.duration_ms, 3000);
    }

    #[test]
    fn test_highlight_config_from_color() {
        let color = Color::rgb(100, 150, 200);
        let config = HighlightConfig::from_color(color);

        assert_eq!(config.color, (100, 150, 200));
        assert_eq!(config.border_width, 3);
        assert_eq!(config.duration_ms, 2000);
    }

    // Platform-specific tests only run on their respective platforms
    // プラットフォーム固有のテストはそれぞれのプラットフォームでのみ実行されます

    #[test]
    #[cfg(target_os = "windows")]
    #[ignore = "Requires GUI environment"]
    fn test_windows_highlight() {
        let region = Region::new(100, 100, 200, 150);
        let config = HighlightConfig::new().with_duration_ms(500);

        let result = show_highlight_with_config(&region, &config);
        assert!(result.is_ok());

        // Wait for the highlight to appear
        // ハイライトが表示されるのを待つ
        std::thread::sleep(Duration::from_millis(600));
    }

    #[test]
    fn test_highlight_api() {
        let region = Region::new(100, 100, 200, 150);
        let color = Color::rgb(255, 0, 0);

        // This will log a warning on non-Windows platforms or succeed on Windows
        // Windowsでは成功し、非Windowsプラットフォームでは警告をログ出力します
        let _ = highlight(&region, 1000, color);
    }

    #[test]
    fn test_highlight_match_api() {
        let region = Region::new(100, 100, 200, 150);
        let match_result = crate::Match::new(region, 0.95);

        // This will log a warning on non-Windows platforms or succeed on Windows
        // Windowsでは成功し、非Windowsプラットフォームでは警告をログ出力します
        let _ = highlight_match(&match_result, 1000);
    }
}
