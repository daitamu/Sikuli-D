//! SikuliX Core Library - Next Generation
//! SikuliX コアライブラリ - 次世代版
//!
//! A high-performance, low-memory GUI automation library written in Rust.
//! Rustで書かれた高性能・省メモリのGUI自動化ライブラリ。
//!
//! # Features / 機能
//! - Image matching with template matching algorithm / テンプレートマッチングによる画像認識
//! - OCR (Optical Character Recognition) / OCR（光学文字認識）
//! - Cross-platform screen capture / クロスプラットフォームスクリーンキャプチャ
//! - Mouse and keyboard control / マウス・キーボード制御
//! - Python scripting support (Python 2/3 dual runtime) / Pythonスクリプト対応（Python 2/3デュアルランタイム）

pub mod image;
pub mod python;
pub mod screen;

// Re-export commonly used types
pub use image::ImageMatcher;
pub use image::{OcrConfig, OcrEngine, OcrLanguage, OcrResult};
pub use python::{PythonVersion, SyntaxAnalyzer};
pub use screen::{Key, Keyboard, Mouse, Screen};

#[cfg(feature = "python")]
pub use python::PythonRuntime;

use thiserror::Error;

/// SikuliX Core Error Types
/// SikuliX コアエラー型
#[derive(Error, Debug)]
pub enum SikulixError {
    /// Image not found on screen / 画面上に画像が見つかりません
    #[error("Image not found on screen")]
    ImageNotFound,

    /// Image loading failed / 画像の読み込みに失敗
    #[error("Image loading failed: {0}")]
    ImageLoadError(String),

    /// Screen capture failed / スクリーンキャプチャに失敗
    #[error("Screen capture failed: {0}")]
    ScreenCaptureError(String),

    /// OCR failed / OCRに失敗
    #[error("OCR failed: {0}")]
    OcrError(String),

    /// Mouse operation failed / マウス操作に失敗
    #[error("Mouse operation failed: {0}")]
    MouseError(String),

    /// Keyboard operation failed / キーボード操作に失敗
    #[error("Keyboard operation failed: {0}")]
    KeyboardError(String),

    /// Python error / Pythonエラー
    #[error("Python error: {0}")]
    PythonError(String),

    /// IO error / IOエラー
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SikulixError>;

/// Represents a rectangular region on the screen
/// 画面上の矩形領域を表します
///
/// Used for defining areas for screen capture, image search, and click targets.
/// スクリーンキャプチャ、画像検索、クリック対象の領域定義に使用します。
///
/// # Example / 使用例
///
/// ```
/// use sikulix_core::Region;
///
/// let region = Region::new(100, 100, 200, 150);
/// let (cx, cy) = region.center();
/// assert!(region.contains(150, 150));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    /// X coordinate of the top-left corner / 左上隅のX座標
    pub x: i32,
    /// Y coordinate of the top-left corner / 左上隅のY座標
    pub y: i32,
    /// Width of the region / 領域の幅
    pub width: u32,
    /// Height of the region / 領域の高さ
    pub height: u32,
}

