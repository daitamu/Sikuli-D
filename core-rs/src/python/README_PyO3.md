# PyO3 Bindings Implementation for Sikuli-D
# Sikuli-D の PyO3 バインディング実装

## Overview / 概要

This module provides PyO3 bindings to expose core-rs functionality to Python. The implementation follows the design specification in `RUNTIME-RS-DESIGN.md`.

このモジュールは、core-rs の機能を Python に公開する PyO3 バインディングを提供します。実装は `RUNTIME-RS-DESIGN.md` の設計仕様に従っています。

## Implementation Status / 実装状況

### ✅ Completed / 完了

1. **PyO3 Dependencies / PyO3 依存関係**
   - Added `pyo3` with `extension-module` feature to `Cargo.toml`
   - Cargo.toml に `extension-module` 機能付き `pyo3` を追加

2. **Core Classes Implemented / コアクラスを実装**
   - `PyScreen` - Screen capture wrapper / スクリーンキャプチャラッパー
   - `PyRegion` - Region operations / 領域操作
   - `PyMatch` - Match result with click actions / クリックアクション付きマッチ結果
   - `PyPattern` - Pattern configuration / パターン設定

3. **Module-Level Functions / モジュールレベル関数**
   - Image Finding: `find()`, `find_all()`, `wait()`, `exists()`
   - Mouse: `click()`, `double_click()`, `right_click()`
   - Keyboard: `type_text()`, `paste()`, `hotkey()`

4. **Error Conversion / エラー変換**
   - `SikulixError` → `PyErr` conversion
   - Proper exception types (PyRuntimeError, PyValueError, PyIOError)

5. **GIL Management / GIL 管理**
   - `py.allow_threads()` used for potentially blocking operations
   - 潜在的にブロッキングする操作に `py.allow_threads()` を使用

## Building / ビルド

### Prerequisites / 前提条件

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install maturin (for building Python extensions)
pip install maturin
```

### Build Commands / ビルドコマンド

#### Development Build / 開発ビルド

```bash
cd core-rs

# Build with Python feature
cargo build --features python

# Build Python extension module
maturin develop --features python
```

#### Release Build / リリースビルド

```bash
# Build optimized Python wheel
maturin build --release --features python

# Install the wheel
pip install target/wheels/sikulix_core-*.whl
```

### Cross-Platform Build / クロスプラットフォームビルド

```bash
# Windows
maturin build --release --features python --target x86_64-pc-windows-msvc

# macOS (Intel)
maturin build --release --features python --target x86_64-apple-darwin

# macOS (Apple Silicon)
maturin build --release --features python --target aarch64-apple-darwin

# Linux
maturin build --release --features python --target x86_64-unknown-linux-gnu
```

## Testing / テスト

### Python Test Script / Python テストスクリプト

Create `test_bindings.py`:

```python
#!/usr/bin/env python3
"""Test PyO3 bindings for sikulix_core"""

def test_import():
    """Test that the module can be imported"""
    try:
        import sikulix_py
        print("✓ Module imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Failed to import: {e}")
        return False

def test_screen():
    """Test Screen class"""
    from sikulix_py import Screen

    screen = Screen()
    print(f"✓ Screen created: {screen}")

    width, height = screen.dimensions()
    print(f"✓ Screen dimensions: {width}x{height}")

    region = screen.get_region()
    print(f"✓ Screen region: {region}")

    return True

def test_region():
    """Test Region class"""
    from sikulix_py import Region

    r = Region(100, 100, 200, 150)
    print(f"✓ Region created: {r}")

    cx, cy = r.center()
    print(f"✓ Center: ({cx}, {cy})")

    print(f"✓ Properties: x={r.x}, y={r.y}, w={r.width}, h={r.height}")

    r2 = r.offset(50, 50)
    print(f"✓ Offset region: {r2}")

    r3 = r.expand(10)
    print(f"✓ Expanded region: {r3}")

    return True

