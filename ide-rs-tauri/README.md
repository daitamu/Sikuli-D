# sikulix-ide-tauri

**SikuliX IDE - Next Generation Desktop Application**

**SikuliX IDE - 次世代デスクトップアプリケーション**

---

A modern, cross-platform IDE for visual GUI automation, built with Tauri and Rust.

Tauri と Rust で構築された、モダンでクロスプラットフォーム対応の GUI 自動化 IDE。

## Features / 機能

- **Script Editor** - Monaco-based code editor with syntax highlighting
  - **スクリプトエディタ** - Monaco ベースのコードエディタ（シンタックスハイライト対応）
- **Screen Capture** - Visual region selection with overlay UI
  - **スクリーンキャプチャ** - オーバーレイ UI による領域選択
- **Project Management** - .sikuli project structure support
  - **プロジェクト管理** - .sikuli プロジェクト構造対応
- **Settings Panel** - Comprehensive settings with profiles and hotkeys
  - **設定パネル** - プロファイル・ホットキー対応の総合設定
- **Plugin System** - Install, configure, and manage plugins
  - **プラグインシステム** - プラグインのインストール・設定・管理
- **Multi-language UI** - Japanese/English internationalization
  - **多言語 UI** - 日本語/英語の国際化対応

## Requirements / 動作要件

| Component | Version |
|-----------|---------|
| Rust | 1.70+ |
| Node.js | 18+ (for Tauri CLI) |
| Tauri CLI | 2.x |

### Platform-specific / プラットフォーム固有

**Windows:**
- Windows 10/11
- WebView2 Runtime (included with Windows 11)

**macOS:**
- macOS 10.15+
- Screen Recording permission required for capture

**Linux:**
- WebKitGTK 4.1+
- libappindicator (for system tray)

## Building / ビルド

### Development / 開発

```bash
# Install Tauri CLI
cargo install tauri-cli

# Run in development mode
cargo tauri dev
```

### Release / リリース

```bash
# Build release binary
cargo tauri build
```

Output will be in `target/release/bundle/`.

## Project Structure / プロジェクト構造

```
ide-rs-tauri/
├── src/
│   ├── main.rs       # Application entry point / アプリケーションエントリーポイント
│   ├── capture.rs    # Screen capture commands / スクリーンキャプチャコマンド
│   ├── settings.rs   # Settings management / 設定管理
│   └── plugins.rs    # Plugin management / プラグイン管理
├── dist/
│   ├── index.html    # Main IDE window / メイン IDE ウィンドウ
│   ├── capture.html  # Capture overlay / キャプチャオーバーレイ
│   ├── settings.html # Settings dialog / 設定ダイアログ
│   └── plugins.html  # Plugin manager / プラグインマネージャ
├── icons/            # Application icons / アプリケーションアイコン
├── Cargo.toml        # Rust dependencies / Rust 依存関係
└── tauri.conf.json   # Tauri configuration / Tauri 設定
```

## Tauri Commands / Tauri コマンド

### File Operations / ファイル操作
- `open_file` - Open file dialog / ファイル選択ダイアログ
- `save_file` - Save with dialog / 保存ダイアログ
- `read_file` - Read file content / ファイル読み込み
- `write_file` - Write file content / ファイル書き込み

### Script Execution / スクリプト実行
- `run_script` - Execute Python script / Python スクリプト実行
- `analyze_python_version` - Detect Python version / Python バージョン検出

### Screen Capture / スクリーンキャプチャ
- `start_capture` - Start capture overlay / キャプチャ開始
- `complete_capture` - Save captured region / キャプチャ完了
- `cancel_capture` - Cancel capture / キャプチャキャンセル

### Settings / 設定
- `get_settings` / `save_settings` - Settings CRUD / 設定の読み書き
- `get_profiles` / `create_profile` - Profile management / プロファイル管理
- `get_hotkeys` / `set_hotkey` - Hotkey configuration / ホットキー設定

### Plugins / プラグイン
- `get_plugins` - List installed plugins / プラグイン一覧
- `enable_plugin` / `disable_plugin` - Toggle plugin / プラグイン有効化
- `install_plugin_from_file` - Install from ZIP / ZIP からインストール
- `uninstall_plugin` - Remove plugin / プラグイン削除
- `get_plugin_settings` / `save_plugin_settings` - Plugin config / プラグイン設定

## Configuration / 設定

### tauri.conf.json

Key configuration options:

```json
{
  "productName": "SikuliX IDE",
  "version": "0.1.0",
  "bundle": {
    "targets": ["msi", "nsis"]
  },
  "app": {
    "windows": [{
      "title": "SikuliX IDE",
      "width": 1200,
      "height": 800
    }]
  }
}
```

## License / ライセンス

MIT License - see [LICENSE](../LICENSE) for details.

MITライセンス - 詳細は [LICENSE](../LICENSE) を参照してください。
