//! Comprehensive workflow integration tests
//! 包括的なワークフロー統合テスト
//!
//! Tests complete workflows that combine multiple components.
//! 複数のコンポーネントを組み合わせた完全なワークフローをテストします。

use sikulix_core::{ImageMatcher, Pattern, Region, Screen, Observer};
use image::{DynamicImage, RgbaImage};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Helper to create a test image
/// テスト画像を作成するヘルパー
fn create_test_image(width: u32, height: u32, color: (u8, u8, u8)) -> DynamicImage {
    let mut img = RgbaImage::new(width, height);
    for pixel in img.pixels_mut() {
        pixel[0] = color.0;
        pixel[1] = color.1;
        pixel[2] = color.2;
        pixel[3] = 255;
    }
    DynamicImage::ImageRgba8(img)
}

#[test]
fn test_find_and_inspect_workflow() -> sikulix_core::Result<()> {
    // Test workflow: create pattern -> find in screen -> inspect match
    // ワークフロー: パターン作成 -> 画面検索 -> マッチ検査

    // 1. Create a pattern
    // 1. パターンを作成
    let pattern_img = create_test_image(50, 50, (200, 150, 100));
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.85);

    // 2. Create a screen with the pattern
    // 2. パターンを含む画面を作成
    let mut screen_img = create_test_image(800, 600, (128, 128, 128));
    let pattern_rgba = pattern_img.to_rgba8();
    for y in 0..pattern_rgba.height() {
        for x in 0..pattern_rgba.width() {
            let screen_x = 100 + x;
            let screen_y = 100 + y;
            if screen_x < 800 && screen_y < 600 {
                let pixel = pattern_rgba.get_pixel(x, y);
                screen_img.as_mut_rgba8().unwrap().put_pixel(screen_x, screen_y, *pixel);
            }
        }
    }

    // 3. Find the pattern
    // 3. パターンを検索
    let matcher = ImageMatcher::new().with_min_similarity(0.85);
    let result = matcher.find(&screen_img, &pattern)?;

    assert!(result.is_some(), "Pattern should be found");

    // 4. Inspect the match
    // 4. マッチを検査
    let m = result.unwrap();
    let (cx, cy) = m.center();

    assert!(m.score >= 0.85, "Match score should be high");
    assert!((cx - 125).abs() < 10, "Match should be near expected location");
    assert!((cy - 125).abs() < 10, "Match should be near expected location");

    // 5. Extract region
    // 5. 領域を抽出
    let match_region = m.region;
    assert!(match_region.width >= 40 && match_region.width <= 60);
    assert!(match_region.height >= 40 && match_region.height <= 60);

    Ok(())
}

#[test]
fn test_multi_pattern_workflow() -> sikulix_core::Result<()> {
    // Test workflow: find multiple patterns in sequence
    // ワークフロー: 複数パターンを順番に検索

    // Create different patterns
    // 異なるパターンを作成
    let pattern1_img = create_test_image(30, 30, (255, 0, 0));
    let pattern2_img = create_test_image(30, 30, (0, 255, 0));
    let pattern3_img = create_test_image(30, 30, (0, 0, 255));

    let patterns = vec![pattern1_img, pattern2_img, pattern3_img];
    let positions = vec![(50, 50), (200, 100), (400, 300)];

    // Create screen with all patterns
    // すべてのパターンを含む画面を作成
    let mut screen_img = create_test_image(800, 600, (128, 128, 128));
    for (pattern_img, (px, py)) in patterns.iter().zip(positions.iter()) {
        let pattern_rgba = pattern_img.to_rgba8();
        for y in 0..pattern_rgba.height() {
            for x in 0..pattern_rgba.width() {
                let screen_x = px + x;
                let screen_y = py + y;
                if screen_x < 800 && screen_y < 600 {
                    let pixel = pattern_rgba.get_pixel(x, y);
                    screen_img.as_mut_rgba8().unwrap().put_pixel(screen_x, screen_y, *pixel);
                }
            }
        }
    }

    // Find each pattern
    // 各パターンを検索
    let matcher = ImageMatcher::new().with_min_similarity(0.9);

    for (i, pattern_img) in patterns.iter().enumerate() {
        let pattern_bytes = {
            let mut buffer = Vec::new();
            pattern_img
                .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
                .unwrap();
            buffer
        };
        let pattern = Pattern::new(pattern_bytes);

        let result = matcher.find(&screen_img, &pattern)?;
        assert!(result.is_some(), "Pattern {} should be found", i + 1);
    }

    Ok(())
}

