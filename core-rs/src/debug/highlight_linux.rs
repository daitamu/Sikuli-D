//! Linux (X11) Highlight Overlay Implementation
//! Linux (X11) ハイライトオーバーレイ実装
//!
//! Provides platform-specific highlight overlay using X11 override-redirect windows.
//! X11 オーバーライドリダイレクトウィンドウを使用したプラットフォーム固有のハイライトオーバーレイを提供します。

use crate::{Color, Region, Result};
use log::{info, warn};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::*;
#[cfg(target_os = "linux")]
use x11rb::rust_connection::RustConnection;
#[cfg(target_os = "linux")]
use x11rb::COPY_DEPTH_FROM_PARENT;

/// Configuration for highlight overlay display
/// ハイライトオーバーレイ表示の設定
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// RGB color / RGB色
    pub color: (u8, u8, u8),
    /// Border width in pixels / 境界線の幅（ピクセル）
    pub border_width: u32,
    /// Duration in milliseconds (0 = manual close) / 表示時間（ミリ秒）（0 = 手動クローズ）
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

impl From<Color> for HighlightConfig {
    fn from(color: Color) -> Self {
        Self {
            color: (color.r, color.g, color.b),
            ..Default::default()
        }
    }
}

/// Show a highlight overlay on the screen (Linux X11 implementation)
/// 画面にハイライトオーバーレイを表示（Linux X11実装）
///
/// # Arguments / 引数
///
/// * `region` - Region to highlight / ハイライトする領域
/// * `duration_ms` - Display duration in milliseconds / 表示時間（ミリ秒）
/// * `color` - Highlight color / ハイライト色
///
/// # Returns / 戻り値
///
/// * `Ok(())` - Success / 成功
/// * `Err(SikulixError)` - On failure / 失敗時
///
/// # X11 Implementation Details / X11 実装詳細
///
/// - Creates an override-redirect window (not managed by window manager)
///   オーバーライドリダイレクトウィンドウを作成（ウィンドウマネージャーで管理されない）
/// - Sets border color for the highlight effect
///   ハイライト効果の境界線色を設定
/// - Click-through is achieved via InputOnly window or empty event mask
///   クリックスルーは InputOnly ウィンドウまたは空のイベントマスクで実現
/// - Auto-closes after specified duration
///   指定時間後に自動的に閉じる
///
/// # Platform Support / プラットフォームサポート
///
/// - ✅ X11 (Full support)
/// - ⚠️ Wayland (Limited - compositor dependent, may fall back to logging)
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::{Region, Color};
/// use sikulid::debug::highlight_linux::highlight;
///
/// let region = Region::new(100, 100, 200, 150);
/// let color = Color::rgb(255, 0, 0);
/// highlight(&region, 2000, color).ok();
/// ```
#[cfg(target_os = "linux")]
pub fn highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()> {
    let config = HighlightConfig {
        color: (color.r, color.g, color.b),
        border_width: 3,
        duration_ms,
    };

    show_highlight_impl(region, &config)
}

/// Show a highlight overlay with custom configuration
/// カスタム設定でハイライトオーバーレイを表示
///
/// # Arguments / 引数
///
/// * `region` - Region to highlight / ハイライトする領域
/// * `config` - Highlight configuration / ハイライト設定
///
/// # Returns / 戻り値
///
/// * `Ok(())` - Success / 成功
/// * `Err(SikulixError)` - On failure / 失敗時
#[cfg(target_os = "linux")]
pub fn show_highlight(region: &Region, config: &HighlightConfig) -> Result<()> {
    show_highlight_impl(region, config)
}

