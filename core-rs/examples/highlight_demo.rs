//! Highlight Overlay Demo
//! ハイライトオーバーレイのデモ
//!
//! This example demonstrates the visual highlight overlay functionality.
//! この例はビジュアルハイライトオーバーレイ機能を示します。
//!
//! Run with: cargo run --example highlight_demo
//! 実行方法: cargo run --example highlight_demo

use sikulix_core::{Color, Region};

fn main() {
    // Initialize logging
    // ロギングを初期化
    env_logger::init();

    println!("Highlight Overlay Demo");
    println!("ハイライトオーバーレイデモ");
    println!();

    // Example 1: Simple highlight with default settings
    // 例1: デフォルト設定でのシンプルなハイライト
    println!("Example 1: Basic highlight (red, 2 seconds)");
    println!("例1: 基本的なハイライト（赤、2秒）");
    let region1 = Region::new(100, 100, 300, 200);
    let color = Color::rgb(255, 0, 0); // Red

    match sikulix_core::debug::highlight(&region1, 2000, color) {
        Ok(()) => println!("✓ Highlight shown successfully"),
        Err(e) => println!("✗ Failed to show highlight: {}", e),
    }

    std::thread::sleep(std::time::Duration::from_millis(2500));

    // Example 2: Custom color and duration
    // 例2: カスタム色と時間
    println!("\nExample 2: Green highlight for 1.5 seconds");
    println!("例2: 緑色のハイライトを1.5秒");
    let region2 = Region::new(500, 300, 250, 150);
    let green = Color::rgb(0, 255, 0);

    match sikulix_core::debug::highlight(&region2, 1500, green) {
        Ok(()) => println!("✓ Green highlight shown"),
        Err(e) => println!("✗ Failed: {}", e),
    }

    std::thread::sleep(std::time::Duration::from_millis(2000));

    // Example 3: Using HighlightConfig for advanced options
    // 例3: 高度なオプションにHighlightConfigを使用
    println!("\nExample 3: Custom configuration (blue, thick border)");
    println!("例3: カスタム設定（青、太い境界線）");
    let region3 = Region::new(200, 400, 400, 100);
    let config = sikulix_core::debug::HighlightConfig::new()
        .with_color(0, 0, 255) // Blue
        .with_border_width(5)
        .with_duration_ms(3000);

    match sikulix_core::debug::show_highlight_with_config(&region3, &config) {
        Ok(()) => println!("✓ Custom highlight shown"),
        Err(e) => println!("✗ Failed: {}", e),
    }

    std::thread::sleep(std::time::Duration::from_millis(3500));

    // Example 4: Multiple overlapping highlights
    // 例4: 複数の重なり合うハイライト
    println!("\nExample 4: Multiple simultaneous highlights");
    println!("例4: 複数の同時ハイライト");

    let regions = vec![
        (Region::new(150, 150, 150, 100), Color::rgb(255, 0, 0)),   // Red
        (Region::new(250, 200, 150, 100), Color::rgb(0, 255, 0)),   // Green
        (Region::new(350, 250, 150, 100), Color::rgb(0, 0, 255)),   // Blue
    ];

    for (region, color) in regions {
        let _ = sikulix_core::debug::highlight(&region, 2000, color);
        std::thread::sleep(std::time::Duration::from_millis(200)); // Small delay
    }

    println!("✓ Multiple highlights shown");
    std::thread::sleep(std::time::Duration::from_millis(2500));

    // Example 5: Using with Match result (simulated)
    // 例5: マッチ結果と共に使用（シミュレート）
    println!("\nExample 5: Highlighting a match result");
    println!("例5: マッチ結果のハイライト");
    let match_region = Region::new(300, 300, 200, 150);
    let match_result = sikulix_core::Match::new(match_region, 0.95);

    match sikulix_core::debug::highlight_match(&match_result, 2000) {
        Ok(()) => println!("✓ Match highlight shown (score: {})", match_result.score_percent()),
        Err(e) => println!("✗ Failed: {}", e),
    }

    std::thread::sleep(std::time::Duration::from_millis(2500));

    println!("\nDemo complete!");
    println!("デモ完了！");
}
