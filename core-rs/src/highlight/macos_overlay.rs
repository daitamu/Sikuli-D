//! macOS Highlight Overlay Implementation
//! macOS ハイライトオーバーレイ実装
//!
//! Implements visual highlight overlays using NSWindow with floating window level.
//! NSWindow とフローティングウィンドウレベルを使用して視覚的ハイライトオーバーレイを実装します。

#![allow(non_upper_case_globals)]

use crate::{Region, Result, SikulixError};
use log::{debug, error, info};
use std::time::Duration;

/// Color configuration for highlight overlay
/// ハイライトオーバーレイの色設定
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: f64, // 0.0 - 1.0
    pub g: f64, // 0.0 - 1.0
    pub b: f64, // 0.0 - 1.0
    pub a: f64, // 0.0 - 1.0
}

impl Color {
    /// Create a new color / 新しい色を作成
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: r.clamp(0.0, 1.0),
            g: g.clamp(0.0, 1.0),
            b: b.clamp(0.0, 1.0),
            a: a.clamp(0.0, 1.0),
        }
    }

    /// Create red color / 赤色を作成
    pub fn red() -> Self {
        Self::new(1.0, 0.0, 0.0, 1.0)
    }

    /// Create green color / 緑色を作成
    pub fn green() -> Self {
        Self::new(0.0, 1.0, 0.0, 1.0)
    }

    /// Create blue color / 青色を作成
    pub fn blue() -> Self {
        Self::new(0.0, 0.0, 1.0, 1.0)
    }

    /// Create yellow color / 黄色を作成
    pub fn yellow() -> Self {
        Self::new(1.0, 1.0, 0.0, 1.0)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::red()
    }
}

/// Configuration for highlight overlay
/// ハイライトオーバーレイの設定
#[derive(Debug, Clone)]
pub struct HighlightConfig {
    /// Border color / 境界線の色
    pub color: Color,
    /// Border width in pixels / 境界線の幅（ピクセル）
    pub border_width: f64,
    /// Duration in milliseconds (0 = until manually closed) / 表示時間（ミリ秒、0 = 手動で閉じるまで）
    pub duration_ms: u64,
    /// Background opacity (0.0 = transparent, 1.0 = opaque) / 背景の不透明度（0.0 = 透明、1.0 = 不透明）
    pub background_opacity: f64,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            color: Color::red(),
            border_width: 3.0,
            duration_ms: 2000,
            background_opacity: 0.2,
        }
    }
}

/// Handle to a highlight overlay window
/// ハイライトオーバーレイウィンドウへのハンドル
pub struct HighlightHandle {
    window_ptr: *mut objc::runtime::Object,
    region: Region,
}

impl HighlightHandle {
    fn new(window_ptr: *mut objc::runtime::Object, region: Region) -> Self {
        Self { window_ptr, region }
    }

    /// Close the highlight window / ハイライトウィンドウを閉じる
    pub fn close(&mut self) {
        if !self.window_ptr.is_null() {
            unsafe {
                let _: () = objc::msg_send![self.window_ptr, close];
                let _: () = objc::msg_send![self.window_ptr, release];
            }
            self.window_ptr = std::ptr::null_mut();
            debug!("Closed highlight window for region {:?}", self.region);
        }
    }

    /// Get the highlighted region / ハイライトされた領域を取得
    pub fn region(&self) -> &Region {
        &self.region
    }
}

impl Drop for HighlightHandle {
    fn drop(&mut self) {
        self.close();
    }
}

// NSWindowStyleMask constants
const NS_WINDOW_STYLE_MASK_BORDERLESS: u64 = 0;

// NSBackingStoreType constants
const NS_BACKING_STORE_BUFFERED: u64 = 2;

// NSWindowLevel constants
const NS_FLOATING_WINDOW_LEVEL: i64 = 3;

/// Display a highlight overlay on the screen
/// 画面にハイライトオーバーレイを表示
///
/// # Arguments / 引数
///
/// * `region` - The screen region to highlight / ハイライトする画面領域
/// * `duration_ms` - Display duration in milliseconds / 表示時間（ミリ秒）
/// * `color` - Border color / 境界線の色
///
/// # Returns / 戻り値
///
/// Returns `Ok(())` on success or an error.
/// 成功時は`Ok(())`、エラー時はエラーを返します。
///
/// # Platform / プラットフォーム
///
/// This function is only available on macOS.
/// この関数はmacOSでのみ利用可能です。
pub fn highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()> {
    let config = HighlightConfig {
        color,
        duration_ms,
        ..Default::default()
    };
    highlight_with_config(region, &config)
}

