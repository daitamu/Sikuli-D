# Sikuli-D Core

**Internal core library for Sikuli-D IDE and Runtime.**

**Sikuli-D IDEとRuntimeの内部コアライブラリ。**

> This is an internal library. For user documentation, see the main [README](../README.md).
>
> これは内部ライブラリです。ユーザードキュメントはメインの[README](../README.md)を参照してください。

---

## Modules / モジュール

| Module | Description |
|--------|-------------|
| `screen` | Screen capture, Mouse, Keyboard control |
| `image` | Image matching (template matching, NCC) |
| `ocr` | OCR via Tesseract |
| `observer` | Screen region monitoring |
| `python` | PyO3 bindings for Python API |
| `settings` | Configuration management |
| `debug` | Debugging support |
| `plugin` | Plugin system |

---

## Features / 機能

- **High Performance** - Parallel image processing with Rayon
  - **高性能** - Rayonによる並列画像処理
- **Cross-platform** - Windows, macOS, Linux support
  - **クロスプラットフォーム** - Windows、macOS、Linux対応
- **Python Bindings** - Native Python 3 API via PyO3
  - **Pythonバインディング** - PyO3によるネイティブPython 3 API
- **Observer API** - Screen monitoring (appear, vanish, change)
  - **Observer API** - 画面監視（出現・消失・変化検出）

---

## Building / ビルド

```bash
# Build library
cargo build --release

# Run tests
cargo test

# Build Python bindings
pip install maturin
maturin build --release
```

---

## License / ライセンス

MIT License - see [LICENSE](../LICENSE) for details.

MITライセンス - 詳細は [LICENSE](../LICENSE) を参照。
