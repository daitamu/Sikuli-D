# Quick Build Guide for PyO3 Bindings
# PyO3 バインディングのクイックビルドガイド

## Prerequisites / 前提条件

### 1. Install Rust / Rust のインストール

**Windows:**
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# Or use PowerShell:
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

**macOS/Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. Install Maturin / Maturin のインストール

```bash
pip install maturin
```

### 3. Verify Installation / インストールの確認

```bash
rustc --version
cargo --version
maturin --version
python --version  # Should be 3.8 or higher
```

---

## Building / ビルド

### Option 1: Development Build (Recommended for Testing) / オプション 1: 開発ビルド（テストに推奨）

```bash
cd c:\VSCode\Sikuli-D\core-rs

# Build and install in current Python environment
maturin develop --features python

# This will:
# 1. Build the Rust extension
# 2. Install it in your Python site-packages
# 3. Make it immediately importable
```

### Option 2: Release Build (For Distribution) / オプション 2: リリースビルド（配布用）

```bash
cd c:\VSCode\Sikuli-D\core-rs

# Build optimized wheel
maturin build --release --features python

# Install the wheel
pip install target\wheels\sikulix_core-*.whl
```

---

## Testing / テスト

### Run Test Script / テストスクリプトを実行

```bash
cd c:\VSCode\Sikuli-D\core-rs
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

...

Total: 6/6 passed
```

### Manual Testing / 手動テスト

```python
# Start Python
python

# Import module
>>> import sikulix_py

# Test Screen
>>> screen = sikulix_py.Screen()
>>> width, height = screen.dimensions()
>>> print(f"Screen: {width}x{height}")

# Test Region
>>> region = sikulix_py.Region(100, 100, 200, 150)
>>> print(region)
Region(100, 100, 200, 150)
>>> cx, cy = region.center()
>>> print(f"Center: ({cx}, {cy})")
Center: (200, 175)
```

---

## Common Issues / よくある問題

### Issue: "pyo3-build-config" error

**Solution:**
```bash
# Make sure Python development headers are installed
# Windows: Should be installed with Python
# macOS: xcode-select --install
# Linux: sudo apt-get install python3-dev
```

### Issue: "link.exe not found" (Windows)

**Solution:**
Install Visual Studio Build Tools:
- Download from: https://visualstudio.microsoft.com/downloads/
- Select "Desktop development with C++"

### Issue: Module not found after build

**Solution:**
```bash
# Make sure you're using the same Python that maturin built for
which python  # or: where python

# Rebuild with specific Python
maturin develop --features python --python /path/to/python
```

---

## Platform-Specific Notes / プラットフォーム固有の注意事項

### Windows

- Requires Visual Studio Build Tools
- Use PowerShell or Command Prompt
- Path separators: use `\` or `/`

### macOS

- Requires Xcode Command Line Tools
- May need to sign the binary for distribution
- Apple Silicon requires `--target aarch64-apple-darwin`

### Linux

- Requires `python3-dev` package
- May need `libclang` for some dependencies
- Use system package manager for dependencies

---

## Cleaning / クリーンアップ

```bash
# Remove build artifacts
cargo clean

# Uninstall Python module
pip uninstall sikulix-py
```

---

## Next Steps After Successful Build / ビルド成功後の次のステップ

1. **Run full test suite:**
   ```bash
   python test_bindings.py
   ```

2. **Test with real images:**
   - Create test images
   - Try find(), wait(), click() operations

3. **Integrate with runtime-rs:**
   - Update `sikulix_api/__init__.py`
   - Test both native and fallback modes

4. **Cross-platform testing:**
   - Build on Windows, macOS, and Linux
   - Verify all tests pass on each platform

---

## Quick Reference Commands / クイックリファレンスコマンド

```bash
# One-time setup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
pip install maturin

# Development workflow
cd c:\VSCode\Sikuli-D\core-rs
maturin develop --features python
python test_bindings.py

# Release build
maturin build --release --features python
pip install target/wheels/sikulix_core-*.whl

# Clean
cargo clean
pip uninstall sikulix-py
```

---

**For detailed documentation, see:** `README_PyO3.md`
**詳細なドキュメントについては、** `README_PyO3.md` **を参照してください**
