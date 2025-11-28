//! Coordinate conversion utilities for DPI scaling
//! DPIスケーリング用の座標変換ユーティリティ
//!
//! Design principle: Logical pixels are the master data.
//! Convert to physical only when needed for OS operations.
//! No physical-to-logical conversion to avoid rounding errors.
//!
//! 設計原則: 論理ピクセルがマスターデータ。
//! OS操作時のみ物理ピクセルに変換。
//! 丸め誤差回避のため、物理→論理変換は提供しない。

use crate::Region;
use image::DynamicImage;

/// Convert logical pixel coordinate to physical pixel coordinate
/// 論理ピクセル座標を物理ピクセル座標に変換
///
/// # Arguments / 引数
/// * `logical` - Logical pixel coordinate / 論理ピクセル座標
/// * `scale_factor` - DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%) / DPIスケールファクター
///
/// # Returns / 戻り値
/// Physical pixel coordinate / 物理ピクセル座標
///
/// # Example / 使用例
/// ```
/// use sikulid::screen::coordinates::logical_to_physical;
///
/// assert_eq!(logical_to_physical(100, 1.5), 150);
/// assert_eq!(logical_to_physical(-100, 1.5), -150);
/// ```
pub fn logical_to_physical(logical: i32, scale_factor: f64) -> i32 {
    (logical as f64 * scale_factor).round() as i32
}

/// Convert logical Region to physical Region
/// 論理Regionを物理Regionに変換
///
/// # Arguments / 引数
/// * `region` - Region in logical pixels / 論理ピクセルのRegion
/// * `scale_factor` - DPI scale factor / DPIスケールファクター
///
/// # Returns / 戻り値
/// Region in physical pixels / 物理ピクセルのRegion
///
/// # Example / 使用例
/// ```
/// use sikulid::{Region, screen::coordinates::region_to_physical};
///
/// let logical = Region::new(100, 200, 300, 400);
/// let physical = region_to_physical(&logical, 1.5);
/// assert_eq!(physical.x, 150);
/// assert_eq!(physical.y, 300);
/// assert_eq!(physical.width, 450);
/// assert_eq!(physical.height, 600);
/// ```
pub fn region_to_physical(region: &Region, scale_factor: f64) -> Region {
    Region::new(
        logical_to_physical(region.x, scale_factor),
        logical_to_physical(region.y, scale_factor),
        (region.width as f64 * scale_factor).round() as u32,
        (region.height as f64 * scale_factor).round() as u32,
    )
}

/// Resize image from physical dimensions to logical dimensions
/// 物理サイズの画像を論理サイズにリサイズ
///
/// This is used when capturing screen at physical resolution
/// and needing to match templates at logical resolution.
/// 物理解像度でキャプチャした画面を論理解像度のテンプレートと
/// マッチングする際に使用します。
///
/// # Arguments / 引数
/// * `image` - Image captured at physical resolution / 物理解像度でキャプチャした画像
/// * `scale_factor` - DPI scale factor / DPIスケールファクター
///
/// # Returns / 戻り値
/// Image resized to logical dimensions / 論理サイズにリサイズした画像
///
/// # Example / 使用例
/// ```ignore
/// // Physical 3840x2160 at 150% → Logical 2560x1440
/// let logical_image = resize_to_logical(&physical_image, 1.5);
/// ```
pub fn resize_to_logical(image: &DynamicImage, scale_factor: f64) -> DynamicImage {
    // No scaling needed if scale_factor is 1.0
    if (scale_factor - 1.0).abs() < 0.001 {
        return image.clone();
    }

    let logical_width = (image.width() as f64 / scale_factor).round() as u32;
    let logical_height = (image.height() as f64 / scale_factor).round() as u32;

    // Use Lanczos3 filter for high quality downscaling
    image.resize_exact(
        logical_width,
        logical_height,
        image::imageops::FilterType::Lanczos3,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // === 座標変換テスト (論理→物理のみ) ===

    #[test]
    fn test_logical_to_physical_100_percent() {
        assert_eq!(logical_to_physical(100, 1.0), 100);
        assert_eq!(logical_to_physical(0, 1.0), 0);
        assert_eq!(logical_to_physical(-100, 1.0), -100);
    }

    #[test]
    fn test_logical_to_physical_150_percent() {
        assert_eq!(logical_to_physical(100, 1.5), 150);
        assert_eq!(logical_to_physical(200, 1.5), 300);
        assert_eq!(logical_to_physical(-100, 1.5), -150);
    }

    #[test]
    fn test_logical_to_physical_200_percent() {
        assert_eq!(logical_to_physical(100, 2.0), 200);
        assert_eq!(logical_to_physical(50, 2.0), 100);
    }

    #[test]
    fn test_logical_to_physical_125_percent() {
        // 125% scaling (common on laptops)
        assert_eq!(logical_to_physical(100, 1.25), 125);
        assert_eq!(logical_to_physical(80, 1.25), 100);
    }

    #[test]
    fn test_region_to_physical() {
        let region = Region::new(100, 200, 300, 400);
        let physical = region_to_physical(&region, 1.5);
        assert_eq!(physical.x, 150);
        assert_eq!(physical.y, 300);
        assert_eq!(physical.width, 450);
        assert_eq!(physical.height, 600);
    }

    #[test]
    fn test_region_to_physical_negative_coords() {
        // Left monitor scenario (negative x)
        let region = Region::new(-1920, 0, 1920, 1080);
        let physical = region_to_physical(&region, 1.5);
        assert_eq!(physical.x, -2880);
        assert_eq!(physical.y, 0);
        assert_eq!(physical.width, 2880);
        assert_eq!(physical.height, 1620);
    }

    #[test]
    fn test_negative_coordinates() {
        // Left monitor scenario
        let x = -1920;
        let physical = logical_to_physical(x, 1.0);
        assert_eq!(physical, -1920);

        // 150% scaling
        let physical_150 = logical_to_physical(x, 1.5);
        assert_eq!(physical_150, -2880);
    }

    // === 画像リサイズテスト ===

    #[test]
    fn test_resize_to_logical_no_scaling() {
        // scale_factor 1.0 should return clone without resize
        let img = DynamicImage::new_rgb8(1920, 1080);
        let result = resize_to_logical(&img, 1.0);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }

    #[test]
    fn test_resize_to_logical_150_percent() {
        // Physical 3840x2160 → Logical 2560x1440 (150%)
        let img = DynamicImage::new_rgb8(3840, 2160);
        let result = resize_to_logical(&img, 1.5);
        assert_eq!(result.width(), 2560);
        assert_eq!(result.height(), 1440);
    }

    #[test]
    fn test_resize_to_logical_200_percent() {
        // Physical 3840x2160 → Logical 1920x1080 (200%)
        let img = DynamicImage::new_rgb8(3840, 2160);
        let result = resize_to_logical(&img, 2.0);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }

    #[test]
    fn test_resize_to_logical_125_percent() {
        // Physical 2400x1350 → Logical 1920x1080 (125%)
        let img = DynamicImage::new_rgb8(2400, 1350);
        let result = resize_to_logical(&img, 1.25);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }
}