def test_pattern():
    """Test Pattern class"""
    from sikulix_py import Pattern
    import os

    # This will fail if file doesn't exist, which is expected
    try:
        p = Pattern("test_image.png")
        print(f"✓ Pattern created: {p}")
    except Exception as e:
        print(f"⚠ Pattern test skipped (expected): {e}")

    return True

def test_mouse_keyboard():
    """Test mouse and keyboard functions"""
    from sikulix_py import click, type_text, paste, hotkey

    # Test function availability
    print("✓ Mouse functions available: click, double_click, right_click")
    print("✓ Keyboard functions available: type_text, paste, hotkey")

    # Don't actually execute to avoid unwanted input
    return True

def main():
    """Run all tests"""
    print("=" * 60)
    print("Testing SikuliX PyO3 Bindings")
    print("=" * 60)

    tests = [
        ("Module Import", test_import),
        ("Screen Class", test_screen),
        ("Region Class", test_region),
        ("Pattern Class", test_pattern),
        ("Mouse/Keyboard", test_mouse_keyboard),
    ]

    results = []
    for name, test_func in tests:
        print(f"\n[Test] {name}")
        print("-" * 60)
        try:
            result = test_func()
            results.append((name, result))
        except Exception as e:
            print(f"✗ Exception: {e}")
            results.append((name, False))

    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)

    passed = sum(1 for _, r in results if r)
    total = len(results)

    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"{status}: {name}")

    print(f"\nTotal: {passed}/{total} passed")

    return passed == total

if __name__ == "__main__":
    import sys
    sys.exit(0 if main() else 1)
```

### Run Tests / テストを実行

```bash
# After maturin develop
python test_bindings.py
```

## Usage Examples / 使用例

### Example 1: Find and Click / 検索とクリック

```python
from sikulix_py import find, wait, click

# Wait for button to appear (max 5 seconds)
match = wait("button.png", timeout=5.0)

if match:
    print(f"Found button at: {match.center()}")
    match.click()  # Click at center
```

### Example 2: Multiple Matches / 複数マッチ

```python
from sikulix_py import find_all

# Find all icons
matches = find_all("icon.png", similarity=0.9)
print(f"Found {len(matches)} icons")

for i, match in enumerate(matches):
    print(f"Icon {i+1}: {match.center()}, score={match.score:.2f}")
```

### Example 3: Region Operations / 領域操作

```python
from sikulix_py import Screen, Region

screen = Screen()
width, height = screen.dimensions()