#[cfg(target_os = "linux")]
fn show_highlight_impl(region: &Region, config: &HighlightConfig) -> Result<()> {
    info!(
        "Showing highlight overlay: region=({}, {}, {}x{}), color=({}, {}, {}), border_width={}, duration={}ms",
        region.x, region.y, region.width, region.height,
        config.color.0, config.color.1, config.color.2,
        config.border_width, config.duration_ms
    );

    // Check if we're running under Wayland
    // Wayland 環境で実行されているか確認
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        warn!(
            "Wayland display server detected. Highlight overlay may not work properly. \
            Falling back to logging. For full highlight support, use X11 or Xwayland."
        );
        warn!(
            "Waylandディスプレイサーバーが検出されました。ハイライトオーバーレイが正しく動作しない可能性があります。\
            ログ出力にフォールバックします。完全なハイライトサポートにはX11またはXwaylandを使用してください。"
        );

        // Log the highlight information
        // ハイライト情報をログ出力
        info!(
            "HIGHLIGHT: Region [{}x{}+{}+{}] color=RGB({}, {}, {})",
            region.width,
            region.height,
            region.x,
            region.y,
            config.color.0,
            config.color.1,
            config.color.2
        );

        // Sleep for the duration
        // 指定時間スリープ
        if config.duration_ms > 0 {
            thread::sleep(Duration::from_millis(config.duration_ms));
        }

        return Ok(());
    }

    // Connect to X11 server
    // X11サーバーに接続
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::PlatformError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];

    // Generate window ID
    // ウィンドウIDを生成
    let window = conn
        .generate_id()
        .map_err(|e| SikulixError::PlatformError(format!("Failed to generate window ID: {}", e)))?;

    // Calculate border pixel value (RGB to X11 pixel format)
    // 境界線ピクセル値を計算（RGBからX11ピクセルフォーマットへ）
    let border_pixel =
        config.color.0 as u32 | ((config.color.1 as u32) << 8) | ((config.color.2 as u32) << 16);

    // Create window attributes
    // ウィンドウ属性を作成
    let values = CreateWindowAux::new()
        .override_redirect(1) // Don't let window manager manage this window
        .background_pixel(screen.black_pixel) // Transparent background
        .border_pixel(border_pixel) // Border color
        .event_mask(EventMask::NO_EVENT); // No events - click-through

    debug!(
        "Creating highlight window: id={}, x={}, y={}, width={}, height={}, border_width={}",
        window, region.x, region.y, region.width, region.height, config.border_width
    );

    // Create the window
    // ウィンドウを作成
    conn.create_window(
        COPY_DEPTH_FROM_PARENT,     // depth
        window,                     // wid
        screen.root,                // parent
        region.x as i16,            // x
        region.y as i16,            // y
        region.width as u16,        // width
        region.height as u16,       // height
        config.border_width as u16, // border_width
        WindowClass::INPUT_OUTPUT,  // class
        screen.root_visual,         // visual
        &values,                    // value_list
    )
    .map_err(|e| SikulixError::PlatformError(format!("Failed to create window: {}", e)))?;

    // Try to set window transparency (optional, may not work on all compositors)
    // ウィンドウの透明度を設定（オプション、すべてのコンポジターで動作しない可能性あり）
    try_set_transparency(&conn, window).ok();

    // Map the window (make it visible)
    // ウィンドウをマップ（表示）
    conn.map_window(window)
        .map_err(|e| SikulixError::PlatformError(format!("Failed to map window: {}", e)))?;

    // Flush to ensure window is displayed
    // フラッシュしてウィンドウを確実に表示
    conn.flush()
        .map_err(|e| SikulixError::PlatformError(format!("Failed to flush: {}", e)))?;

    debug!("Highlight window created and mapped successfully");

    // Auto-close after duration
    // 指定時間後に自動クローズ
    if config.duration_ms > 0 {
        // Spawn a thread to close the window after duration
        // 指定時間後にウィンドウを閉じるスレッドを生成
        let duration = config.duration_ms;
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(duration));

            // Reconnect to X11 to close the window
            // ウィンドウを閉じるためにX11に再接続
            if let Ok((conn, _)) = RustConnection::connect(None) {
                debug!("Closing highlight window after {}ms", duration);
                let _ = conn.destroy_window(window);
                let _ = conn.flush();
            }
        });
    }

    Ok(())
}

