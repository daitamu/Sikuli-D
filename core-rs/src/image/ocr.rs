//! OCR (Optical Character Recognition) implementation
//!
//! This module provides OCR functionality using Tesseract via the leptess crate.
//! The OCR feature is optional and requires the `ocr` feature flag to be enabled.

use crate::{Region, Result, SikulixError};
use image::DynamicImage;

/// Supported OCR languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrLanguage {
    /// English
    English,
    /// Japanese
    Japanese,
    /// Chinese Simplified
    ChineseSimplified,
    /// Chinese Traditional
    ChineseTraditional,
    /// Korean
    Korean,
    /// German
    German,
    /// French
    French,
    /// Spanish
    Spanish,
    /// Custom language code
    Custom,
}

impl OcrLanguage {
    /// Get the Tesseract language code
    pub fn code(&self) -> &'static str {
        match self {
            OcrLanguage::English => "eng",
            OcrLanguage::Japanese => "jpn",
            OcrLanguage::ChineseSimplified => "chi_sim",
            OcrLanguage::ChineseTraditional => "chi_tra",
            OcrLanguage::Korean => "kor",
            OcrLanguage::German => "deu",
            OcrLanguage::French => "fra",
            OcrLanguage::Spanish => "spa",
            OcrLanguage::Custom => "eng", // Default fallback
        }
    }
}

/// OCR result containing recognized text and confidence
#[derive(Debug, Clone)]
pub struct OcrResult {
    /// Recognized text
    pub text: String,
    /// Recognition confidence (0.0 - 1.0)
    pub confidence: f64,
    /// Bounding box of the recognized text (if available)
    pub region: Option<Region>,
}

impl OcrResult {
    /// Create a new OCR result
    pub fn new(text: String, confidence: f64) -> Self {
        Self {
            text,
            confidence,
            region: None,
        }
    }

    /// Create an OCR result with a bounding region
    pub fn with_region(text: String, confidence: f64, region: Region) -> Self {
        Self {
            text,
            confidence,
            region: Some(region),
        }
    }
}

/// OCR configuration options
#[derive(Debug, Clone)]
pub struct OcrConfig {
    /// Primary language for recognition
    pub language: OcrLanguage,
    /// Custom language code (used when language is Custom)
    pub custom_language: Option<String>,
    /// Path to Tesseract data files (tessdata)
    pub tessdata_path: Option<String>,
    /// Page segmentation mode (PSM)
    pub page_segmentation_mode: i32,
    /// Whitelist of characters to recognize (empty = all)
    pub whitelist: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            language: OcrLanguage::English,
            custom_language: None,
            tessdata_path: None,
            page_segmentation_mode: 3, // PSM_AUTO
            whitelist: String::new(),
        }
    }
}

impl OcrConfig {
    /// Create a new OCR config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the recognition language
    pub fn with_language(mut self, language: OcrLanguage) -> Self {
        self.language = language;
        self
    }

    /// Set a custom language code
    pub fn with_custom_language(mut self, code: &str) -> Self {
        self.language = OcrLanguage::Custom;
        self.custom_language = Some(code.to_string());
        self
    }

    /// Set the tessdata path
    pub fn with_tessdata_path(mut self, path: &str) -> Self {
        self.tessdata_path = Some(path.to_string());
        self
    }

    /// Set the page segmentation mode
    pub fn with_page_segmentation_mode(mut self, mode: i32) -> Self {
        self.page_segmentation_mode = mode;
        self
    }

    /// Set character whitelist
    pub fn with_whitelist(mut self, whitelist: &str) -> Self {
        self.whitelist = whitelist.to_string();
        self
    }

    /// Get the effective language code
    pub fn get_language_code(&self) -> &str {
        if let Some(ref custom) = self.custom_language {
            custom.as_str()
        } else {
            self.language.code()
        }
    }
}

/// OCR engine for text recognition
pub struct OcrEngine {
    config: OcrConfig,
}

impl Default for OcrEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl OcrEngine {
    /// Create a new OCR engine with default configuration
    pub fn new() -> Self {
        Self {
            config: OcrConfig::default(),
        }
    }

    /// Create an OCR engine with custom configuration
    pub fn with_config(config: OcrConfig) -> Self {
        Self { config }
    }

    /// Set the configuration
    pub fn set_config(&mut self, config: OcrConfig) {
        self.config = config;
    }

    /// Get the current configuration
    pub fn config(&self) -> &OcrConfig {
        &self.config
    }

