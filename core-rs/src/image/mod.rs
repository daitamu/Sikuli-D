//! Image processing and matching module

mod matcher;

pub use matcher::*;

use crate::{Region, Result, SikulixError};
use image::DynamicImage;

/// Load an image from file
pub fn load_image(path: &str) -> Result<DynamicImage> {
    image::open(path).map_err(|e| SikulixError::ImageLoadError(e.to_string()))
}

/// Load an image from bytes
pub fn load_image_from_bytes(data: &[u8]) -> Result<DynamicImage> {
    image::load_from_memory(data).map_err(|e| SikulixError::ImageLoadError(e.to_string()))
}

/// Save an image to file
pub fn save_image(img: &DynamicImage, path: &str) -> Result<()> {
    img.save(path).map_err(|e| SikulixError::ImageLoadError(e.to_string()))
}

/// Crop a region from an image
pub fn crop(img: &DynamicImage, region: &Region) -> DynamicImage {
    img.crop_imm(
        region.x as u32,
        region.y as u32,
        region.width,
        region.height,
    )
}
