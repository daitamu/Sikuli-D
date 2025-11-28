//! SikuliX Core Library - Next Generation

pub mod app;
pub mod color;
pub mod debug;
pub mod highlight;
pub mod image;
pub mod input;
pub mod location;
pub mod observer;
pub mod plugin;
pub mod python;
pub mod screen;
pub mod settings;
pub mod timeout;

pub use app::App;
pub use color::{get_color, save_region_capture, save_screen_capture};
pub use highlight::Highlight;
pub use image::ocr::{read_text, read_text_japanese, read_text_region};
pub use image::ImageMatcher;
pub use image::{OcrConfig, OcrEngine, OcrLanguage, OcrResult};
pub use location::Location;
pub use observer::Observer;
pub use python::{PythonVersion, SyntaxAnalyzer};
pub use screen::{Key, Keyboard, Mouse, Screen};
pub use timeout::{
    wait_for_condition, wait_for_condition_with_cancel, with_timeout, with_timeout_and_cancel,
    CancellationToken, DefaultTimeouts, TimeoutGuard,
};

#[cfg(feature = "python")]
pub use python::PythonRuntime;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SikulixError {
    #[error("Image not found on screen")]
    ImageNotFound,
    #[error("Image loading failed: {0}")]
    ImageLoadError(String),
    #[error("Screen capture failed: {0}")]
    ScreenCaptureError(String),
    #[error("OCR failed: {0}")]
    OcrError(String),
    #[error("Mouse operation failed: {0}")]
    MouseError(String),
    #[error("Keyboard operation failed: {0}")]
    KeyboardError(String),
    #[error("Python error: {0}")]
    PythonError(String),
    #[error("Platform error: {0}")]
    PlatformError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("FindFailed: {pattern_name} not found within {timeout_secs}s")]
    FindFailed {
        pattern_name: String,
        timeout_secs: f64,
    },
    #[error("Operation timed out after {timeout_secs}s: {operation}")]
    Timeout {
        operation: String,
        timeout_secs: f64,
    },
    #[error("Operation cancelled: {0}")]
    Cancelled(String),
    #[error("Wait condition not met within {timeout_secs}s: {condition}")]
    WaitTimeout {
        condition: String,
        timeout_secs: f64,
    },
    #[error("Script execution timed out after {timeout_secs}s: {script}")]
    ScriptTimeout { script: String, timeout_secs: f64 },
}

pub type Result<T> = std::result::Result<T, SikulixError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DpiMode {
    #[default]
    Physical,
    Logical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelFormat {
    #[default]
    Bgra,
    Rgba,
    Bgr,
    Rgb,
    Bgr24,
    Rgba32,
}

#[derive(Debug, Clone)]
pub struct RawCapture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub format: PixelFormat,
    pub dpi_scale: f64,
    pub dpi_mode: DpiMode,
}

impl RawCapture {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: PixelFormat) -> Self {
        let bytes_per_pixel = match format {
            PixelFormat::Bgra | PixelFormat::Rgba | PixelFormat::Rgba32 => 4,
            PixelFormat::Bgr | PixelFormat::Rgb | PixelFormat::Bgr24 => 3,
        };
        Self {
            data,
            width,
            height,
            stride: width * bytes_per_pixel,
            format,
            dpi_scale: 1.0,
            dpi_mode: DpiMode::Physical,
        }
    }
}

/// Color with RGBA components
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
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
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + (self.width as i32 / 2),
            self.y + (self.height as i32 / 2),
        )
    }
    pub fn top_left(&self) -> (i32, i32) {
        (self.x, self.y)
    }
    pub fn bottom_right(&self) -> (i32, i32) {
        (self.x + self.width as i32, self.y + self.height as i32)
    }
    pub fn area(&self) -> u64 {
        self.width as u64 * self.height as u64
    }
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }
    pub fn intersects(&self, other: &Region) -> bool {
        self.x < other.x + other.width as i32
            && self.x + self.width as i32 > other.x
            && self.y < other.y + other.height as i32
            && self.y + self.height as i32 > other.y
    }
    pub fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
            ..*self
        }
    }
    pub fn expand(&self, amount: i32) -> Self {
        Self {
            x: self.x - amount,
            y: self.y - amount,
            width: (self.width as i32 + 2 * amount).max(0) as u32,
            height: (self.height as i32 + 2 * amount).max(0) as u32,
        }
    }
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