    /// Recognize text from an image
    #[cfg(feature = "ocr")]
    pub fn recognize(&self, image: &DynamicImage) -> Result<OcrResult> {
        use leptess::LepTess;

        // Convert image to grayscale
        let gray = image.to_luma8();
        let (width, height) = gray.dimensions();
        let raw_data = gray.into_raw();

        // Get tessdata path
        let tessdata = self.config.tessdata_path.as_deref();

        // Create Tesseract instance
        let mut lt = LepTess::new(tessdata, self.config.get_language_code()).map_err(|e| {
            SikulixError::OcrError(format!("Failed to initialize Tesseract: {}", e))
        })?;

        // Set page segmentation mode
        lt.set_variable(
            leptess::Variable::PageSegMode,
            &self.config.page_segmentation_mode.to_string(),
        )
        .map_err(|e| SikulixError::OcrError(format!("Failed to set PSM: {}", e)))?;

        // Set character whitelist if specified
        if !self.config.whitelist.is_empty() {
            lt.set_variable(
                leptess::Variable::TesseditCharWhitelist,
                &self.config.whitelist,
            )
            .map_err(|e| SikulixError::OcrError(format!("Failed to set whitelist: {}", e)))?;
        }

        // Set image data (8-bit grayscale)
        lt.set_image_from_mem(&raw_data, width as i32, height as i32, 1, width as i32)
            .map_err(|e| SikulixError::OcrError(format!("Failed to set image: {}", e)))?;

        // Perform recognition
        let text = lt
            .get_utf8_text()
            .map_err(|e| SikulixError::OcrError(format!("Failed to recognize text: {}", e)))?;

        // Get mean confidence
        let confidence = lt.mean_text_conf() as f64 / 100.0;

        Ok(OcrResult::new(text.trim().to_string(), confidence))
    }

    /// Recognize text from an image (stub when OCR feature is disabled)
    #[cfg(not(feature = "ocr"))]
    pub fn recognize(&self, _image: &DynamicImage) -> Result<OcrResult> {
        Err(SikulixError::OcrError(
            "OCR feature is not enabled. Build with --features ocr".to_string(),
        ))
    }

    /// Recognize text from a specific region of an image
    pub fn recognize_region(&self, image: &DynamicImage, region: &Region) -> Result<OcrResult> {
        // Crop the image to the specified region
        let cropped = image.crop_imm(
            region.x as u32,
            region.y as u32,
            region.width,
            region.height,
        );

        let mut result = self.recognize(&cropped)?;
        result.region = Some(*region);
        Ok(result)
    }
}

/// Convenience function to recognize text from an image
pub fn read_text(image: &DynamicImage) -> Result<String> {
    let engine = OcrEngine::new();
    Ok(engine.recognize(image)?.text)
}

/// Convenience function to recognize text with Japanese language
pub fn read_text_japanese(image: &DynamicImage) -> Result<String> {
    let config = OcrConfig::new().with_language(OcrLanguage::Japanese);
    let engine = OcrEngine::with_config(config);
    Ok(engine.recognize(image)?.text)
}

/// Convenience function to recognize text from a region
pub fn read_text_region(image: &DynamicImage, region: &Region) -> Result<String> {
    let engine = OcrEngine::new();
    Ok(engine.recognize_region(image, region)?.text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_config_default() {
        let config = OcrConfig::default();
        assert_eq!(config.language, OcrLanguage::English);
        assert_eq!(config.get_language_code(), "eng");
    }

    #[test]
    fn test_ocr_config_japanese() {
        let config = OcrConfig::new().with_language(OcrLanguage::Japanese);
        assert_eq!(config.get_language_code(), "jpn");
    }

    #[test]
    fn test_ocr_config_custom_language() {
        let config = OcrConfig::new().with_custom_language("jpn+eng");
        assert_eq!(config.get_language_code(), "jpn+eng");
    }

    #[test]
    fn test_ocr_result() {
        let result = OcrResult::new("Hello".to_string(), 0.95);
        assert_eq!(result.text, "Hello");
        assert!((result.confidence - 0.95).abs() < f64::EPSILON);
        assert!(result.region.is_none());
    }

    #[test]
    fn test_ocr_result_with_region() {
        let region = Region::new(10, 20, 100, 50);
        let result = OcrResult::with_region("Test".to_string(), 0.85, region);
        assert!(result.region.is_some());
    }
}
