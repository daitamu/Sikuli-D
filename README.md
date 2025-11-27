# Sikuli-D (シクリッド / pronounced "sik-lid")

**Based on SikuliX 2.0.5, this version includes Japanese support and custom features.**

**SikuliX 2.0.5をベースに日本語対応や独自機能を追加したバージョンです。**

---

## Acknowledgments / 謝辞

This project is based on [SikuliX](https://github.com/RaiMan/SikuliX1), an amazing open-source GUI automation tool created by [RaiMan](https://github.com/RaiMan) and contributors.

We deeply appreciate the years of dedication and hard work that went into building SikuliX. Without their foundation, this project would not exist.

---

このプロジェクトは [RaiMan](https://github.com/RaiMan) 氏と多くの貢献者によって作られた素晴らしいオープンソースGUI自動化ツール [SikuliX](https://github.com/RaiMan/SikuliX1) をベースにしています。

SikuliXの構築に費やされた長年の献身と努力に深く感謝いたします。この基盤がなければ、本プロジェクトは存在しませんでした。

---

## What is Sikuli-D? / Sikuli-Dとは？

### English

Sikuli-D is a fork of SikuliX that automates anything you see on your desktop screen. It uses **image recognition** powered by OpenCV to identify GUI components and can interact with them using mouse and keyboard actions.

**Key Features:**
- High-performance Rust core with parallel image processing
- Native Python 3 scripting via PyO3 bindings
- Cross-platform support (Windows, macOS, Linux)
- OCR support via Tesseract 5
- Observer API for screen monitoring (appear, vanish, change detection)
- Mouse control (click, drag, scroll, mouseDown/Up)
- Keyboard control with Japanese/Unicode support
- SikuliX-compatible API design

### 日本語

Sikuli-Dは、デスクトップ画面上のあらゆる操作を自動化できるSikuliXのフォークです。Rustで書かれた高性能コアによる**画像認識**を使用してGUIコンポーネントを識別し、マウスやキーボード操作で制御できます。

**主な特徴：**
- 並列画像処理による高性能Rustコア
- PyO3バインディングによるネイティブPython 3スクリプティング
- クロスプラットフォーム対応（Windows、macOS、Linux）
- Tesseract 5によるOCRサポート
- 画面監視用Observer API（出現・消失・変化検出）
- マウス制御（クリック、ドラッグ、スクロール、mouseDown/Up）
- 日本語/Unicode対応キーボード制御
- SikuliX互換API設計

---

## Requirements / 動作要件

### Rust Core (Recommended) / Rustコア（推奨）

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Python | 3.8+ |
| Tesseract | 5.x (for OCR) |

### Legacy Java Version / レガシーJava版

| Component | Version |
|-----------|---------|
| Java | 17+ (LTS) |
| Maven | 3.6+ |

---

## Build / ビルド方法

### Rust Core / Rustコア

```bash
# Clone the repository / リポジトリをクローン
git clone https://github.com/daitamu/Sikuli-D.git
cd Sikuli-D

# Build core library / コアライブラリをビルド
cd core-rs
cargo build --release

# Build Python bindings / Pythonバインディングをビルド
pip install maturin
maturin build --release

# Run tests / テスト実行
cargo test
```

### Legacy Java Version / レガシーJava版

```bash
# Build all modules / 全モジュールをビルド
mvn clean install

# Build without tests / テストなしでビルド
mvn clean install -DskipTests
```

---

## Project Structure / プロジェクト構成

```
Sikuli-D/
├── API/           # Java Core library / Javaコアライブラリ
├── IDE/           # Java GUI IDE / Java GUI開発環境
├── core-rs/       # Rust Core library / Rustコアライブラリ
├── ide-rs-tauri/  # Rust/Tauri IDE / Rust/Tauri開発環境
├── runtime-rs/    # Python runtime / Pythonランタイム
├── Support/       # Support tools / サポートツール
├── pages/         # Documentation / ドキュメント
└── pom.xml        # Maven parent POM
```

### Rust Core (core-rs) / Rustコア

A next-generation core library written in Rust with:
- **High Performance** - Optimized image matching with parallel processing
- **Python 3 Bindings** - Native Python API via PyO3
- **Cross-platform** - Windows, macOS, Linux support
- **Observer API** - Screen region monitoring for GUI automation

Rustで書かれた次世代コアライブラリ：
- **高性能** - 並列処理による最適化された画像マッチング
- **Python 3バインディング** - PyO3によるネイティブPython API
- **クロスプラットフォーム** - Windows, macOS, Linux対応
- **Observer API** - GUI自動化のための画面領域監視

See [core-rs/README.md](core-rs/README.md) for details.
詳細は [core-rs/README.md](core-rs/README.md) を参照。

---

## License / ライセンス

This project is licensed under the **MIT License** - see the [LICENSE](LICENSE) file for details.

このプロジェクトは **MITライセンス** の下で公開されています。詳細は [LICENSE](LICENSE) ファイルをご覧ください。

---

## Original Project / 元プロジェクト

- **SikuliX**: https://github.com/RaiMan/SikuliX1
- **Documentation**: https://sikulix.github.io/

---

## Contributing / 貢献

Contributions are welcome! Feel free to open issues or submit pull requests.

貢献を歓迎します！Issueの作成やPull Requestの送信をお気軽にどうぞ。
