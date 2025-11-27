# PyO3 Bindings Implementation Summary
# PyO3 バインディング実装サマリー

**Date / 日付**: 2025-11-27
**Task**: Wave 2 Task 3-2A: PyO3バインディングの実装
**Status / ステータス**: ✅ Implementation Complete (Build and Test Required)

---

## Overview / 概要

Implemented PyO3 bindings to expose core-rs functionality to Python, enabling native Python integration with Sikuli-D runtime.

core-rs の機能を Python に公開する PyO3 バインディングを実装し、Sikuli-D ランタイムとのネイティブ Python 統合を可能にしました。

---

## Implementation Details / 実装詳細

### 1. Files Created / 作成されたファイル

#### Primary Implementation / 主要実装

- **`core-rs/src/python/bindings.rs`** (676 lines)
  - Complete PyO3 bindings implementation
  - 完全な PyO3 バインディング実装

#### Documentation / ドキュメント

- **`core-rs/src/python/README_PyO3.md`**
  - Comprehensive build, test, and usage guide
  - 包括的なビルド、テスト、使用ガイド

- **`core-rs/PyO3_IMPLEMENTATION_SUMMARY.md`** (this file)
  - Implementation summary and status
  - 実装サマリーとステータス

#### Testing / テスト

- **`core-rs/test_bindings.py`**
  - Python test script for validating bindings
  - バインディングを検証する Python テストスクリプト

### 2. Files Modified / 変更されたファイル

- **`core-rs/Cargo.toml`**
  - Updated PyO3 dependency: removed `auto-initialize` feature, kept `extension-module`
  - PyO3 依存関係を更新：`auto-initialize` 機能を削除、`extension-module` を保持

- **`core-rs/src/python/mod.rs`**
  - Added `pub mod bindings;` with `#[cfg(feature = "python")]`
  - `#[cfg(feature = "python")]` で `pub mod bindings;` を追加

---

## Implementation Architecture / 実装アーキテクチャ

### Class Wrappers / クラスラッパー

#### 1. PyScreen
```rust
#[pyclass(name = "Screen")]
struct PyScreen {
    inner: Screen,
}
```

**Methods / メソッド:**
- `new(index: Option<u32>) -> Self`
- `capture(&self, py: Python) -> PyResult<Vec<u8>>`
- `dimensions(&mut self) -> PyResult<(u32, u32)>`
- `get_region(&mut self) -> PyResult<PyRegion>`

**Features / 機能:**
- GIL released during capture with `py.allow_threads()`
- キャプチャ中に `py.allow_threads()` で GIL を解放

---

#### 2. PyRegion
```rust
#[pyclass(name = "Region")]
#[derive(Clone)]
struct PyRegion {
    inner: Region,
}
```

**Methods / メソッド:**
- `new(x, y, width, height) -> Self`
- `center() -> (i32, i32)`
- `contains(x, y) -> bool`
- `area() -> u64`
- `offset(dx, dy) -> Self`
- `expand(amount) -> Self`

**Properties / プロパティ:**
- `x`, `y`, `width`, `height` (read-only getters)

**Special Methods / 特殊メソッド:**
- `__repr__()` for string representation

---

#### 3. PyMatch
```rust
#[pyclass(name = "Match")]
#[derive(Clone)]
struct PyMatch {
    inner: Match,
}
```

**Methods / メソッド:**
- `center() -> (i32, i32)`
- `target() -> (i32, i32)` (alias for center)
- `click() -> PyResult<()>`
- `double_click() -> PyResult<()>`
- `right_click() -> PyResult<()>`
- `highlight(seconds: Option<f64>) -> PyResult<()>`

**Properties / プロパティ:**
- `score: f64` (read-only)
- `region: PyRegion` (read-only)

**Special Methods / 特殊メソッド:**
- `__repr__()`

---

#### 4. PyPattern
```rust
#[pyclass(name = "Pattern")]
struct PyPattern {
    inner: Pattern,
}
```

**Methods / メソッド:**
- `new(path: String) -> PyResult<Self>`
- `similar(similarity: f64) -> PyRefMut<Self>` (builder pattern)
- `target_offset(x, y) -> PyRefMut<Self>` (builder pattern)

