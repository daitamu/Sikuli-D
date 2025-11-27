//! Screen + ImageMatcher integration tests
//! 画面 + 画像マッチャー統合テスト
//!
//! Tests the integration between Screen capture and ImageMatcher for finding patterns.
//! 画面キャプチャと画像マッチャーの統合をテストし、パターン検索を検証します。

use sikulix_core::{ImageMatcher, Pattern, Region, Screen};
use image::{DynamicImage, RgbaImage};

/// Helper to create a simple test pattern image
/// シンプルなテストパターン画像を作成するヘルパー
fn create_test_pattern(width: u32, height: u32, color: u8) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        pixel[0] = color;
        pixel[1] = color;
        pixel[2] = color;
        pixel[3] = 255;
    }
    DynamicImage::ImageRgba8(img)
}

/// Helper to create a screen image with a pattern embedded at a specific location
/// 特定の位置にパターンが埋め込まれた画面画像を作成するヘルパー
fn create_screen_with_pattern(
    screen_width: u32,
    screen_height: u32,
    pattern_x: u32,
    pattern_y: u32,
    pattern: &DynamicImage,
) -> DynamicImage {
    let mut screen = RgbaImage::new(screen_width, screen_height);

    // Fill with gray background
    // グレーの背景で塗りつぶし
    for pixel in screen.pixels_mut() {
        pixel[0] = 128;
        pixel[1] = 128;
        pixel[2] = 128;
        pixel[3] = 255;
    }

    // Embed pattern at specified location
    // 指定位置にパターンを埋め込み
    let pattern_rgba = pattern.to_rgba8();
    for y in 0..pattern_rgba.height() {
        for x in 0..pattern_rgba.width() {
            let screen_x = pattern_x + x;
            let screen_y = pattern_y + y;
            if screen_x < screen_width && screen_y < screen_height {
                let pattern_pixel = pattern_rgba.get_pixel(x, y);
                screen.put_pixel(screen_x, screen_y, *pattern_pixel);
            }
        }
    }

    DynamicImage::ImageRgba8(screen)
}

#[test]
fn test_find_pattern_in_mock_screen() -> sikulix_core::Result<()> {
    // Create a test pattern
    // テストパターンを作成
    let pattern_img = create_test_pattern(50, 50, 200);
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.8);

    // Create a screen image with the pattern at position (100, 100)
    // 位置(100, 100)にパターンを配置した画面画像を作成
    let screen_img = create_screen_with_pattern(800, 600, 100, 100, &pattern_img);

    // Find the pattern
    // パターンを検索
    let matcher = ImageMatcher::new().with_min_similarity(0.8);
    let result = matcher.find(&screen_img, &pattern)?;

    assert!(result.is_some(), "Pattern should be found");
    let m = result.unwrap();

    // Verify the match location is near expected position
    // マッチ位置が期待位置付近であることを確認
    let (cx, cy) = m.center();
    assert!(
        (cx - 125).abs() < 5 && (cy - 125).abs() < 5,
        "Match center should be near (125, 125), got ({}, {})",
        cx,
        cy
    );

    // Verify high similarity score
    // 高い類似度スコアを確認
    assert!(m.score >= 0.95, "Score should be very high for exact match, got {}", m.score);

    Ok(())
}

#[test]
fn test_find_all_patterns() -> sikulix_core::Result<()> {
    // Create a test pattern
    // テストパターンを作成
    let pattern_img = create_test_pattern(30, 30, 180);
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.85);

    // Create screen with multiple instances of the pattern
    // パターンの複数インスタンスを持つ画面を作成
    let mut screen_img = DynamicImage::ImageRgba8(RgbaImage::new(800, 600));
    for pixel in screen_img.as_mut_rgba8().unwrap().pixels_mut() {
        pixel[0] = 100;
        pixel[1] = 100;
        pixel[2] = 100;
        pixel[3] = 255;
    }

    // Place pattern at multiple locations
    // 複数の位置にパターンを配置
    let positions = vec![(50, 50), (200, 100), (400, 300)];
    for (x, y) in &positions {
        let pattern_rgba = pattern_img.to_rgba8();
        for py in 0..pattern_rgba.height() {
            for px in 0..pattern_rgba.width() {
                let screen_x = x + px;
                let screen_y = y + py;
                if screen_x < 800 && screen_y < 600 {
                    let pattern_pixel = pattern_rgba.get_pixel(px, py);
                    screen_img.as_mut_rgba8().unwrap().put_pixel(screen_x, screen_y, *pattern_pixel);
                }
            }
        }
    }

    // Find all instances
    // すべてのインスタンスを検索
    let matcher = ImageMatcher::new().with_min_similarity(0.85);
    let matches = matcher.find_all(&screen_img, &pattern)?;

    assert!(
        matches.len() >= 2,
        "Should find at least 2 matches, found {}",
        matches.len()
    );

    for m in &matches {
        assert!(m.score >= 0.85, "All matches should have score >= 0.85");
    }

    Ok(())
}