/// Display a highlight overlay with custom configuration
/// カスタム設定でハイライトオーバーレイを表示
///
/// # Arguments / 引数
///
/// * `region` - The screen region to highlight / ハイライトする画面領域
/// * `config` - Highlight configuration / ハイライト設定
///
/// # Returns / 戻り値
///
/// Returns `Ok(())` on success or an error.
/// 成功時は`Ok(())`、エラー時はエラーを返します。
pub fn highlight_with_config(region: &Region, config: &HighlightConfig) -> Result<()> {
    info!(
        "Creating highlight overlay at ({}, {}) size {}x{} for {}ms",
        region.x, region.y, region.width, region.height, config.duration_ms
    );

    unsafe {
        // Get the autorelease pool
        let pool: *mut objc::runtime::Object = objc::msg_send![
            objc::class!(NSAutoreleasePool),
            new
        ];

        let result = create_highlight_window(region, config);

        // Release the pool
        let _: () = objc::msg_send![pool, release];

        match result {
            Ok(mut handle) => {
                // If duration is specified, close after timeout
                if config.duration_ms > 0 {
                    std::thread::sleep(Duration::from_millis(config.duration_ms));
                    handle.close();
                }
                Ok(())
            }
            Err(e) => {
                error!("Failed to create highlight window: {}", e);
                Err(e)
            }
        }
    }
}

/// Create a highlight window with the given configuration
/// 指定された設定でハイライトウィンドウを作成
unsafe fn create_highlight_window(
    region: &Region,
    config: &HighlightConfig,
) -> Result<HighlightHandle> {
    // Convert region to NSRect (macOS coordinates: origin at bottom-left)
    // 領域をNSRectに変換（macOS座標系：原点は左下）
    let screen_height = get_screen_height()?;
    let ns_rect = make_nsrect(
        region.x as f64,
        screen_height - (region.y as f64) - (region.height as f64),
        region.width as f64,
        region.height as f64,
    );

    // Create NSWindow
    let window_class = objc::class!(NSWindow);
    let window: *mut objc::runtime::Object = objc::msg_send![window_class, alloc];
    let window: *mut objc::runtime::Object = objc::msg_send![
        window,
        initWithContentRect: ns_rect
        styleMask: NS_WINDOW_STYLE_MASK_BORDERLESS
        backing: NS_BACKING_STORE_BUFFERED
        defer: 0u8
    ];

    if window.is_null() {
        return Err(SikulixError::PlatformError(
            "Failed to create NSWindow".to_string(),
        ));
    }

    // Configure window properties
    configure_window(window, config)?;

    // Create and configure the content view
    let view = create_content_view(ns_rect, config)?;
    let _: () = objc::msg_send![window, setContentView: view];

    // Show the window
    let _: () = objc::msg_send![window, makeKeyAndOrderFront: std::ptr::null::<objc::runtime::Object>()];

    debug!("Created highlight window at region {:?}", region);
    Ok(HighlightHandle::new(window, *region))
}

/// Configure NSWindow properties for highlight overlay
/// ハイライトオーバーレイ用のNSWindowプロパティを設定
unsafe fn configure_window(
    window: *mut objc::runtime::Object,
    config: &HighlightConfig,
) -> Result<()> {
    // Set background color (clear or semi-transparent)
    let clear_color: *mut objc::runtime::Object = objc::msg_send![
        objc::class!(NSColor),
        clearColor
    ];
    let _: () = objc::msg_send![window, setBackgroundColor: clear_color];

    // Make window transparent
    let _: () = objc::msg_send![window, setOpaque: 0u8];

    // Set floating window level (always on top)
    let _: () = objc::msg_send![window, setLevel: NS_FLOATING_WINDOW_LEVEL];

    // Enable click-through (ignores mouse events)
    let _: () = objc::msg_send![window, setIgnoresMouseEvents: 1u8];

    // Disable window animations
    let _: () = objc::msg_send![window, setAnimationBehavior: 0i64];

    // Disable shadow
    let _: () = objc::msg_send![window, setHasShadow: 0u8];

    debug!("Configured window properties for highlight overlay");
    Ok(())
}

