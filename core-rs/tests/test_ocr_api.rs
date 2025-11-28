//! Test OCR API functionality
//! OCR API機能のテスト
//!
//! This test file verifies the OCR text reading API.
//! このテストファイルはOCRテキスト読み取りAPIを検証します。

use sikulid::{OcrConfig, OcrLanguage, Region};

// Note: text_from_image and text_from_region functions are not yet exported.
// These tests are placeholders for future implementation.
// 注意: text_from_image と text_from_region 関数はまだエクスポートされていません。
// これらのテストは将来の実装のためのプレースホルダーです。

#[test]
#[ignore = "OCR API not yet exported - pending implementation"]
fn test_text_from_image_compiles() {
    // Test that text_from_image function exists and compiles
    // text_from_image関数が存在してコンパイルされることをテスト
    // let img = DynamicImage::ImageRgb8(RgbImage::new(100, 50));
    // let _result = text_from_image(&img);
}

#[test]
#[ignore = "OCR API not yet exported - pending implementation"]
fn test_text_from_region_compiles() {
    // Test that text_from_region function exists and compiles
    // text_from_region関数が存在してコンパイルされることをテスト
    // let region = Region::new(100, 100, 200, 50);
    // let _result = text_from_region(&region);
}

#[test]
#[ignore = "Region.text() method not yet implemented"]
fn test_region_text_method_compiles() {
    // Test that Region has text() method
    // Regionがtext()メソッドを持つことをテスト
    let _region = Region::new(100, 100, 200, 50);
    // let _result = region.text();
}

#[test]
#[ignore = "Region.text_with_config() method not yet implemented"]
fn test_region_text_with_config_compiles() {
    // Test that Region has text_with_config() method
    // Regionがtext_with_config()メソッドを持つことをテスト
    let _region = Region::new(100, 100, 200, 50);
    let _config = OcrConfig::new().with_language(OcrLanguage::Japanese);
    // let _result = region.text_with_config(&config);
}

#[test]
fn test_ocr_config_creation() {
    // Test OcrConfig creation and methods
    // OcrConfigの作成とメソッドをテスト
    let config = OcrConfig::new()
        .with_language(OcrLanguage::Japanese)
        .with_page_segmentation_mode(3);

    assert_eq!(config.get_language_code(), "jpn");
    assert_eq!(config.page_segmentation_mode, 3);
}

#[test]
fn test_multiple_languages() {
    // Test different language configurations
    // 異なる言語設定をテスト
    let langs = vec![
        (OcrLanguage::English, "eng"),
        (OcrLanguage::Japanese, "jpn"),
        (OcrLanguage::ChineseSimplified, "chi_sim"),
        (OcrLanguage::ChineseTraditional, "chi_tra"),
        (OcrLanguage::Korean, "kor"),
    ];

    for (lang, code) in langs {
        let config = OcrConfig::new().with_language(lang);
        assert_eq!(config.get_language_code(), code);
    }
}
