//! Observer demonstration
//! オブザーバーのデモンストレーション
//!
//! This example demonstrates the Observer API for background monitoring.
//! この例はバックグラウンド監視のためのObserver APIを実証します。
//!
//! # Usage / 使用方法
//!
//! ```bash
//! cargo run --example observer_demo
//! ```

use sikulix_core::{Observer, Pattern, Region};
use std::cell::Cell;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();

    println!("=== SikuliX Observer Demo ===\n");

    // Example 1: Basic observation with timeout
    // 例1: タイムアウト付き基本監視
    println!("Example 1: Basic observation with timeout");
    basic_observation_example();

    // Example 2: Background observation with callbacks
    // 例2: コールバック付きバックグラウンド監視
    println!("\nExample 2: Background observation with callbacks");
    background_observation_example();

    // Example 3: Change detection
    // 例3: 変化検出
    println!("\nExample 3: Change detection");
    change_detection_example();

    println!("\n=== Demo completed ===");
}

/// Basic observation with timeout
/// タイムアウト付き基本監視
fn basic_observation_example() {
    let region = Region::new(0, 0, 200, 200);
    let observer = Observer::new(region);

    println!("Observing region (0, 0, 200, 200) for 2 seconds...");

    // Observe for 2 seconds (blocking)
    // 2秒間監視（ブロッキング）
    match observer.observe(2.0) {
        Ok(_) => println!("Observation completed successfully"),
        Err(e) => eprintln!("Observation error: {}", e),
    }
}

/// Background observation with callbacks
/// コールバック付きバックグラウンド監視
fn background_observation_example() {
    let region = Region::new(100, 100, 400, 300);
    let mut observer = Observer::new(region);

    // Set faster interval for demo
    // デモ用に高速間隔を設定
    observer.set_interval(200);

    // Register appearance callback
    // 出現コールバックを登録
    // Note: This requires a valid pattern image
    // 注意: これには有効なパターン画像が必要
    let pattern = Pattern::new(vec![
        137, 80, 78, 71, 13, 10, 26, 10, // Minimal PNG header
        // In real usage, load from file: Pattern::from_file("button.png").unwrap()
    ]);

    observer.on_appear(pattern, |m| {
        println!("  ✓ Pattern appeared at ({}, {})", m.region.x, m.region.y);
    });

    // Start background observation
    // バックグラウンド監視を開始
    println!("Starting background observation for 3 seconds...");
    let handle = observer.observe_in_background();

    // Simulate doing other work
    // 他の作業をシミュレート
    for i in 1..=3 {
        println!("  Main thread working... ({}s)", i);
        thread::sleep(Duration::from_secs(1));
    }

    // Stop observation
    // 監視を停止
    println!("Stopping observer...");
    observer.stop();

    match handle.join() {
        Ok(Ok(_)) => println!("Background observation completed successfully"),
        Ok(Err(e)) => eprintln!("Observation error: {}", e),
        Err(_) => eprintln!("Thread join error"),
    }
}

/// Change detection example
/// 変化検出の例
fn change_detection_example() {
    let region = Region::new(0, 0, 300, 300);
    let mut observer = Observer::new(region);

    observer.set_interval(300);

    let change_count = Cell::new(0);

    // Register change callback
    // 変化コールバックを登録
    observer.on_change(0.05, move |change_amount| {
        let count = change_count.get() + 1;
        change_count.set(count);
        println!(
            "  ⚠ Screen changed by {:.1}% (change #{})",
            change_amount * 100.0,
            count
        );
    });

    println!("Monitoring for visual changes for 3 seconds...");
    println!("(Move your mouse or windows to trigger changes)");

    let handle = observer.observe_in_background();

    thread::sleep(Duration::from_secs(3));

    observer.stop();
    handle.join().unwrap().unwrap();

    println!("Change detection completed");
}

/// Pattern vanish detection example (commented out as it needs setup)
/// パターン消失検出の例（セットアップが必要なのでコメントアウト）
#[allow(dead_code)]
fn vanish_detection_example() {
    let region = Region::new(0, 0, 800, 600);
    let mut observer = Observer::new(region);

    // Load pattern that might disappear
    // 消えるかもしれないパターンを読み込み
    let pattern = Pattern::from_file("popup.png").unwrap();

    observer.on_vanish(pattern, || {
        println!("  ✓ Popup disappeared!");
    });

    println!("Monitoring for popup to vanish...");
    let handle = observer.observe_in_background();

    thread::sleep(Duration::from_secs(10));

    observer.stop();
    handle.join().unwrap().unwrap();
}

/// Advanced example with multiple handlers
/// 複数のハンドラーを持つ高度な例
#[allow(dead_code)]
fn advanced_multi_handler_example() {
    let region = Region::new(0, 0, 1024, 768);
    let mut observer = Observer::new(region);

    // Set custom similarity threshold
    // カスタム類似度閾値を設定
    observer.set_min_similarity(0.85);
    observer.set_interval(100);

    // Monitor multiple patterns
    // 複数のパターンを監視
    let button1 = Pattern::from_file("button1.png").unwrap();
    let button2 = Pattern::from_file("button2.png").unwrap();

    observer.on_appear(button1, |m| {
        println!("Button 1 appeared at {:?}", m.center());
    });

    observer.on_appear(button2, |m| {
        println!("Button 2 appeared at {:?}", m.center());
    });

    observer.on_change(0.1, |change| {
        println!("Significant change: {:.1}%", change * 100.0);
    });

    println!("Observing with multiple handlers...");
    let handle = observer.observe_in_background();

    // Run for 1 minute
    // 1分間実行
    thread::sleep(Duration::from_secs(60));

    observer.stop();
    handle.join().unwrap().unwrap();
}