/// Create and configure the content view with border
/// 境界線付きのコンテンツビューを作成して設定
unsafe fn create_content_view(
    frame: NSRect,
    config: &HighlightConfig,
) -> Result<*mut objc::runtime::Object> {
    // Create NSView
    let view_class = objc::class!(NSView);
    let view: *mut objc::runtime::Object = objc::msg_send![view_class, alloc];
    let view: *mut objc::runtime::Object = objc::msg_send![view, initWithFrame: frame];

    if view.is_null() {
        return Err(SikulixError::PlatformError(
            "Failed to create NSView".to_string(),
        ));
    }

    // Enable layer-backed view
    let _: () = objc::msg_send![view, setWantsLayer: 1u8];

    // Get the layer
    let layer: *mut objc::runtime::Object = objc::msg_send![view, layer];

    if !layer.is_null() {
        // Set border width
        let _: () = objc::msg_send![layer, setBorderWidth: config.border_width];

        // Create and set border color (CGColor)
        let ns_color: *mut objc::runtime::Object = objc::msg_send![
            objc::class!(NSColor),
            colorWithRed: config.color.r
            green: config.color.g
            blue: config.color.b
            alpha: config.color.a
        ];

        let cg_color: *mut objc::runtime::Object = objc::msg_send![ns_color, CGColor];
        let _: () = objc::msg_send![layer, setBorderColor: cg_color];

        // Set background color with opacity
        let bg_color: *mut objc::runtime::Object = objc::msg_send![
            objc::class!(NSColor),
            colorWithRed: config.color.r
            green: config.color.g
            blue: config.color.b
            alpha: config.background_opacity
        ];

        let bg_cg_color: *mut objc::runtime::Object = objc::msg_send![bg_color, CGColor];
        let _: () = objc::msg_send![layer, setBackgroundColor: bg_cg_color];

        // Set corner radius for rounded corners (optional, looks better)
        let _: () = objc::msg_send![layer, setCornerRadius: 4.0f64];

        debug!("Configured CALayer with border and background");
    }

    Ok(view)
}

/// NSRect structure for macOS
/// macOS用のNSRect構造体
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NSRect {
    origin: NSPoint,
    size: NSSize,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NSPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NSSize {
    width: f64,
    height: f64,
}

/// Helper function to create NSRect
/// NSRectを作成するヘルパー関数
fn make_nsrect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

/// Get the primary screen height (for coordinate conversion)
/// プライマリスクリーンの高さを取得（座標変換用）
fn get_screen_height() -> Result<f64> {
    unsafe {
        let main_screen: *mut objc::runtime::Object = objc::msg_send![
            objc::class!(NSScreen),
            mainScreen
        ];

        if main_screen.is_null() {
            return Err(SikulixError::PlatformError(
                "Failed to get main screen".to_string(),
            ));
        }

        let frame: NSRect = objc::msg_send![main_screen, frame];
        Ok(frame.size.height)
    }
}

/// Highlight a match result with default configuration
/// デフォルト設定でマッチ結果をハイライト
///
/// # Arguments / 引数
///
/// * `match_region` - The matched region to highlight / ハイライトするマッチ領域
/// * `duration_ms` - Display duration in milliseconds / 表示時間（ミリ秒）
///
/// # Returns / 戻り値
///
/// Returns `Ok(())` on success or an error.
/// 成功時は`Ok(())`、エラー時はエラーを返します。
pub fn highlight_match(match_region: &Region, duration_ms: u64) -> Result<()> {
    highlight(match_region, duration_ms, Color::red())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(0.5, 0.6, 0.7, 0.8);
        assert_eq!(color.r, 0.5);
        assert_eq!(color.g, 0.6);
        assert_eq!(color.b, 0.7);
        assert_eq!(color.a, 0.8);
    }

    #[test]
    fn test_color_clamp() {
        let color = Color::new(1.5, -0.5, 0.5, 2.0);
        assert_eq!(color.r, 1.0); // Clamped to 1.0
        assert_eq!(color.g, 0.0); // Clamped to 0.0
        assert_eq!(color.b, 0.5);
        assert_eq!(color.a, 1.0); // Clamped to 1.0
    }

    #[test]
    fn test_color_presets() {
        let red = Color::red();
        assert_eq!(red.r, 1.0);
        assert_eq!(red.g, 0.0);
        assert_eq!(red.b, 0.0);

        let green = Color::green();
        assert_eq!(green.g, 1.0);

        let blue = Color::blue();
        assert_eq!(blue.b, 1.0);

        let yellow = Color::yellow();
        assert_eq!(yellow.r, 1.0);
        assert_eq!(yellow.g, 1.0);
    }

    #[test]
    fn test_highlight_config_default() {
        let config = HighlightConfig::default();
        assert_eq!(config.border_width, 3.0);
        assert_eq!(config.duration_ms, 2000);
        assert_eq!(config.background_opacity, 0.2);
    }

    // Note: Actual window creation tests require a macOS environment with GUI support
    // 注意: 実際のウィンドウ作成テストにはGUIサポート付きのmacOS環境が必要です
}
