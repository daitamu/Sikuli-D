//! Linux Highlight Overlay Implementation (Stub)
//! Linux ハイライトオーバーレイ実装（スタブ）
//!
//! Stub implementation for Linux highlight overlay.
//! Future implementation will use X11 override-redirect windows.
//! Linuxハイライトオーバーレイのスタブ実装。
//! 将来の実装ではX11オーバーライドリダイレクトウィンドウを使用します。

use crate::{Region, Result};
use log::warn;

/// Color configuration for highlight overlay
/// ハイライトオーバーレイの色設定
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8, // 0-255
    pub g: u8, // 0-255
    pub b: u8, // 0-255
    pub a: u8, // 0-255
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn red() -> Self {
        Self::new(255, 0, 0, 255)
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
    pub color: Color,
    pub border_width: u32,
    pub duration_ms: u64,
}

impl Default for HighlightConfig {
    fn default() -> Self {
        Self {
            color: Color::red(),
            border_width: 3,
            duration_ms: 2000,
        }
    }
}

/// Display a highlight overlay (stub implementation)
/// ハイライトオーバーレイを表示（スタブ実装）
pub fn highlight(region: &Region, duration_ms: u64, _color: Color) -> Result<()> {
    warn!(
        "Linux highlight overlay not yet implemented. Would highlight region at ({}, {}) size {}x{} for {}ms",
        region.x, region.y, region.width, region.height, duration_ms
    );
    Ok(())
}

/// Highlight a match result (stub implementation)
/// マッチ結果をハイライト（スタブ実装）
pub fn highlight_match(match_region: &Region, duration_ms: u64) -> Result<()> {
    highlight(match_region, duration_ms, Color::red())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(128, 64, 32, 255);
        assert_eq!(color.r, 128);
        assert_eq!(color.g, 64);
        assert_eq!(color.b, 32);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_red() {
        let red = Color::red();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
    }

    #[test]
    fn test_highlight_config_default() {
        let config = HighlightConfig::default();
        assert_eq!(config.border_width, 3);
        assert_eq!(config.duration_ms, 2000);
    }

    #[test]
    fn test_highlight_stub() {
        let region = Region::new(100, 100, 200, 150);
        let result = highlight(&region, 2000, Color::red());
        assert!(result.is_ok());
    }
}