**Properties / プロパティ:**
- `similarity: f64` (read-only)

**Special Methods / 特殊メソッド:**
- `__repr__()`

---

### Module-Level Functions / モジュールレベル関数

#### Image Finding / 画像検索

1. **`find(pattern, similarity) -> Option<PyMatch>`**
   - Find single best match
   - 最良の単一マッチを検索
   - GIL released during processing

2. **`find_all(pattern, similarity) -> Vec<PyMatch>`**
   - Find all occurrences
   - 全ての出現箇所を検索
   - GIL released during processing

3. **`wait(pattern, timeout) -> PyMatch`**
   - Wait for pattern with timeout
   - タイムアウト付きでパターンを待機
   - Raises exception on timeout
   - GIL released during processing

4. **`exists(pattern, timeout) -> Option<PyMatch>`**
   - Non-throwing variant of wait
   - wait の例外を投げないバリアント
   - Returns None on timeout
   - GIL released during processing

#### Mouse Operations / マウス操作

1. **`click(x: Option<i32>, y: Option<i32>) -> PyResult<()>`**
   - Click at coordinates or current position
   - 座標または現在位置でクリック

2. **`double_click(x, y) -> PyResult<()>`**
   - Double click
   - ダブルクリック

3. **`right_click(x, y) -> PyResult<()>`**
   - Right click
   - 右クリック

#### Keyboard Operations / キーボード操作

1. **`type_text(text: &str) -> PyResult<()>`**
   - Type ASCII text
   - ASCII テキストを入力

2. **`paste(text: &str) -> PyResult<()>`**
   - Paste via clipboard (Unicode safe)
   - クリップボード経由でペースト（Unicode セーフ）

3. **`hotkey(keys: Vec<String>) -> PyResult<()>`**
   - Press key combination
   - キーの組み合わせを押下
   - Supports modifiers: ctrl, shift, alt, meta
   - Function keys: f1-f12
   - Navigation keys: enter, tab, escape, etc.
   - Letter and number keys: a-z, 0-9

---

### Error Conversion / エラー変換

```rust
fn to_pyerr(err: SikulixError) -> PyErr
```

**Mapping / マッピング:**

| Rust Error | Python Exception | Description |
|------------|------------------|-------------|
| `ImageNotFound` | `PyRuntimeError` | Image not found on screen |
| `ImageLoadError` | `PyValueError` | Failed to load image file |
| `ScreenCaptureError` | `PyRuntimeError` | Screen capture failed |
| `FindFailed` | `PyRuntimeError` | Pattern not found within timeout |
| `MouseError` | `PyRuntimeError` | Mouse operation failed |
| `KeyboardError` | `PyRuntimeError` | Keyboard operation failed |
| `IoError` | `PyIOError` | File I/O error |

---

### GIL Management / GIL 管理

All potentially blocking operations release the GIL using `py.allow_threads()`:

すべての潜在的にブロッキングする操作は `py.allow_threads()` を使用して GIL を解放します：

- Screen capture / スクリーンキャプチャ
- Image matching (find, find_all, wait, exists) / 画像マッチング
- Wait operations / 待機操作

This allows other Python threads to run concurrently during these operations.

これにより、これらの操作中に他の Python スレッドが並行して実行できます。

---

## Key Design Decisions / 主要な設計決定

### 1. Builder Pattern for PyPattern
Pattern configuration uses builder pattern for chaining:
```python
p = Pattern("image.png").similar(0.9).target_offset(10, 5)
```

### 2. Optional Parameters
Functions accept `Option<T>` for flexibility:
```python
find("image.png")  # Use default similarity
find("image.png", similarity=0.95)  # Custom similarity
```

### 3. Consistent Error Handling
All errors converted to appropriate Python exceptions with descriptive messages.

すべてのエラーは説明的なメッセージを持つ適切な Python 例外に変換されます。

### 4. Thread Safety
GIL is released for long-running operations to allow Python-level concurrency.

長時間実行される操作では GIL が解放され、Python レベルの並行性が可能になります。

---

## Build Configuration / ビルド設定

### Cargo.toml Configuration

```toml
[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
pyo3 = { version = "0.21", features = ["extension-module"], optional = true }

[features]
python = ["pyo3"]
```

