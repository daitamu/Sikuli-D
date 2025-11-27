# Sikuli-D IDE

**A modern desktop application for visual GUI automation.**

**視覚的なGUI自動化のためのモダンなデスクトップアプリケーション。**

---

## Features / 機能

- **Script Editor** - Monaco-based editor with Python syntax highlighting
- **Screen Capture** - Visual region selection with overlay UI
- **Project Management** - .sikuli project structure support
- **Built-in Runtime** - Execute scripts without external Python
- **Plugin System** - Extensible plugin architecture
- **Multi-language UI** - Japanese/English interface

  - **スクリプトエディタ** - Pythonシンタックスハイライト対応のMonacoベースエディタ
  - **スクリーンキャプチャ** - オーバーレイUIによる視覚的な領域選択
  - **プロジェクト管理** - .sikuliプロジェクト形式対応
  - **ランタイム内蔵** - 外部Pythonなしでスクリプト実行可能
  - **プラグインシステム** - 拡張可能なプラグインアーキテクチャ
  - **多言語UI** - 日本語/英語インターフェース

---

## Requirements / 動作要件

### For Users / 利用者向け

| OS | Requirement |
|----|-------------|
| Windows | Windows 10/11 (WebView2 included) |
| macOS | macOS 10.15+ (Screen Recording permission required) |
| Linux | WebKitGTK 4.1+, libappindicator |

### For Developers / 開発者向け

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Node.js | 18+ |
| Tauri CLI | 2.x |

---

## Building / ビルド

### Development / 開発

```bash
# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
cargo tauri dev
```

### Release / リリースビルド

```bash
# Build release binary
cargo tauri build
```

Output: `target/release/bundle/`

---

## Project Structure / プロジェクト構造

```
ide-rs-tauri/
├── src/              # Rust backend
│   ├── main.rs       # Application entry point
│   ├── capture.rs    # Screen capture
│   ├── settings.rs   # Settings management
│   └── plugins.rs    # Plugin management
├── dist/             # Web frontend
│   ├── index.html    # Main IDE window
│   ├── capture.html  # Capture overlay
│   ├── settings.html # Settings dialog
│   └── plugins.html  # Plugin manager
├── icons/            # Application icons
├── Cargo.toml        # Rust dependencies
└── tauri.conf.json   # Tauri configuration
```

---

## License / ライセンス

MIT License - see [LICENSE](../LICENSE) for details.

MITライセンス - 詳細は [LICENSE](../LICENSE) を参照。
