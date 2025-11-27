# sikulix-core

**SikuliX Core Library - Next Generation**

**SikuliX コアライブラリ - 次世代版**

---

A high-performance, low-memory GUI automation library written in Rust.

Rustで書かれた高性能・省メモリのGUI自動化ライブラリ。

## Features / 機能

- **Image Matching** - Template matching algorithm for finding images on screen
  - **画像マッチング** - テンプレートマッチングによる画面上の画像検索
- **OCR** - Optical Character Recognition via Tesseract
  - **OCR** - Tesseractによる光学文字認識
- **Screen Capture** - Cross-platform screen capture (Windows, macOS, Linux)
  - **スクリーンキャプチャ** - クロスプラットフォーム対応（Windows, macOS, Linux）
- **Input Control** - Mouse and keyboard simulation
  - **入力制御** - マウス・キーボードシミュレーション
- **Python Integration** - Python 3 native bindings via PyO3
  - **Python統合** - PyO3によるPython 3ネイティブバインディング
- **Observer API** - Monitor screen regions for appearance, vanishing, and changes
  - **Observer API** - 画面領域の出現・消失・変化を監視
- **Scroll/Drag Support** - Mouse scroll and drag operations
  - **スクロール/ドラッグ対応** - マウススクロールとドラッグ操作
- **Debug Support** - Breakpoints, step execution, variable inspection
  - **デバッグ対応** - ブレークポイント、ステップ実行、変数表示
- **Plugin System** - Extensible plugin architecture with permission model
  - **プラグインシステム** - パーミッションモデル付き拡張可能アーキテクチャ

## Requirements / 動作要件

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Tesseract | 5.x (for OCR features) |

### Platform-specific / プラットフォーム固有

**Windows:**
- Windows 10/11
- Visual Studio Build Tools (for compilation)

**macOS:**
- macOS 10.15+
- Screen Recording permission required

**Linux:**
- X11 (Wayland support planned)
- libxdo-dev, libx11-dev

## Installation / インストール

Add to your `Cargo.toml`:

```toml
[dependencies]
sikulix-core = { path = "../core-rs" }
```

## Quick Start / クイックスタート

### Screen Capture / スクリーンキャプチャ

```rust
use sikulix_core::Screen;

let screen = Screen::default();
let screenshot = screen.capture().expect("Failed to capture screen");
println!("Captured: {}x{}", screenshot.width(), screenshot.height());
```

### Mouse Control / マウス制御

```rust
use sikulix_core::Mouse;

// Move mouse to position
Mouse::move_to(100, 200).expect("Failed to move mouse");

// Click at current position
Mouse::click().expect("Failed to click");

// Drag from one point to another
Mouse::drag(100, 100, 300, 300).expect("Failed to drag");

// Scroll operations
Mouse::scroll(3).expect("Failed to scroll up");
Mouse::scroll(-3).expect("Failed to scroll down");
Mouse::scroll_horizontal(3).expect("Failed to scroll right");

// Mouse button control
Mouse::mouse_down().expect("Failed to press mouse");
Mouse::mouse_up().expect("Failed to release mouse");
```

### Keyboard Control / キーボード制御

```rust
use sikulix_core::{Keyboard, Key};

// Type text
Keyboard::type_text("Hello, World!").expect("Failed to type");

// Press hotkey combination
Keyboard::hotkey(&[Key::Ctrl, Key::S]).expect("Failed to press hotkey");

// Paste text (supports Japanese/Unicode)
Keyboard::paste_text("日本語テキスト").expect("Failed to paste");
```

### Image Matching / 画像マッチング

```rust
use sikulix_core::{Screen, ImageMatcher, Pattern};

let screen = Screen::default();
let screenshot = screen.capture().expect("Failed to capture");

let pattern = Pattern::new("button.png", 0.8);
let matcher = ImageMatcher::new();

if let Ok(Some(match_result)) = matcher.find(&screenshot, &pattern) {
    println!("Found at ({}, {})", match_result.x, match_result.y);
}
```

### Observer (Python) / オブザーバー (Python)

```python
from sikulid_api import Observer, Screen

# Create observer for a region
screen = Screen()
region = screen.get_region()
observer = Observer(region)

# Set observation interval (milliseconds)
observer.setInterval(200)

# Register callbacks
def on_appear(match):
    print(f"Pattern appeared at {match}")

def on_vanish():
    print("Pattern vanished!")

def on_change(amount):
    print(f"Screen changed by {amount}")

observer.onAppear("button.png", on_appear, 0.8)
observer.onVanish("dialog.png", on_vanish)
observer.onChange(0.1, on_change)

# Start observing (blocks for specified seconds)
observer.observe(10.0)
```

## Module Overview / モジュール概要

| Module | Description |
|--------|-------------|
| `screen` | Screen capture, Mouse, Keyboard control |
| `image` | Image matching, OCR |
| `python` | Python script execution, PyO3 bindings |
| `observer` | Screen region monitoring (appear, vanish, change) |
| `debug` | Debugging support |
| `settings` | Configuration management |
| `plugin` | Plugin system |
| `project` | Project file handling |

## Testing / テスト

```bash
# Run all unit tests
cargo test

# Run integration tests (requires actual system interaction)
cargo test -- --ignored

# Run with verbose output
cargo test -- --nocapture
```

## Performance / パフォーマンス

### Benchmarks / ベンチマーク

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench matching
cargo bench --bench screen_capture
cargo bench --bench ncc_calculation

# Save benchmark results
cargo bench > benchmark_results.txt
```

### Performance Targets / パフォーマンス目標

| Operation | Target | Optimized |
|-----------|--------|-----------|
| Screen capture (1920×1080) | < 50ms | < 30ms |
| Image matching (50×50 template) | < 100ms | < 50ms |
| Non-maximum suppression (100 matches) | < 10ms | < 5ms |

See [PERFORMANCE.md](./PERFORMANCE.md) for detailed optimization information.

詳細な最適化情報は [PERFORMANCE.md](./PERFORMANCE.md) を参照してください。

### Optimizations Implemented / 実装された最適化

- **Parallel Processing** - Row-level parallelization using Rayon
  - **並列処理** - Rayon を使用した行レベルの並列化
- **Memory Access Optimization** - Improved cache locality in NCC calculation
  - **メモリアクセス最適化** - NCC計算のキャッシュ局所性改善
- **Pre-computed Statistics** - Template statistics cached for reuse
  - **事前計算統計** - テンプレート統計の再利用キャッシング
- **Efficient NMS** - Optimized non-maximum suppression with early exits
  - **効率的なNMS** - 早期終了を伴う最適化された非最大値抑制

## License / ライセンス

MIT License - see [LICENSE](../LICENSE) for details.

MITライセンス - 詳細は [LICENSE](../LICENSE) を参照してください。
