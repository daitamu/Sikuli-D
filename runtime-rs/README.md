# Sikuli-D Runtime

**A standalone Python runtime for GUI automation scripts.**

**GUI自動化スクリプト用のスタンドアロンPythonランタイム。**

---

## Features / 機能

- **Python 2/3 Dual Support** - Automatic detection and conversion of legacy scripts
  - **Python 2/3 両対応** - レガシースクリプトの自動検出と変換
- **Built-in Python** - No external Python installation required
  - **Python内蔵** - 外部Pythonのインストール不要
- **Japanese Support** - Full Unicode support in logs and output
  - **日本語対応** - ログ出力で日本語を使用してもエラーになりません
- **REPL Mode** - Interactive development and testing
  - **REPLモード** - 対話的な開発とテスト
- **Headless Execution** - Run scripts without GUI
  - **ヘッドレス実行** - GUIなしでスクリプト実行

---

## Quick Start / クイックスタート

### Basic Script / 基本スクリプト

```python
from sikulid_api import *

# Click on an image
click("button.png")

# Type text (Japanese supported)
type("こんにちは世界")

# Wait for element
wait("dialog.png", 10)

# Find all matches
matches = findAll("icon.png")
for m in matches:
    print(f"Found at {m.x}, {m.y}")
```

### Observer Pattern / オブザーバーパターン

```python
from sikulid_api import Observer, Screen

screen = Screen()
region = screen.get_region()
observer = Observer(region)

def on_appear(match):
    print(f"Element appeared at {match}")

observer.onAppear("target.png", on_appear)
observer.observe(30)  # Watch for 30 seconds
```

---

## API Reference / APIリファレンス

### Screen Operations / 画面操作

| Function | Description |
|----------|-------------|
| `click(image)` | Click on image location |
| `doubleClick(image)` | Double-click on image |
| `rightClick(image)` | Right-click on image |
| `hover(image)` | Move mouse to image |
| `drag(from, to)` | Drag from one location to another |
| `dragDrop(from, to)` | Drag and drop |

### Keyboard / キーボード

| Function | Description |
|----------|-------------|
| `type(text)` | Type text (Unicode supported) |
| `paste(text)` | Paste text from clipboard |
| `hotkey(keys...)` | Press key combination |

### Wait / 待機

| Function | Description |
|----------|-------------|
| `wait(image, timeout)` | Wait for image to appear |
| `waitVanish(image, timeout)` | Wait for image to disappear |
| `exists(image, timeout)` | Check if image exists |

### Find / 検索

| Function | Description |
|----------|-------------|
| `find(image)` | Find first match |
| `findAll(image)` | Find all matches |

### Settings / 設定

| Function | Description |
|----------|-------------|
| `Settings.MinSimilarity` | Default match threshold (0.7) |
| `Settings.AutoWaitTimeout` | Default wait timeout (3.0s) |
| `Settings.ObserveScanRate` | Observer scan interval (3.0s) |

---

## Python 2/3 Compatibility / Python 2/3 互換性

The runtime automatically detects Python 2 syntax and converts it:

ランタイムはPython 2構文を自動検出して変換します：

```python
# Python 2 style (automatically converted)
print "Hello"           # → print("Hello")
raw_input("Name: ")     # → input("Name: ")
xrange(10)              # → range(10)
```

---

## Building / ビルド

```bash
cd runtime-rs

# Build Python bindings
pip install maturin
maturin build --release

# Install locally
pip install target/wheels/*.whl
```

---

## Project Structure / プロジェクト構造

```
runtime-rs/
├── src/              # Rust source
├── sikulid_api/      # Python wrapper module
│   └── __init__.py   # API exports
├── examples/         # Example scripts
├── Cargo.toml        # Rust dependencies
└── README.md         # This file
```

---

## License / ライセンス

MIT License - see [LICENSE](../LICENSE) for details.

MITライセンス - 詳細は [LICENSE](../LICENSE) を参照。