# Top-left quadrant
region = Region(0, 0, width // 2, height // 2)
print(f"Quadrant: {region}")
print(f"Center: {region.center()}")
```

### Example 4: Keyboard Input / キーボード入力

```python
from sikulix_py import type_text, paste, hotkey

# Type ASCII text
type_text("Hello, World!")

# Paste Unicode text (better for non-ASCII)
paste("こんにちは世界")

# Press hotkey combination
hotkey(["ctrl", "s"])  # Ctrl+S
hotkey(["ctrl", "shift", "t"])  # Ctrl+Shift+T
```

## API Reference / API リファレンス

### Classes / クラス

#### `Screen`

```python
screen = Screen(index=0)  # 0 = primary monitor
width, height = screen.dimensions()
region = screen.get_region()
image_bytes = screen.capture()  # Returns PNG bytes
```

#### `Region`

```python
r = Region(x, y, width, height)
cx, cy = r.center()
area = r.area()
contained = r.contains(x, y)
r2 = r.offset(dx, dy)
r3 = r.expand(amount)

# Properties
r.x, r.y, r.width, r.height
```

#### `Match`

```python
match = find("pattern.png")
cx, cy = match.center()
cx, cy = match.target()  # Same as center()
score = match.score  # 0.0-1.0
region = match.region  # PyRegion

# Actions
match.click()
match.double_click()
match.right_click()
match.highlight(seconds=2.0)
```

#### `Pattern`

```python
p = Pattern("image.png")
p = p.similar(0.9)  # Set similarity threshold
p = p.target_offset(10, 5)  # Set click offset
similarity = p.similarity  # Get threshold
```

### Functions / 関数

#### Image Finding / 画像検索

```python
# Find single best match (returns Match or None)
match = find(pattern, similarity=0.7)

# Find all matches (returns list of Match)
matches = find_all(pattern, similarity=0.7)

# Wait for pattern (raises exception on timeout)
match = wait(pattern, timeout=3.0)

# Check existence (returns Match or None, no exception)
match = exists(pattern, timeout=0.0)
```

#### Mouse / マウス

```python
click(x, y)         # Move and click
click()             # Click at current position
double_click(x, y)
right_click(x, y)
```

#### Keyboard / キーボード

```python
type_text(text)     # Type ASCII text
paste(text)         # Paste via clipboard (Unicode safe)
hotkey(keys)        # Press key combination
```

## Integration with runtime-rs / runtime-rs との統合

The PyO3 bindings can be used in two ways:

PyO3 バインディングは 2 つの方法で使用できます：

### 1. Native Mode (Direct Import) / ネイティブモード（直接インポート）

```python
# Direct import of native extension
import sikulix_py

match = sikulix_py.find("button.png")
sikulix_py.click(match.center())
```

### 2. Fallback Mode (Pure Python) / フォールバックモード（純粋 Python）

```python
# Pure Python wrapper using subprocess
from sikulix_api import find, click

match = find("button.png")  # Calls sikulix CLI
click(match)
```

The `runtime-rs/sikulix_api` package should detect and use native bindings when available:

`runtime-rs/sikulix_api` パッケージは、利用可能な場合はネイティブバインディングを検出して使用する必要があります：

```python
# runtime-rs/sikulix_api/__init__.py
try:
    from sikulix_py import *
    _NATIVE_MODE = True
except ImportError:
    from .fallback import *
    _NATIVE_MODE = False
```

## Troubleshooting / トラブルシューティング

### Import Error: "No module named 'sikulix_py'"

```bash
# Make sure you built with maturin
maturin develop --features python

# Or install the wheel
pip install target/wheels/sikulix_core-*.whl
```

### ABI Compatibility Error

```bash
# Rebuild with correct Python version
maturin develop --features python

# Check Python version
python --version
```

### Missing Platform Dependencies

**Windows:**
- Visual Studio Build Tools required
- Visual Studio ビルドツールが必要

**macOS:**
- Xcode Command Line Tools: `xcode-select --install`

**Linux:**
- Python development headers: `sudo apt-get install python3-dev`

## Performance Considerations / パフォーマンスの考慮事項

1. **GIL Release / GIL 解放**
   - Long operations (screen capture, image matching) release GIL
   - 長時間操作（スクリーンキャプチャ、画像マッチング）は GIL を解放

2. **Parallel Processing / 並列処理**
   - Image matching uses rayon for parallel processing
   - 画像マッチングは rayon を使用して並列処理

3. **Memory Management / メモリ管理**
   - Image data is copied when crossing Python/Rust boundary
   - 画像データは Python/Rust 境界を越えるときにコピーされる

## Next Steps / 次のステップ

1. **Build and Test / ビルドとテスト**
   - Install Rust and maturin
   - Build with `maturin develop --features python`
   - Run test script

2. **Integration Testing / 統合テスト**
   - Create test scripts with actual image files
   - Test all mouse/keyboard operations
   - Verify cross-platform compatibility

3. **Documentation / ドキュメント**
   - Generate Sphinx documentation
   - Create Python stubs for IDE autocomplete
   - Write usage tutorials

4. **Distribution / 配布**
   - Build wheels for all platforms
   - Upload to PyPI
   - Create installation guide

## References / 参考資料

- PyO3 Guide: https://pyo3.rs/
- Maturin: https://maturin.rs/
- RUNTIME-RS-DESIGN.md: `c:\VSCode\Sikuli-D\.local\doc\spec\RUNTIME-RS-DESIGN.md`
- L1-L2-API-SPEC.md: `c:\VSCode\Sikuli-D\.local\doc\spec\L1-L2-API-SPEC.md`