#[test]
fn test_observer_pattern_appear_workflow() {
    // Test workflow: set up observer -> wait for pattern
    // ワークフロー: オブザーバーセットアップ -> パターン待機

    let region = Region::new(0, 0, 200, 200);
    let mut observer = Observer::new(region);

    let found = Arc::new(Mutex::new(false));
    let found_clone = found.clone();

    // Create a test pattern
    // テストパターンを作成
    let pattern_img = create_test_image(30, 30, (200, 200, 200));
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes);

    // Set up callback
    // コールバックをセットアップ
    observer.on_appear(pattern, move |m| {
        println!("Pattern appeared at {:?}", m.center());
        *found_clone.lock().unwrap() = true;
    });

    // In a real scenario, the observer would run in background
    // 実際のシナリオでは、オブザーバーはバックグラウンドで実行される
    // For this test, we just verify the setup worked
    // このテストでは、セットアップが機能したことを確認するだけ
    assert!(!observer.is_running());
}

#[test]
fn test_change_detection_workflow() {
    // Test workflow: monitor region for changes
    // ワークフロー: 領域の変化を監視

    let region = Region::new(100, 100, 300, 200);
    let mut observer = Observer::new(region);

    observer.set_interval(100);
    observer.set_min_similarity(0.95);

    let change_count = Arc::new(Mutex::new(0));
    let change_count_clone = change_count.clone();

    observer.on_change(0.1, move |amount| {
        println!("Change detected: {:.2}%", amount * 100.0);
        *change_count_clone.lock().unwrap() += 1;
    });

    // Verify setup
    // セットアップを確認
    assert!(!observer.is_running());
    assert_eq!(*change_count.lock().unwrap(), 0);
}

#[test]
fn test_region_operations_workflow() {
    // Test workflow: region calculations and manipulations
    // ワークフロー: 領域計算と操作

    let r1 = Region::new(100, 100, 200, 150);
    let r2 = Region::new(150, 125, 200, 150);

    // Test intersection
    // 交差をテスト
    assert!(r1.intersects(&r2));

    let intersection = r1.intersection(&r2);
    assert!(intersection.is_some());

    if let Some(inter) = intersection {
        assert!(inter.area() > 0);
        assert!(inter.width <= r1.width.min(r2.width));
        assert!(inter.height <= r1.height.min(r2.height));
    }

    // Test region expansion
    // 領域拡張をテスト
    let expanded = r1.expand(10);
    assert_eq!(expanded.width, r1.width + 20);
    assert_eq!(expanded.height, r1.height + 20);

    // Test region offset
    // 領域オフセットをテスト
    let offset = r1.offset(50, 50);
    assert_eq!(offset.x, r1.x + 50);
    assert_eq!(offset.y, r1.y + 50);

    // Test contains
    // 内包をテスト
    let (cx, cy) = r1.center();
    assert!(r1.contains(cx, cy));
}

#[test]
fn test_pattern_configuration_workflow() {
    // Test workflow: configure pattern with various settings
    // ワークフロー: 様々な設定でパターンを構成

    let pattern_img = create_test_image(40, 40, (180, 180, 180));
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };

    // Configure pattern
    // パターンを構成
    let pattern = Pattern::new(pattern_bytes)
        .similar(0.9)
        .target_offset(10, 10);

    // Verify configuration
    // 構成を確認
    assert_eq!(pattern.similarity, 0.9);
    assert_eq!(pattern.target_offset, (10, 10));
    assert!(pattern.is_valid());
}

