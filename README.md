# Sikuli-D (シクリッド / pronounced "sik-lid")

**A GUI automation tool with image recognition, built in Rust.**

**画像認識による GUI 自動化ツール。Rust で構築。**

---

## Key Features / 主な特徴

### English

- **Python 2/3 Dual Support** - Run legacy SikuliX scripts without modification
- **Built-in Python Runtime** - No separate Python installation required
- **Japanese Language Support** - Full Unicode support in logs, output, and scripts
- **Built with Rust** - Parallel image processing for better performance
- **Cross-platform** - Windows, macOS, Linux support
- **SikuliX Compatible** - We will continue to maintain the SikuliX API

### 日本語

- **Python 2/3 両対応** - レガシーな SikuliX スクリプトをそのまま実行可能
- **Python ランタイム内蔵** - 別途 Python のインストール不要
- **日本語完全対応** - ログ出力やスクリプトで日本語を使用してもエラーになりません
- **Rust 言語で開発** - 並列画像処理による高いパフォーマンス
- **クロスプラットフォーム** - Windows、macOS、Linux 対応
- **SikuliX 互換** - SikuliX の API は今後も維持し続けます

---

## Components / コンポーネント

Sikuli-D consists of two main modules:

Sikuli-D は 2 つの主要モジュールで構成されます：

### Sikuli-D IDE

A modern desktop application for creating and running automation scripts.

自動化スクリプトを作成・実行するためのモダンなデスクトップアプリケーション。

- Monaco-based script editor with syntax highlighting
- Visual screen capture with region selection
- Project management (.sikuli format)
- Plugin system for extensibility
- Japanese/English UI

See [ide-rs-tauri/README.md](ide-rs-tauri/README.md) for details.

### Sikuli-D Runtime

A standalone Python runtime for executing automation scripts.

自動化スクリプトを実行するためのスタンドアロン Python ランタイム。

- Python 2/3 automatic detection and conversion
- Built-in Python interpreter (no installation required)
- REPL mode for interactive development
- Headless execution support

See [runtime-rs/README.md](runtime-rs/README.md) for details.

---

## Quick Start / クイックスタート

### Using the IDE / IDE を使用する場合

```bash
# Download and run the installer
# インストーラーをダウンロードして実行

# Or build from source:
cd ide-rs-tauri
cargo tauri build
```

### Using the Runtime / ランタイムを使用する場合

```python
from sikulid_api import *

# Find and click an image on screen
click("button.png")

# Type text with Japanese support
type("こんにちは")

# Wait for an element to appear
wait("dialog.png", 10)
```

---

## Requirements / 動作要件

### For Users / 利用者向け

| Component | Requirement                            |
| --------- | -------------------------------------- |
| OS        | Windows 10+, macOS 10.15+, Linux (X11) |
| Tesseract | 5.x (optional, for OCR)                |

### For Developers / 開発者向け

| Component | Version       |
| --------- | ------------- |
| Rust      | 1.70+         |
| Node.js   | 18+ (for IDE) |

---

## Project Structure / プロジェクト構成

```
Sikuli-D/
├── ide-rs-tauri/  # Sikuli-D IDE (Tauri application)
├── runtime-rs/    # Sikuli-D Runtime (Python execution)
├── core-rs/       # Shared core library (internal)
└── pages/         # Documentation website
```

---

## Acknowledgments / 謝辞

This project is based on [SikuliX](https://github.com/RaiMan/SikuliX1), created by [RaiMan](https://github.com/RaiMan) and contributors. We deeply appreciate their work.

このプロジェクトは [RaiMan](https://github.com/RaiMan) 氏と貢献者によって作られた [SikuliX](https://github.com/RaiMan/SikuliX1) をベースにしています。

---

## License / ライセンス

MIT License - see [LICENSE](LICENSE) for details.

MIT ライセンス - 詳細は [LICENSE](LICENSE) を参照。

---

## Contributing / 貢献

Contributions are welcome! Feel free to open issues or submit pull requests.

貢献を歓迎します！Issue の作成や Pull Request の送信をお気軽にどうぞ。