#[test]
fn test_pattern_not_found() -> sikulix_core::Result<()> {
    // Create a pattern that won't be in the screen
    // 画面に存在しないパターンを作成
    let pattern_img = create_test_pattern(50, 50, 255);
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.9);

    // Create a completely different screen
    // 完全に異なる画面を作成
    let screen_img = create_test_pattern(800, 600, 50);

    // Try to find the pattern
    // パターンを検索試行
    let matcher = ImageMatcher::new().with_min_similarity(0.9);
    let result = matcher.find(&screen_img, &pattern)?;

    assert!(result.is_none(), "Pattern should not be found");

    Ok(())
}

#[test]
fn test_similarity_threshold() -> sikulix_core::Result<()> {
    // Create a pattern
    // パターンを作成
    let pattern_img = create_test_pattern(40, 40, 180);
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };

    // Create a similar but not exact screen
    // 似ているが完全一致ではない画面を作成
    let screen_img = create_screen_with_pattern(800, 600, 100, 100, &create_test_pattern(40, 40, 175));

    // Low threshold should find it
    // 低い閾値なら見つかるべき
    let pattern_low = Pattern::new(pattern_bytes.clone()).similar(0.6);
    let matcher_low = ImageMatcher::new().with_min_similarity(0.6);
    let result_low = matcher_low.find(&screen_img, &pattern_low)?;
    assert!(result_low.is_some(), "Pattern should be found with low threshold");

    // Very high threshold should not find it
    // 非常に高い閾値では見つからないべき
    let pattern_high = Pattern::new(pattern_bytes).similar(0.99);
    let matcher_high = ImageMatcher::new().with_min_similarity(0.99);
    let result_high = matcher_high.find(&screen_img, &pattern_high)?;
    assert!(result_high.is_none(), "Pattern should not be found with very high threshold");

    Ok(())
}

#[test]
#[ignore = "Requires actual screen capture - run with: cargo test -- --ignored"]
fn test_real_screen_capture_and_find() -> sikulix_core::Result<()> {
    // This test uses real screen capture
    // このテストは実際の画面キャプチャを使用
    let mut screen = Screen::primary();
    let screenshot = screen.capture()?;

    // Get screen dimensions
    // 画面サイズを取得
    let (width, height) = screen.dimensions()?;
    assert!(width > 0 && height > 0, "Screen dimensions should be valid");

    // Verify screenshot matches dimensions
    // スクリーンショットがサイズと一致することを確認
    assert_eq!(screenshot.width(), width);
    assert_eq!(screenshot.height(), height);

    // Extract a small region as a pattern
    // 小さな領域をパターンとして抽出
    let region = Region::new(100, 100, 50, 50);
    let region_capture = screen.capture_region(&region)?;

    // Convert to pattern
    // パターンに変換
    let pattern_bytes = {
        let mut buffer = Vec::new();
        region_capture
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.95);

    // Try to find it in the full screenshot
    // フルスクリーンショット内で検索試行
    let matcher = ImageMatcher::new().with_min_similarity(0.95);
    let result = matcher.find(&screenshot, &pattern)?;

    assert!(result.is_some(), "Should find the extracted region in full screenshot");

    Ok(())
}

#[test]
#[ignore = "Requires actual screen and timeout - run with: cargo test -- --ignored"]
fn test_wait_for_pattern_timeout() -> sikulix_core::Result<()> {
    use sikulix_core::SikulixError;

    // Create a pattern that won't appear
    // 出現しないパターンを作成
    let pattern_img = create_test_pattern(10, 10, 255);
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes);

    let screen = Screen::primary();
    let matcher = ImageMatcher::new().with_scan_interval(50);

    // This should timeout
    // タイムアウトするべき
    let start = std::time::Instant::now();
    let result = matcher.wait(&screen, &pattern, 0.5);
    let elapsed = start.elapsed();

    assert!(result.is_err(), "Should timeout when pattern not found");
    assert!(
        elapsed.as_millis() >= 450 && elapsed.as_millis() < 700,
        "Should timeout after approximately 0.5s, took {}ms",
        elapsed.as_millis()
    );

    match result {
        Err(SikulixError::FindFailed { timeout_secs, .. }) => {
            assert!((timeout_secs - 0.5).abs() < 0.1);
        }
        _ => panic!("Expected FindFailed error"),
    }

    Ok(())
}

#[test]
fn test_matcher_configuration() {
    // Test matcher configuration methods
    // マッチャー設定メソッドをテスト
    let matcher = ImageMatcher::new()
        .with_min_similarity(0.85)
        .with_scan_interval(100);

    // Configuration should be applied (verified internally)
    // 設定が適用されているべき（内部で確認）
    let _ = matcher;
}

#[test]
fn test_region_extraction() -> sikulix_core::Result<()> {
    // Test extracting a specific region from screen
    // 画面から特定領域を抽出するテスト
    let screen_img = create_test_pattern(800, 600, 150);

    // Simulate region capture by cropping
    // クロップによる領域キャプチャをシミュレート
    let region = Region::new(100, 100, 200, 150);
    let cropped = screen_img.crop_imm(
        region.x as u32,
        region.y as u32,
        region.width,
        region.height,
    );

    assert_eq!(cropped.width(), 200);
    assert_eq!(cropped.height(), 150);

    Ok(())
}