#[test]
#[ignore = "Requires actual screen - run with: cargo test -- --ignored"]
fn test_complete_automation_workflow() -> sikulix_core::Result<()> {
    // Test complete automation workflow
    // 完全な自動化ワークフローをテスト

    // 1. Capture screen
    // 1. 画面をキャプチャ
    let mut screen = Screen::primary();
    let screenshot = screen.capture()?;

    // 2. Extract a region as pattern
    // 2. 領域をパターンとして抽出
    let region = Region::new(100, 100, 100, 50);
    let pattern_img = screen.capture_region(&region)?;

    // 3. Find the pattern
    // 3. パターンを検索
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes).similar(0.95);

    let matcher = ImageMatcher::new().with_min_similarity(0.95);
    let result = matcher.find(&screenshot, &pattern)?;

    assert!(result.is_some(), "Should find extracted region");

    // 4. Verify match location
    // 4. マッチ位置を確認
    if let Some(m) = result {
        let (cx, cy) = m.center();
        let expected_center = region.center();

        // Should be close to original location
        // 元の位置に近いべき
        assert!(
            (cx - expected_center.0).abs() < 20,
            "Center X should be close to original"
        );
        assert!(
            (cy - expected_center.1).abs() < 20,
            "Center Y should be close to original"
        );
    }

    Ok(())
}

#[test]
fn test_error_handling_workflow() {
    // Test workflow with various error conditions
    // 様々なエラー条件でのワークフローをテスト

    // Invalid pattern file
    // 無効なパターンファイル
    let result = Pattern::from_file("nonexistent.png");
    assert!(result.is_err(), "Should fail for nonexistent file");

    // Empty pattern
    // 空のパターン
    let empty_pattern = Pattern::new(vec![]);
    assert!(!empty_pattern.is_valid(), "Empty pattern should be invalid");

    // Region with zero dimensions
    // ゼロ寸法の領域
    let zero_region = Region::new(0, 0, 0, 0);
    assert_eq!(zero_region.area(), 0);
}

#[test]
fn test_concurrent_operations_workflow() {
    // Test concurrent pattern matching operations
    // 並行パターンマッチング操作をテスト

    use std::sync::Arc;
    use std::thread;

    let screen_img = Arc::new(create_test_image(800, 600, (128, 128, 128)));
    let pattern_img = create_test_image(50, 50, (200, 200, 200));
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Arc::new(Pattern::new(pattern_bytes));

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let screen_clone = screen_img.clone();
            let pattern_clone = pattern.clone();

            thread::spawn(move || {
                let matcher = ImageMatcher::new();
                let _result = matcher.find(&*screen_clone, &*pattern_clone);
                println!("Thread {} completed search", i);
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_performance_considerations() {
    // Test that operations complete in reasonable time
    // 操作が妥当な時間で完了することをテスト

    let screen_img = create_test_image(1920, 1080, (128, 128, 128));
    let pattern_img = create_test_image(50, 50, (200, 200, 200));
    let pattern_bytes = {
        let mut buffer = Vec::new();
        pattern_img
            .write_to(&mut std::io::Cursor::new(&mut buffer), image::ImageFormat::Png)
            .unwrap();
        buffer
    };
    let pattern = Pattern::new(pattern_bytes);

    let matcher = ImageMatcher::new();

    let start = std::time::Instant::now();
    let _result = matcher.find(&screen_img, &pattern);
    let elapsed = start.elapsed();

    // Should complete reasonably quickly (adjust threshold as needed)
    // 妥当な速さで完了するべき（必要に応じて閾値を調整）
    assert!(
        elapsed.as_secs() < 5,
        "Search should complete in less than 5 seconds, took {:?}",
        elapsed
    );
}
