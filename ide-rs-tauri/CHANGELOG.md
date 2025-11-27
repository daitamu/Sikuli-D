# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2025-11-26

### Added / 追加

#### Main Window / メインウィンドウ
- Monaco-based code editor with syntax highlighting
  - シンタックスハイライト付きMonacoベースコードエディタ
- File operations (new, open, save, save as)
  - ファイル操作（新規、開く、保存、名前を付けて保存）
- Recent files management
  - 最近使用したファイルの管理
- Script execution with output capture
  - 出力キャプチャ付きスクリプト実行
- Multi-tab support
  - マルチタブ対応

#### Screen Capture / スクリーンキャプチャ
- Overlay-based region selection UI
  - オーバーレイベースの領域選択UI
- Real-time coordinate display
  - リアルタイム座標表示
- Crosshair guides for precise selection
  - 精密な選択のためのクロスヘアガイド
- Keyboard shortcuts (Escape to cancel)
  - キーボードショートカット（Escapeでキャンセル）

#### Settings Dialog / 設定ダイアログ
- General settings (language, theme, auto-save)
  - 一般設定（言語、テーマ、自動保存）
- Editor settings (font, tab size, line numbers)
  - エディタ設定（フォント、タブサイズ、行番号）
- Execution settings (timeout, similarity)
  - 実行設定（タイムアウト、類似度）
- Hotkey configuration with conflict detection
  - 競合検出付きホットキー設定
- Profile management (create, switch, delete)
  - プロファイル管理（作成、切り替え、削除）

#### Plugin Manager / プラグインマネージャ
- Plugin listing with status display
  - ステータス表示付きプラグイン一覧
- Enable/disable plugin toggle
  - プラグイン有効/無効切り替え
- Plugin installation from ZIP files
  - ZIPファイルからのプラグインインストール
- Plugin uninstallation with confirmation
  - 確認付きプラグインアンインストール
- Plugin settings dialog (general, permissions, about)
  - プラグイン設定ダイアログ（一般、パーミッション、概要）
- Permission management UI
  - パーミッション管理UI

#### Internationalization / 国際化
- Japanese/English UI support
  - 日本語/英語UI対応
- Dynamic language switching
  - 動的な言語切り替え

### Technical / 技術的詳細

#### Tauri Commands / Tauriコマンド
- File operations: `open_file`, `save_file`, `read_file`, `write_file`
- Script execution: `run_script`, `analyze_python_version`
- Capture: `start_capture`, `complete_capture`, `cancel_capture`
- Settings: `get_settings`, `save_settings`, `get_profiles`, etc.
- Plugins: `get_plugins`, `enable_plugin`, `install_plugin_from_file`, etc.

#### Dependencies / 依存関係
- Tauri 2.x
- tauri-plugin-fs
- tauri-plugin-dialog
- tauri-plugin-shell
- sikulix-core (workspace)
- zip (for plugin installation)

### Performance / パフォーマンス
- Zero clippy warnings
  - Clippy警告ゼロ
- Proper code formatting (rustfmt)
  - 適切なコードフォーマット（rustfmt）

---

## Future / 今後の予定

### Planned for 1.0.0 / 1.0.0で予定
- [ ] Image library panel
  - [ ] 画像ライブラリパネル
- [ ] Pattern editor dialog
  - [ ] パターンエディタダイアログ
- [ ] Debug panel (breakpoints, variables)
  - [ ] デバッグパネル（ブレークポイント、変数）
- [ ] Log panel with real-time output
  - [ ] リアルタイム出力付きログパネル
- [ ] macOS/Linux installer support
  - [ ] macOS/Linuxインストーラサポート

---

## Notes / 注意事項

- This is a pre-release version (0.x.x)
  - これはプレリリースバージョン（0.x.x）です
- UI and features may change before 1.0.0
  - 1.0.0より前にUIや機能が変更される場合があります