### Build Commands / ビルドコマンド

```bash
# Install maturin
pip install maturin

# Development build
cd core-rs
maturin develop --features python

# Release build
maturin build --release --features python

# Install wheel
pip install target/wheels/sikulix_core-*.whl
```

---

## Testing / テスト

### Test Script: `test_bindings.py`

Tests implemented:
1. Module import test
2. Screen class test (dimensions, capture, get_region)
3. Region class test (center, contains, offset, expand, area)
4. Pattern class test (file loading)
5. Mouse/Keyboard function availability
6. Error handling test

### Running Tests / テストの実行

```bash
# After maturin develop
python test_bindings.py
```

Expected output:
```
============================================================
Testing SikuliX PyO3 Bindings
============================================================

[Test] Module Import
------------------------------------------------------------
✓ Module imported successfully

[Test] Screen Class
------------------------------------------------------------
✓ Screen created: <Screen object>
✓ Screen dimensions: 1920x1080
✓ Screen region: Region(0, 0, 1920, 1080)

[Test] Region Class
------------------------------------------------------------
✓ Region created: Region(100, 100, 200, 150)
✓ Center: (200, 175)
✓ Properties: x=100, y=100, w=200, h=150
✓ Offset region: Region(150, 150, 200, 150)
✓ Expanded region: Region(90, 90, 220, 170)
✓ Contains test passed
✓ Area: 30000

...

Total: 6/6 passed
```

---

## Integration with Runtime-RS / Runtime-RS との統合

The bindings can be used in runtime-rs in two modes:

バインディングは runtime-rs で 2 つのモードで使用できます：

### 1. Native Mode (Preferred) / ネイティブモード（推奨）

Direct import of PyO3 extension:
```python
import sikulix_py

match = sikulix_py.find("button.png")
match.click()
```

### 2. Fallback Mode / フォールバックモード

When native bindings are not available, use subprocess-based implementation:
```python
from sikulix_api import find, click

match = find("button.png")  # Calls sikulix CLI
click(match)
```

### Auto-Detection / 自動検出

`runtime-rs/sikulix_api/__init__.py` should implement:

```python
try:
    from sikulix_py import *
    _NATIVE_MODE = True
    print("[SikuliX] Using native (PyO3) bindings", file=sys.stderr)
except ImportError:
    from .fallback import *
    _NATIVE_MODE = False
    print("[SikuliX] Native bindings not available, using fallback mode", file=sys.stderr)
```

---

## Performance Characteristics / パフォーマンス特性

### Advantages / 利点

1. **Native Speed / ネイティブ速度**
   - Direct Rust execution, no subprocess overhead
   - 直接 Rust 実行、サブプロセスのオーバーヘッドなし

2. **Parallel Processing / 並列処理**
   - Image matching uses rayon for multi-core processing
   - 画像マッチングは rayon を使用してマルチコア処理

3. **GIL Release / GIL 解放**
   - Long operations don't block Python threads
   - 長時間操作は Python スレッドをブロックしない

### Considerations / 考慮事項

1. **Memory Copying / メモリコピー**
   - Image data is copied when crossing Python/Rust boundary
   - 画像データは Python/Rust 境界を越えるときにコピーされる
   - Acceptable for typical use cases
   - 一般的なユースケースでは許容範囲

2. **Platform Compatibility / プラットフォーム互換性**
   - Requires Rust toolchain for building
   - ビルドには Rust ツールチェーンが必要
   - Pre-built wheels can be distributed for common platforms
   - 一般的なプラットフォーム向けにビルド済みホイールを配布可能

---

## Next Steps / 次のステップ

### Immediate / 即時