#[derive(Debug, Clone)]
pub struct Match {
    pub region: Region,
    pub score: f64,
}

impl Match {
    pub fn new(region: Region, score: f64) -> Self {
        Self { region, score }
    }
    pub fn center(&self) -> (i32, i32) {
        self.region.center()
    }
    pub fn target(&self) -> (i32, i32) {
        self.region.center()
    }
    pub fn is_good_match(&self, threshold: f64) -> bool {
        self.score >= threshold
    }
    pub fn score_percent(&self) -> String {
        format!("{:.1}%", self.score * 100.0)
    }
    pub fn highlight(&self) -> Result<()> {
        self.highlight_with_duration(2.0)
    }
    pub fn highlight_with_duration(&self, seconds: f64) -> Result<()> {
        let hl = Highlight::new(self.region);
        let _ = hl.show_for(seconds);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub image_data: Vec<u8>,
    pub similarity: f64,
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
    pub fn new(image_data: Vec<u8>) -> Self {
        Self {
            image_data,
            ..Default::default()
        }
    }
    pub fn from_file(path: &str) -> Result<Self> {
        Ok(Self::new(std::fs::read(path)?))
    }
    pub fn similar(mut self, similarity: f64) -> Self {
        self.similarity = similarity.clamp(0.0, 1.0);
        self
    }
    pub fn target_offset(mut self, x: i32, y: i32) -> Self {
        self.target_offset = (x, y);
        self
    }
    pub fn is_valid(&self) -> bool {
        !self.image_data.is_empty()
    }
    pub fn data_size(&self) -> usize {
        self.image_data.len()
    }
}

pub fn sleep(seconds: f64) {
    std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
}
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub fn init() {
    env_logger::init();
    log::info!("SikuliX Core {} initialized", VERSION);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==============================
    // Region Tests / Region テスト
    // ==============================

    #[test]
    fn test_region_new() {
        let r = Region::new(10, 20, 100, 200);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 200);
    }

