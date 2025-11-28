//! Observer integration tests
//! オブザーバー統合テスト
//!
//! These tests verify the Observer functionality with real screen captures.
//! これらのテストは実際の画面キャプチャでObserver機能を検証します。

use sikulid::{Observer, Pattern, Region};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
#[ignore = "Requires screen capture - run with: cargo test -- --ignored"]
fn test_observer_background_execution() {
    // Test that observer can run in background
    // オブザーバーがバックグラウンドで実行できることをテスト
    let region = Region::new(0, 0, 200, 200);
    let observer = Observer::new(region);

    let handle = observer.observe_in_background();

    // Let it run for a bit
    // しばらく実行させる
    thread::sleep(Duration::from_millis(500));

    assert!(observer.is_running());

    // Stop and verify cleanup
    // 停止してクリーンアップを確認
    observer.stop();
    handle.join().unwrap().unwrap();

    assert!(!observer.is_running());
}

#[test]
#[ignore = "Requires screen capture - run with: cargo test -- --ignored"]
fn test_observer_on_change_detection() {
    // Test change detection callback
    // 変化検出コールバックをテスト
    let region = Region::new(0, 0, 100, 100);
    let mut observer = Observer::new(region);
    observer.set_interval(100); // Check every 100ms

    let change_count = Arc::new(AtomicUsize::new(0));
    let change_count_clone = Arc::clone(&change_count);

    observer.on_change(0.05, move |change_amount| {
        println!("Change detected: {:.2}%", change_amount * 100.0);
        change_count_clone.fetch_add(1, Ordering::SeqCst);
    });

    let handle = observer.observe_in_background();

    // Run for a short time
    // 短時間実行
    thread::sleep(Duration::from_millis(1000));

    observer.stop();
    handle.join().unwrap().unwrap();

    // Change count depends on screen activity
    // 変化カウントは画面アクティビティに依存
    println!(
        "Total changes detected: {}",
        change_count.load(Ordering::SeqCst)
    );
}

#[test]
#[ignore = "Requires test pattern image"]
fn test_observer_pattern_appear() {
    // Test pattern appearance detection
    // パターン出現検出をテスト
    let region = Region::new(0, 0, 800, 600);
    let mut observer = Observer::new(region);

    // This test requires a pattern image file
    // このテストにはパターン画像ファイルが必要
    let pattern =
        Pattern::from_file("tests/fixtures/test_button.png").expect("Test pattern not found");

    let found = Arc::new(Mutex::new(false));
    let found_clone = Arc::clone(&found);

    observer.on_appear(pattern, move |m| {
        println!("Pattern found at ({}, {})", m.region.x, m.region.y);
        *found_clone.lock().unwrap() = true;
    });

    let handle = observer.observe_in_background();

    // Wait for potential detection
    // 検出の可能性を待つ
    thread::sleep(Duration::from_secs(2));

    observer.stop();
    handle.join().unwrap().unwrap();

    // Result depends on whether pattern is visible on screen
    // 結果はパターンが画面に表示されているかに依存
    println!("Pattern was found: {}", *found.lock().unwrap());
}

#[test]
fn test_observer_multiple_handlers_registration() {
    // Test registering multiple handlers
    // 複数のハンドラー登録をテスト
    let region = Region::new(0, 0, 100, 100);
    let mut observer = Observer::new(region);

    let counter1 = Arc::new(AtomicUsize::new(0));
    let counter2 = Arc::new(AtomicUsize::new(0));
    let counter3 = Arc::new(AtomicUsize::new(0));

    let c1 = Arc::clone(&counter1);
    let c2 = Arc::clone(&counter2);
    let c3 = Arc::clone(&counter3);

    // Register multiple handlers
    // 複数のハンドラーを登録
    let pattern1 = Pattern::new(vec![1, 2, 3]);
    let pattern2 = Pattern::new(vec![4, 5, 6]);

    observer.on_appear(pattern1, move |_| {
        c1.fetch_add(1, Ordering::SeqCst);
    });

    observer.on_vanish(pattern2, move || {
        c2.fetch_add(1, Ordering::SeqCst);
    });

    observer.on_change(0.1, move |_| {
        c3.fetch_add(1, Ordering::SeqCst);
    });

    // Handlers should be registered (verified internally)
    // ハンドラーは登録されているべき（内部で確認）
    assert!(!observer.is_running());
}

#[test]
fn test_observer_configuration() {
    // Test observer configuration methods
    // オブザーバー設定メソッドをテスト
    let region = Region::new(100, 100, 200, 200);
    let mut observer = Observer::new(region);

    // Test interval setting
    // 間隔設定をテスト
    observer.set_interval(250);

    // Test similarity setting
    // 類似度設定をテスト
    observer.set_min_similarity(0.85);

    // These should not panic
    // パニックしないべき
    assert!(!observer.is_running());
}

#[test]
fn test_observer_timeout_behavior() {
    // Test observe() with timeout
    // タイムアウト付きobserve()をテスト
    let region = Region::new(0, 0, 50, 50);
    let observer = Observer::new(region);

    let start = std::time::Instant::now();
    observer.observe(0.2).unwrap(); // 200ms timeout
    let elapsed = start.elapsed();

    // Should complete after approximately 200ms
    // 約200ms後に完了するべき
    // Allow more slack for CI/CD environments under load
    // CI/CD環境での負荷を考慮して余裕を持たせる
    assert!(elapsed.as_millis() >= 180);
    assert!(elapsed.as_millis() < 1000);
}

#[test]
fn test_observer_immediate_stop() {
    // Test stopping observer immediately after start
    // 開始直後にオブザーバーを停止するテスト
    let region = Region::new(0, 0, 100, 100);
    let observer = Observer::new(region);

    let handle = observer.observe_in_background();

    // Stop immediately
    // 即座に停止
    observer.stop();

    let result = handle.join().unwrap();
    assert!(result.is_ok());
    assert!(!observer.is_running());
}