impl Region {
    /// Create a new region with the given position and size
    /// 指定された位置とサイズで新しい領域を作成
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a region from two corner points
    /// 2つの角の座標から領域を作成
    pub fn from_corners(x1: i32, y1: i32, x2: i32, y2: i32) -> Self {
        let (min_x, max_x) = if x1 <= x2 { (x1, x2) } else { (x2, x1) };
        let (min_y, max_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        Self {
            x: min_x,
            y: min_y,
            width: (max_x - min_x) as u32,
            height: (max_y - min_y) as u32,
        }
    }

    /// Get the center point of the region
    /// 領域の中心座標を取得
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + (self.width as i32 / 2),
            self.y + (self.height as i32 / 2),
        )
    }

    /// Get the top-left corner point
    /// 左上隅の座標を取得
    pub fn top_left(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    /// Get the bottom-right corner point
    /// 右下隅の座標を取得
    pub fn bottom_right(&self) -> (i32, i32) {
        (self.x + self.width as i32, self.y + self.height as i32)
    }

    /// Get the area of the region in pixels
    /// 領域の面積（ピクセル）を取得
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Check if a point is inside the region
    /// 指定座標が領域内にあるか確認
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }

    /// Check if this region intersects with another region
    /// この領域が別の領域と交差するか確認
    pub fn intersects(&self, other: &Region) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }

    /// Create a new region by offsetting this region
    /// 領域を移動した新しい領域を作成
    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..*self
        }
    }

    /// Create a new region by expanding this region on all sides
    /// 全方向に拡張した新しい領域を作成
    pub fn expand(&self, amount: i32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: (self.width as i32 + 2 * amount).max(0) as u32,
            height: (self.height as i32 + 2 * amount).max(0) as u32,
        }
    }

    /// Get the intersection of this region with another
    /// 別の領域との交差領域を取得
    pub fn intersection(&self, other: &Region) -> Option<Self> {
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.width as i32).min(other.x + other.width as i32);
        let y2 = (self.y + self.height as i32).min(other.y + other.height as i32);

        if x1 < x2 && y1 < y2 {
            Some(Self {
                x: x1,
                y: y1,
                width: (x2 - x1) as u32,
                height: (y2 - y1) as u32,
            })
        } else {
            None
        }
    }
}

/// Match result from image search
/// 画像検索のマッチ結果
///
/// Contains the region where a pattern was found and the confidence score.
/// パターンが見つかった領域と信頼度スコアを含みます。
///
/// # Example / 使用例
///
/// ```
/// use sikulix_core::{Match, Region};
///
/// let m = Match::new(Region::new(100, 100, 50, 50), 0.95);
/// assert!(m.is_good_match(0.9));
/// let (cx, cy) = m.center();
/// ```
#[derive(Debug, Clone)]
pub struct Match {
    /// The region where the match was found / マッチが見つかった領域
    pub region: Region,
    /// Confidence score (0.0 - 1.0) / 信頼度スコア（0.0〜1.0）
    pub score: f64,
}

impl Match {
    /// Create a new match result
    /// 新しいマッチ結果を作成
    pub fn new(region: Region, score: f64) -> Self {
        Self { region, score }
    }

    /// Get the center point of the match
    /// マッチの中心座標を取得
    pub fn center(&self) -> (i32, i32) {
        self.region.center()
    }

    /// Get the target point (center + offset if any)
    /// ターゲット座標を取得（中心+オフセット）
    pub fn target(&self) -> (i32, i32) {
        self.region.center()
    }

    /// Check if this match meets a minimum score threshold
    /// マッチが最小スコア閾値を満たすか確認
    pub fn is_good_match(&self, threshold: f64) -> bool {
        self.score >= threshold
    }

    /// Get the match score as a percentage string
    /// マッチスコアをパーセント文字列で取得
    pub fn score_percent(&self) -> String {
        format!("{:.1}%", self.score * 100.0)
    }
}

/// Pattern for image matching
/// 画像マッチング用パターン
///
/// Represents an image template to search for on the screen.
/// Supports similarity threshold and target offset configuration.
/// 画面上で検索する画像テンプレートを表します。
/// 類似度閾値とターゲットオフセットの設定をサポートします。
///
/// # Example / 使用例
///
/// ```
/// use sikulix_core::Pattern;
///
/// // Create pattern with custom similarity
/// // カスタム類似度でパターンを作成
/// let pattern = Pattern::new(vec![/* PNG data */])
///     .similar(0.9)
///     .target_offset(10, 5);
/// ```
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Image data (PNG bytes) / 画像データ（PNGバイト列）
    pub image_data: Vec<u8>,
    /// Similarity threshold (0.0 - 1.0), default: 0.7 / 類似度閾値、デフォルト: 0.7
    pub similarity: f64,
    /// Target offset from center for click operations / クリック操作時の中心からのオフセット
    pub target_offset: (i32, i32),
}

impl Default for Pattern {
    fn default() -> Self {
        Self {
            image_data: Vec::new(),
            similarity: 0.7,
            target_offset: (0, 0),
        }
    }
}