    #[test]
    fn test_region_from_corners() {
        let r = Region::from_corners(10, 20, 110, 220);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 200);
    }

    #[test]
    fn test_region_from_corners_reversed() {
        // Test with reversed coordinates
        let r = Region::from_corners(110, 220, 10, 20);
        assert_eq!(r.x, 10);
        assert_eq!(r.y, 20);
        assert_eq!(r.width, 100);
        assert_eq!(r.height, 200);
    }

    #[test]
    fn test_region_center() {
        assert_eq!(Region::new(100, 100, 50, 50).center(), (125, 125));
        assert_eq!(Region::new(0, 0, 100, 100).center(), (50, 50));
        assert_eq!(Region::new(-50, -50, 100, 100).center(), (0, 0));
    }

    #[test]
    fn test_region_top_left() {
        let r = Region::new(10, 20, 100, 200);
        assert_eq!(r.top_left(), (10, 20));
    }

    #[test]
    fn test_region_bottom_right() {
        let r = Region::new(10, 20, 100, 200);
        assert_eq!(r.bottom_right(), (110, 220));
    }

    #[test]
    fn test_region_area() {
        let r = Region::new(0, 0, 100, 200);
        assert_eq!(r.area(), 20000);
    }

    #[test]
    fn test_region_contains_inside() {
        let r = Region::new(100, 100, 50, 50);
        assert!(r.contains(125, 125));
        assert!(r.contains(100, 100)); // Top-left corner
        assert!(r.contains(149, 149)); // Bottom-right corner (inside)
    }

    #[test]
    fn test_region_contains_outside() {
        let r = Region::new(100, 100, 50, 50);
        assert!(!r.contains(50, 50));
        assert!(!r.contains(150, 150)); // Bottom-right corner (outside)
        assert!(!r.contains(99, 100)); // Just left
        assert!(!r.contains(100, 99)); // Just above
    }

    #[test]
    fn test_region_intersects_overlap() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(50, 50, 100, 100);
        assert!(r1.intersects(&r2));
        assert!(r2.intersects(&r1));
    }

    #[test]
    fn test_region_intersects_no_overlap() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(200, 200, 100, 100);
        assert!(!r1.intersects(&r2));
        assert!(!r2.intersects(&r1));
    }

    #[test]
    fn test_region_intersects_touching() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(100, 0, 100, 100);
        // Touching at edge, should not intersect
        assert!(!r1.intersects(&r2));
    }

    #[test]
    fn test_region_intersection_overlap() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(50, 50, 100, 100);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection, Region::new(50, 50, 50, 50));
    }

    #[test]
    fn test_region_intersection_no_overlap() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(200, 200, 100, 100);
        assert!(r1.intersection(&r2).is_none());
    }

    #[test]
    fn test_region_intersection_full_overlap() {
        let r1 = Region::new(0, 0, 100, 100);
        let r2 = Region::new(25, 25, 50, 50);
        let intersection = r1.intersection(&r2).unwrap();
        assert_eq!(intersection, r2);
    }

    #[test]
    fn test_region_offset() {
        let r = Region::new(10, 20, 100, 200);
        let r2 = r.offset(5, 10);
        assert_eq!(r2.x, 15);
        assert_eq!(r2.y, 30);
        assert_eq!(r2.width, 100);
        assert_eq!(r2.height, 200);
    }

    #[test]
    fn test_region_offset_negative() {
        let r = Region::new(100, 100, 50, 50);
        let r2 = r.offset(-50, -50);
        assert_eq!(r2.x, 50);
        assert_eq!(r2.y, 50);
    }

    #[test]
    fn test_region_expand_positive() {
        let r = Region::new(100, 100, 50, 50);
        let r2 = r.expand(10);
        assert_eq!(r2.x, 90);
        assert_eq!(r2.y, 90);
        assert_eq!(r2.width, 70);
        assert_eq!(r2.height, 70);
    }

    #[test]
    fn test_region_expand_negative() {
        let r = Region::new(100, 100, 50, 50);
        let r2 = r.expand(-10);
        assert_eq!(r2.x, 110);
        assert_eq!(r2.y, 110);
        assert_eq!(r2.width, 30);
        assert_eq!(r2.height, 30);
    }

    #[test]
    fn test_region_expand_too_much_negative() {
        let r = Region::new(100, 100, 50, 50);
        let r2 = r.expand(-30);
        // Should clamp to 0
        assert_eq!(r2.width, 0);
        assert_eq!(r2.height, 0);
    }

    // ==============================
    // Pattern Tests / Pattern テスト
    // ==============================

    #[test]
    fn test_pattern_new() {
        let data = vec![1, 2, 3, 4];
        let p = Pattern::new(data.clone());
        assert_eq!(p.image_data, data);
        assert_eq!(p.similarity, 0.7);
        assert_eq!(p.target_offset, (0, 0));
    }

    #[test]
    fn test_pattern_similar() {
        let p = Pattern::new(vec![1, 2, 3]).similar(0.9);
        assert!((p.similarity - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pattern_similar_clamp_high() {
        let p = Pattern::new(vec![1, 2, 3]).similar(1.5);
        assert!((p.similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pattern_similar_clamp_low() {
        let p = Pattern::new(vec![1, 2, 3]).similar(-0.5);
        assert!((p.similarity - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pattern_target_offset() {
        let p = Pattern::new(vec![1, 2, 3]).target_offset(10, 20);
        assert_eq!(p.target_offset, (10, 20));
    }

    #[test]
    fn test_pattern_is_valid_empty() {
        let p = Pattern::new(vec![]);
        assert!(!p.is_valid());
    }

    #[test]
    fn test_pattern_is_valid_non_empty() {
        let p = Pattern::new(vec![1, 2, 3]);
        assert!(p.is_valid());
    }

    #[test]
    fn test_pattern_data_size() {
        let p = Pattern::new(vec![1, 2, 3, 4, 5]);
        assert_eq!(p.data_size(), 5);
    }

    #[test]
    fn test_pattern_from_file_nonexistent() {
        let result = Pattern::from_file("nonexistent_file_12345.png");
        assert!(result.is_err());
    }

    #[test]
    fn test_pattern_builder_chain() {
        let p = Pattern::new(vec![1, 2, 3])
            .similar(0.85)
            .target_offset(5, 10);
        assert!((p.similarity - 0.85).abs() < f64::EPSILON);
        assert_eq!(p.target_offset, (5, 10));
    }

    // ==============================
    // Match Tests / Match テスト
    // ==============================

    #[test]
    fn test_match_new() {
        let r = Region::new(10, 20, 100, 200);
        let m = Match::new(r, 0.85);
        assert_eq!(m.region, r);
        assert_eq!(m.score, 0.85);
    }

    #[test]
    fn test_match_center() {
        let m = Match::new(Region::new(100, 100, 50, 50), 0.9);
        assert_eq!(m.center(), (125, 125));
    }

    #[test]
    fn test_match_target() {
        let m = Match::new(Region::new(100, 100, 50, 50), 0.9);
        assert_eq!(m.target(), (125, 125));
    }

    #[test]
    fn test_match_is_good_match() {
        let m = Match::new(Region::new(0, 0, 10, 10), 0.85);
        assert!(m.is_good_match(0.8));
        assert!(m.is_good_match(0.85));
        assert!(!m.is_good_match(0.9));
    }

    #[test]
    fn test_match_score_percent() {
        let m = Match::new(Region::new(0, 0, 10, 10), 0.8567);
        assert_eq!(m.score_percent(), "85.7%");
    }

    // ==============================
    // Color Tests / Color テスト
    // ==============================

    #[test]
    fn test_color_new() {
        let c = Color::new(255, 128, 64, 255);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 64);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn test_color_to_hex() {
        let c = Color::rgb(255, 128, 64);
        assert_eq!(c.to_hex(), "#FF8040");
    }

    #[test]
    fn test_color_to_hex_black() {
        let c = Color::rgb(0, 0, 0);
        assert_eq!(c.to_hex(), "#000000");
    }

    #[test]
    fn test_color_to_hex_white() {
        let c = Color::rgb(255, 255, 255);
        assert_eq!(c.to_hex(), "#FFFFFF");
    }

    #[test]
    fn test_color_equality() {
        let c1 = Color::new(255, 128, 64, 255);
        let c2 = Color::new(255, 128, 64, 255);
        let c3 = Color::new(255, 128, 64, 128);
        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    // ==============================
    // RawCapture Tests
    // ==============================

    #[test]
    fn test_raw_capture_new_rgba() {
        let data = vec![0u8; 400]; // 10x10 RGBA
        let capture = RawCapture::new(data.clone(), 10, 10, PixelFormat::Rgba);
        assert_eq!(capture.width, 10);
        assert_eq!(capture.height, 10);
        assert_eq!(capture.stride, 40); // 10 * 4
        assert_eq!(capture.data, data);
    }

    #[test]
    fn test_raw_capture_new_rgb() {
        let data = vec![0u8; 300]; // 10x10 RGB
        let capture = RawCapture::new(data.clone(), 10, 10, PixelFormat::Rgb);
        assert_eq!(capture.stride, 30); // 10 * 3
    }

    #[test]
    fn test_raw_capture_default_dpi() {
        let data = vec![0u8; 400];
        let capture = RawCapture::new(data, 10, 10, PixelFormat::Rgba);
        assert_eq!(capture.dpi_scale, 1.0);
        assert_eq!(capture.dpi_mode, DpiMode::Physical);
    }

    // ==============================
    // SikulixError Tests
    // ==============================

    #[test]
    fn test_error_image_not_found() {
        let err = SikulixError::ImageNotFound;
        assert_eq!(err.to_string(), "Image not found on screen");
    }

    #[test]
    fn test_error_find_failed() {
        let err = SikulixError::FindFailed {
            pattern_name: "button.png".to_string(),
            timeout_secs: 3.0,
        };
        assert!(err.to_string().contains("button.png"));
        assert!(err.to_string().contains("3"));
    }

    #[test]
    fn test_error_timeout() {
        let err = SikulixError::Timeout {
            operation: "screen_capture".to_string(),
            timeout_secs: 5.0,
        };
        assert!(err.to_string().contains("screen_capture"));
        assert!(err.to_string().contains("5"));
    }
}