/// Try to set window transparency using X11 properties
/// X11プロパティを使用してウィンドウの透明度を設定
///
/// This is optional and may not work on all X11 compositors.
/// これはオプションであり、すべてのX11コンポジターで動作しない可能性があります。
#[cfg(target_os = "linux")]
fn try_set_transparency(conn: &impl Connection, window: Window) -> Result<()> {
    // Try to set _NET_WM_WINDOW_OPACITY property
    // _NET_WM_WINDOW_OPACITY プロパティを設定
    let opacity_atom = conn
        .intern_atom(false, b"_NET_WM_WINDOW_OPACITY")
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .map(|reply| reply.atom);

    if let Some(atom) = opacity_atom {
        // Set opacity to 90% (0xE6000000 in X11 format)
        // 不透明度を90%に設定（X11フォーマットで0xE6000000）
        let opacity: u32 = 0xE6000000;
        let data = opacity.to_ne_bytes();

        let _ = conn.change_property32(
            PropMode::REPLACE,
            window,
            atom,
            AtomEnum::CARDINAL,
            &[opacity],
        );

        debug!("Set window opacity property");
    }

    Ok(())
}

/// Stub implementation for non-Linux platforms
/// 非Linuxプラットフォーム用のスタブ実装
#[cfg(not(target_os = "linux"))]
pub fn highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()> {
    warn!("Linux highlight overlay not available on this platform. Logging only.");
    info!(
        "HIGHLIGHT: Region [{}x{}+{}+{}] color=RGB({}, {}, {}) duration={}ms",
        region.width, region.height, region.x, region.y, color.r, color.g, color.b, duration_ms
    );
    if duration_ms > 0 {
        thread::sleep(Duration::from_millis(duration_ms));
    }
    Ok(())
}

/// Stub implementation for non-Linux platforms
/// 非Linuxプラットフォーム用のスタブ実装
#[cfg(not(target_os = "linux"))]
pub fn show_highlight(region: &Region, config: &HighlightConfig) -> Result<()> {
    warn!("Linux highlight overlay not available on this platform. Logging only.");
    info!(
        "HIGHLIGHT: Region [{}x{}+{}+{}] color=RGB({}, {}, {}) border_width={} duration={}ms",
        region.width,
        region.height,
        region.x,
        region.y,
        config.color.0,
        config.color.1,
        config.color.2,
        config.border_width,
        config.duration_ms
    );
    if config.duration_ms > 0 {
        thread::sleep(Duration::from_millis(config.duration_ms));
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
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
    fn test_highlight_config_from_color() {
        let color = Color::rgb(0, 255, 0);
        let config = HighlightConfig::from(color);
        assert_eq!(config.color, (0, 255, 0));
        assert_eq!(config.border_width, 3);
        assert_eq!(config.duration_ms, 2000);
    }

    #[test]
    fn test_highlight_config_custom() {
        let config = HighlightConfig {
            color: (0, 0, 255),
            border_width: 5,
            duration_ms: 1000,
        };
        assert_eq!(config.color, (0, 0, 255));
        assert_eq!(config.border_width, 5);
        assert_eq!(config.duration_ms, 1000);
    }

    // Integration tests - only run if X11 is available
    // 統合テスト - X11が利用可能な場合のみ実行
    #[test]
    #[ignore = "Requires X11 display"]
    fn test_highlight_x11_connection() {
        // Check if we can connect to X11
        // X11に接続できるか確認
        let result = RustConnection::connect(None);
        assert!(result.is_ok(), "Should be able to connect to X11");
    }

    #[test]
    #[ignore = "Requires X11 display and visual verification"]
    fn test_highlight_display_short() {
        // Test displaying a highlight for a short duration
        // 短時間ハイライトを表示するテスト
        let region = Region::new(100, 100, 200, 150);
        let color = Color::rgb(255, 0, 0);
        let result = highlight(&region, 500, color);

        assert!(
            result.is_ok(),
            "Highlight should display successfully: {:?}",
            result.err()
        );
    }

    #[test]
    #[ignore = "Requires X11 display and visual verification"]
    fn test_show_highlight_custom_config() {
        // Test displaying a highlight with custom configuration
        // カスタム設定でハイライトを表示するテスト
        let region = Region::new(200, 200, 300, 200);
        let config = HighlightConfig {
            color: (0, 255, 0), // Green
            border_width: 5,
            duration_ms: 500,
        };

        let result = show_highlight(&region, &config);
        assert!(
            result.is_ok(),
            "Custom highlight should display successfully: {:?}",
            result.err()
        );
    }
}