1. **Install Rust and Build Tools / Rust とビルドツールのインストール**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   pip install maturin
   ```

2. **Build and Test / ビルドとテスト**
   ```bash
   cd core-rs
   maturin develop --features python
   python test_bindings.py
   ```

3. **Verify All Tests Pass / すべてのテストが通ることを確認**
   - Fix any compilation errors
   - Ensure all tests pass

### Short-Term / 短期

1. **Integration Testing / 統合テスト**
   - Create test scripts with actual image files
   - 実際の画像ファイルを使用したテストスクリプトを作成
   - Test mouse/keyboard operations in controlled environment
   - 制御された環境でマウス/キーボード操作をテスト

2. **Cross-Platform Validation / クロスプラットフォーム検証**
   - Build and test on Windows, macOS, Linux
   - Windows、macOS、Linux でビルドとテスト

3. **Runtime-RS Integration / Runtime-RS 統合**
   - Update `runtime-rs/sikulix_api/__init__.py` to detect native bindings
   - ネイティブバインディングを検出するように `runtime-rs/sikulix_api/__init__.py` を更新
   - Implement fallback mode for compatibility
   - 互換性のためのフォールバックモードを実装

### Long-Term / 長期

1. **Documentation / ドキュメント**
   - Generate Sphinx documentation
   - Python type stubs for IDE autocomplete (`.pyi` files)
   - Usage tutorials and examples

2. **Distribution / 配布**
   - Build wheels for all platforms (Windows, macOS, Linux)
   - Upload to PyPI
   - Create installation guide

3. **Performance Optimization / パフォーマンス最適化**
   - Profile and optimize hot paths
   - Consider zero-copy for large images
   - Benchmark against SikuliX Java

---

## Known Limitations / 既知の制限事項

1. **Requires Rust Toolchain / Rust ツールチェーンが必要**
   - Users building from source need Rust installed
   - ソースからビルドするユーザーは Rust をインストールする必要がある
   - Solution: Distribute pre-built wheels
   - 解決策：ビルド済みホイールを配布

2. **Python Version Compatibility / Python バージョン互換性**
   - PyO3 0.21 supports Python 3.8+
   - PyO3 0.21 は Python 3.8+ をサポート
   - Older Python versions require fallback mode
   - 古い Python バージョンはフォールバックモードが必要

3. **Platform-Specific Dependencies / プラットフォーム固有の依存関係**
   - Windows: Visual Studio Build Tools
   - macOS: Xcode Command Line Tools
   - Linux: Python development headers

---

## Compliance with Design Specification / 設計仕様への準拠

Implementation fully complies with RUNTIME-RS-DESIGN.md:

実装は RUNTIME-RS-DESIGN.md に完全に準拠しています：

✅ PyScreen with capture, dimensions, get_region
✅ PyRegion with all geometric operations
✅ PyMatch with click actions and highlighting
✅ PyPattern with builder pattern (similar, target_offset)
✅ Module-level functions: find, find_all, wait, exists
✅ Mouse functions: click, double_click, right_click
✅ Keyboard functions: type_text, paste, hotkey
✅ Error conversion: SikulixError → PyErr
✅ GIL management: allow_threads for long operations
✅ Thread safety: Arc/Mutex not needed (PyO3 handles it)

---

## File Locations / ファイルの場所

```
c:\VSCode\Sikuli-D\
├── core-rs\
│   ├── src\
│   │   └── python\
│   │       ├── mod.rs (modified)
│   │       ├── bindings.rs (NEW)
│   │       └── README_PyO3.md (NEW)
│   ├── Cargo.toml (modified)
│   ├── test_bindings.py (NEW)
│   └── PyO3_IMPLEMENTATION_SUMMARY.md (NEW - this file)
└── .local\
    └── doc\
        └── spec\
            ├── RUNTIME-RS-DESIGN.md (reference)
            └── L1-L2-API-SPEC.md (reference)
```

---

## Conclusion / 結論

The PyO3 bindings implementation is **complete and ready for build and test**. All required classes, functions, and error handling have been implemented according to the design specification.

PyO3 バインディングの実装は **完了し、ビルドとテストの準備ができています**。設計仕様に従って、すべての必要なクラス、関数、エラーハンドリングが実装されました。

The next critical step is to install Rust/Cargo and run the build process to verify compilation and functionality.

次の重要なステップは、Rust/Cargo をインストールし、ビルドプロセスを実行してコンパイルと機能を検証することです。

---

**Implementation Status / 実装ステータス**: ✅ Complete
**Build Status / ビルドステータス**: ⏳ Pending (requires Rust installation)
**Test Status / テストステータス**: ⏳ Pending (requires build)

---

**END OF SUMMARY / サマリー終了**
