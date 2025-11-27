//! Windows Highlight Overlay Implementation
//! Windows ハイライトオーバーレイ実装
//!
//! Delegates to the debug::highlight module which has the full Windows GDI implementation.
//! debug::highlightモジュールに委譲し、完全なWindows GDI実装を使用します。

use crate::{Region, Result};

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

/// Display a highlight overlay using Windows GDI
/// Windows GDIを使用してハイライトオーバーレイを表示
pub fn highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()> {
    // Delegate to debug::highlight which has the full Windows implementation
    // 完全なWindows実装を持つdebug::highlightに委譲
    let core_color = crate::Color::new(color.r, color.g, color.b, color.a);
    crate::debug::highlight::highlight(region, duration_ms, core_color)
}

/// Highlight a match result
/// マッチ結果をハイライト
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