impl Pattern {
    /// Create a new pattern from image data
    /// 画像データから新しいパターンを作成
    pub fn new(image_data: Vec<u8>) -> Self {
        Self {
            image_data,
            ..Default::default()
        }
    }

    /// Load a pattern from a file path
    /// ファイルパスからパターンを読み込み
    pub fn from_file(path: &str) -> Result<Self> {
        let image_data = std::fs::read(path)?;
        Ok(Self::new(image_data))
    }

    /// Set similarity threshold (builder pattern)
    /// 類似度閾値を設定（ビルダーパターン）
    pub fn similar(mut self, similarity: f64) -> Self {
        self.similarity = similarity.clamp(0.0, 1.0);
        self
    }

    /// Set target offset (builder pattern)
    /// ターゲットオフセットを設定（ビルダーパターン）
    pub fn target_offset(mut self, x: i32, y: i32) -> Self {
        self.target_offset = (x, y);
        self
    }

    /// Check if the pattern has valid image data
    /// パターンが有効な画像データを持つか確認
    pub fn is_valid(&self) -> bool {
        !self.image_data.is_empty()
    }

    /// Get the size of the image data in bytes
    /// 画像データのサイズ（バイト）を取得
    pub fn data_size(&self) -> usize {
        self.image_data.len()
    }
}

/// Library version / ライブラリバージョン
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library
/// ライブラリを初期化
pub fn init() {
    env_logger::init();
    log::info!("SikuliX Core {} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_center() {
        let region = Region::new(100, 100, 50, 50);
        assert_eq!(region.center(), (125, 125));
    }

    #[test]
    fn test_region_contains() {
        let region = Region::new(100, 100, 50, 50);
        assert!(region.contains(125, 125));
        assert!(!region.contains(50, 50));
    }

    #[test]
    fn test_region_from_corners() {
        let region = Region::from_corners(150, 200, 100, 100);
        assert_eq!(region.x, 100);
        assert_eq!(region.y, 100);
        assert_eq!(region.width, 50);
        assert_eq!(region.height, 100);
    }

    #[test]
    fn test_region_intersects() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(50, 50, 100, 100);
        let r3 = Region::new(200, 200, 50, 50);
        assert!(r1.intersects(&r2));
        assert!(!r1.intersects(&r3));
    }

    #[test]
    fn test_region_intersection() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(50, 50, 100, 100);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection.x, 50);
        assert_eq!(intersection.y, 50);
        assert_eq!(intersection.width, 50);
        assert_eq!(intersection.height, 50);
    }

    #[test]
    fn test_region_offset() {
        let region = Region::new(100, 100, 50, 50);
        let offset = region.offset(10, -10);
        assert_eq!(offset.x, 110);
        assert_eq!(offset.y, 90);
    }

    #[test]
    fn test_region_expand() {
        let region = Region::new(100, 100, 50, 50);
        let expanded = region.expand(10);
        assert_eq!(expanded.x, 90);
        assert_eq!(expanded.y, 90);
        assert_eq!(expanded.width, 70);
        assert_eq!(expanded.height, 70);
    }

    #[test]
    fn test_region_area() {
        let region = Region::new(0, 0, 100, 50);
        assert_eq!(region.area(), 5000);
    }

    #[test]
    fn test_pattern_similar() {
        let pattern = Pattern::new(vec![]).similar(0.9);
        assert!((pattern.similarity - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pattern_is_valid() {
        let empty = Pattern::new(vec![]);
        let valid = Pattern::new(vec![1, 2, 3]);
        assert!(!empty.is_valid());
        assert!(valid.is_valid());
    }

    #[test]
    fn test_match_is_good_match() {
        let m = Match::new(Region::new(0, 0, 10, 10), 0.85);
        assert!(m.is_good_match(0.8));
        assert!(!m.is_good_match(0.9));
    }

    #[test]
    fn test_match_score_percent() {
        let m = Match::new(Region::new(0, 0, 10, 10), 0.956);
        assert_eq!(m.score_percent(), "95.6%");
    }
}
