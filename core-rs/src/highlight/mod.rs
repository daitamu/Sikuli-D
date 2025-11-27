//! Highlight Module - Visual Feedback for Matches
//! ハイライトモジュール - マッチの視覚的フィードバック
//!
//! Provides visual highlighting functionality for screen regions and matches.
//! 画面領域とマッチの視覚的ハイライト機能を提供します。

use crate::{Region, Result};

// Platform-specific overlay implementations
// プラットフォーム固有のオーバーレイ実装
#[cfg(target_os = "macos")]
pub mod macos_overlay;

#[cfg(target_os = "windows")]
pub mod windows_overlay;

#[cfg(target_os = "linux")]
pub mod linux_overlay;

// Re-export platform-specific types
// プラットフォーム固有の型を再エクスポート
#[cfg(target_os = "macos")]
pub use macos_overlay::{highlight, highlight_match, Color, HighlightConfig};

#[cfg(target_os = "windows")]
pub use windows_overlay::{highlight, highlight_match, Color, HighlightConfig};

#[cfg(target_os = "linux")]
pub use linux_overlay::{highlight, highlight_match, Color, HighlightConfig};

/// Highlight overlay for visual feedback
/// 視覚的フィードバック用のハイライトオーバーレイ
///
/// Displays a visual highlight on a screen region to provide user feedback.
/// This struct provides a high-level interface that wraps platform-specific implementations.
/// 画面領域に視覚的ハイライトを表示してユーザーフィードバックを提供します。
/// この構造体はプラットフォーム固有実装をラップする高レベルインターフェースを提供します。
///
/// # Platform Support / プラットフォームサポート
///
/// - **macOS**: NSWindow with CALayer for graphical overlay / グラフィカルオーバーレイ用のCALayer付きNSWindow
/// - **Windows**: Layered window with GDI+ (stub, to be implemented) / GDI+付きレイヤードウィンドウ（スタブ、実装予定）
/// - **Linux**: X11 override-redirect window (stub, to be implemented) / X11オーバーライドリダイレクトウィンドウ（スタブ、実装予定）
///
/// # Example / 使用例
///
/// ```rust
/// use sikulix_core::{Highlight, Region};
///
/// let region = Region::new(100, 100, 200, 150);
/// let highlight = Highlight::new(region);
/// highlight.show_for(2.0);
/// ```
#[derive(Debug, Clone)]
pub struct Highlight {
    /// The region to highlight / ハイライトする領域
    region: Region,
}

impl Highlight {
    /// Create a new highlight for the given region
    /// 指定された領域の新しいハイライトを作成
    ///
    /// # Arguments / 引数
    ///
    /// * `region` - The region to highlight / ハイライトする領域
    ///
    /// # Example / 使用例
    ///
    /// ```rust
    /// use sikulix_core::{Highlight, Region};
    ///
    /// let region = Region::new(100, 100, 200, 150);
    /// let highlight = Highlight::new(region);
    /// ```
    pub fn new(region: Region) -> Self {
        Self { region }
    }

    /// Show the highlight for a specified duration
    /// 指定された時間だけハイライトを表示
    ///
    /// Displays the highlight for the given number of seconds.
    /// 指定された秒数だけハイライトを表示します。
    ///
    /// # Arguments / 引数
    ///
    /// * `seconds` - Duration to show highlight in seconds
    ///   ハイライト表示時間（秒）
    ///
    /// # Platform Implementation / プラットフォーム実装
    ///
    /// - **macOS**: Creates a floating NSWindow overlay / フローティングNSWindowオーバーレイを作成
    /// - **Windows**: Creates a layered window (stub) / レイヤードウィンドウを作成（スタブ）
    /// - **Linux**: Creates X11 window (stub) / X11ウィンドウを作成（スタブ）
    ///
    /// # Example / 使用例
    ///
    /// ```rust
    /// use sikulix_core::{Highlight, Region};
    ///
    /// let highlight = Highlight::new(Region::new(100, 100, 200, 150));
    /// highlight.show_for(2.5);
    /// ```
    pub fn show_for(&self, seconds: f64) -> Result<()> {
        let duration_ms = (seconds * 1000.0).max(0.0) as u64;

        #[cfg(target_os = "macos")]
        {
            highlight(&self.region, duration_ms, Color::default())
        }

        #[cfg(target_os = "windows")]
        {
            highlight(&self.region, duration_ms, Color::default())
        }

        #[cfg(target_os = "linux")]
        {
            highlight(&self.region, duration_ms, Color::default())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            log::warn!(
                "Highlight not implemented for this platform. Region: ({}, {}) {}x{}",
                self.region.x,
                self.region.y,
                self.region.width,
                self.region.height
            );
            Ok(())
        }
    }

    /// Get the highlighted region
    /// ハイライトされている領域を取得
    ///
    /// # Returns / 戻り値
    ///
    /// Returns the region being highlighted.
    /// ハイライトされている領域を返します。
    pub fn get_region(&self) -> &Region {
        &self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_new() {
        let region = Region::new(100, 100, 200, 150);
        let highlight = Highlight::new(region);
        assert_eq!(highlight.get_region(), &region);
    }

    #[test]
    fn test_highlight_get_region() {
        let region = Region::new(100, 100, 200, 150);
        let highlight = Highlight::new(region);

        let returned_region = highlight.get_region();
        assert_eq!(returned_region.x, 100);
        assert_eq!(returned_region.y, 100);
        assert_eq!(returned_region.width, 200);
        assert_eq!(returned_region.height, 150);
    }

    #[test]
    fn test_highlight_clone() {
        let region = Region::new(100, 100, 200, 150);
        let highlight1 = Highlight::new(region);

        let highlight2 = highlight1.clone();
        assert_eq!(highlight1.get_region(), highlight2.get_region());
    }

    #[test]
    fn test_highlight_debug() {
        let region = Region::new(100, 100, 200, 150);
        let highlight = Highlight::new(region);

        let debug_str = format!("{:?}", highlight);
        assert!(debug_str.contains("Highlight"));
    }

    #[test]
    fn test_highlight_multiple_regions() {
        let region1 = Region::new(0, 0, 100, 100);
        let region2 = Region::new(200, 200, 50, 50);

        let highlight1 = Highlight::new(region1);
        let highlight2 = Highlight::new(region2);

        assert_eq!(highlight1.get_region(), &region1);
        assert_eq!(highlight2.get_region(), &region2);
    }

    // Platform-specific tests would require actual GUI environment
    // プラットフォーム固有のテストには実際のGUI環境が必要です
    #[test]
    #[ignore = "Requires GUI environment"]
    fn test_highlight_show_for() {
        let region = Region::new(100, 100, 200, 150);
        let highlight = Highlight::new(region);

        // This would actually show a highlight on screen
        // これは実際に画面にハイライトを表示します
        let result = highlight.show_for(0.1);
        assert!(result.is_ok());
    }
}
