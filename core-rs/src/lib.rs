//! SikuliX Core Library - Next Generation
//!
//! A high-performance, low-memory GUI automation library written in Rust.
//!
//! # Features
//! - Image matching with template matching algorithm
//! - OCR (Optical Character Recognition)
//! - Cross-platform screen capture
//! - Mouse and keyboard control
//! - Python scripting support (Python 2/3 dual runtime)

pub mod image;
pub mod python;
pub mod screen;

// Re-export commonly used types
pub use image::ImageMatcher;
pub use python::{PythonVersion, SyntaxAnalyzer};
pub use screen::{Key, Keyboard, Mouse, Screen};

#[cfg(feature = "python")]
pub use python::PythonRuntime;

use thiserror::Error;

/// SikuliX Core Error Types
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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, SikulixError>;

/// Represents a region on the screen
#[derive(Debug, Clone, Copy, PartialEq)]
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

    /// Get the center point of the region
    pub fn center(&self) -> (i32, i32) {
        (
            self.x + (self.width as i32 / 2),
            self.y + (self.height as i32 / 2),
        )
    }

    /// Check if a point is inside the region
    pub fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x
            && x < self.x + self.width as i32
            && y >= self.y
            && y < self.y + self.height as i32
    }
}

/// Match result from image search
#[derive(Debug, Clone)]
pub struct Match {
    pub region: Region,
    pub score: f64,
}

impl Match {
    pub fn new(region: Region, score: f64) -> Self {
        Self { region, score }
    }

    /// Get the center point of the match
    pub fn center(&self) -> (i32, i32) {
        self.region.center()
    }
}

/// Pattern for image matching
#[derive(Debug, Clone)]
pub struct Pattern {
    /// Image data (PNG bytes)
    pub image_data: Vec<u8>,
    /// Similarity threshold (0.0 - 1.0)
    pub similarity: f64,
    /// Target offset from center
    pub target_offset: (i32, i32),
}

impl Pattern {
    pub fn new(image_data: Vec<u8>) -> Self {
        Self {
            image_data,
            similarity: 0.7,
            target_offset: (0, 0),
        }
    }

    /// Set similarity threshold
    pub fn similar(mut self, similarity: f64) -> Self {
        self.similarity = similarity.clamp(0.0, 1.0);
        self
    }

    /// Set target offset
    pub fn target_offset(mut self, x: i32, y: i32) -> Self {
        self.target_offset = (x, y);
        self
    }
}

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the library
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
    fn test_pattern_similar() {
        let pattern = Pattern::new(vec![]).similar(0.9);
        assert!((pattern.similarity - 0.9).abs() < f64::EPSILON);
    }
}
