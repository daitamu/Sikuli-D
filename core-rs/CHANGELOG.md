# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Performance Optimizations / パフォーマンス最適化

#### NCC Calculation / NCC計算
- Optimized memory access patterns for better cache locality
  - キャッシュ局所性向上のためのメモリアクセスパターン最適化
- Bounds-checked unsafe pixel access for faster computation
  - より高速な計算のための境界チェック済み unsafe ピクセルアクセス
- Pre-computed template statistics to eliminate redundant calculations
  - 冗長な計算を排除するための事前計算済みテンプレート統計
- Row-major access pattern for auto-vectorization
  - 自動ベクトル化のための行優先アクセスパターン

#### Non-Maximum Suppression / 非最大値抑制
- Memory allocation reduction using move semantics
  - 移動セマンティクスを使用したメモリ割り当ての削減
- Unstable sort for faster sorting of matches
  - マッチの高速ソートのための不安定ソート
- Early exit optimization for overlap calculation
  - 重複計算の早期終了最適化
- Pre-allocated result vectors with capacity estimation
  - 容量推定による結果ベクトルの事前割り当て

#### Compiler Optimizations / コンパイラ最適化
- Added `[profile.bench]` for benchmark builds with debug symbols
  - デバッグシンボル付きベンチマークビルド用の `[profile.bench]` 追加
- Maintained LTO and opt-level 3 for maximum performance
  - 最大パフォーマンスのため LTO と opt-level 3 を維持

### Benchmarks / ベンチマーク

#### Added Benchmark Suites / 追加されたベンチマークスイート
- `benches/matching.rs` - Image matching performance tests
  - 画像マッチングパフォーマンステスト
- `benches/screen_capture.rs` - Screen capture performance tests
  - 画面キャプチャパフォーマンステスト
- `benches/ncc_calculation.rs` - NCC calculation detailed benchmarks
  - NCC計算詳細ベンチマーク

#### Benchmark Coverage / ベンチマークカバレッジ
- Screen capture at various resolutions (800x600 to 3840x2160)
  - 様々な解像度での画面キャプチャ（800x600から3840x2160）
- NCC performance by template size (16x16 to 200x200)
  - テンプレートサイズ別NCC性能（16x16から200x200）
- Similarity threshold impact testing
  - 類似度閾値の影響テスト
- find() vs find_all() comparison
  - find() と find_all() の比較

### Documentation / ドキュメント

#### Performance Documentation / パフォーマンスドキュメント
- `PERFORMANCE.md` - Comprehensive performance optimization guide
  - 包括的なパフォーマンス最適化ガイド
- `BENCHMARK_RESULTS.md` - Template for recording benchmark results
  - ベンチマーク結果記録用テンプレート
- Updated `README.md` with performance section
  - パフォーマンスセクションを含むREADME.mdの更新
- Benchmark execution scripts (`run_benchmarks.sh`, `run_benchmarks.bat`)
  - ベンチマーク実行スクリプト（`run_benchmarks.sh`、`run_benchmarks.bat`）

### Performance Targets / パフォーマンス目標

| Operation | Before | Target | Status |
|-----------|--------|--------|--------|
| Screen capture (1920×1080) | - | < 50ms | 🎯 To be verified |
| Image matching (50×50) | - | < 100ms | 🎯 To be verified |
| NCC calculation | - | < 0.1ms/pos | 🎯 To be verified |
| NMS (100 matches) | - | < 10ms | 🎯 To be verified |

---

## [0.1.0] - 2025-11-26

### Added / 追加

#### Screen Module / スクリーンモジュール
- Screen capture for Windows, macOS, and Linux
  - Windows, macOS, Linux向けスクリーンキャプチャ
- Mouse control: move, click, double-click, right-click, middle-click, drag
  - マウス制御：移動、クリック、ダブルクリック、右クリック、中クリック、ドラッグ
- Smooth mouse movement with ease-in-out animation
  - イーズインアウトアニメーションによるスムーズなマウス移動
- Keyboard control: type text, key press/release, hotkey combinations
  - キーボード制御：テキスト入力、キー押下/解放、ホットキー組み合わせ
- Unicode text input support (Japanese, etc.)
  - Unicode テキスト入力サポート（日本語など）
- Special character escape sequences (\n, \t, \b, {KEY})
  - 特殊文字エスケープシーケンス (\n, \t, \b, {KEY})

#### Image Module / 画像モジュール
- Template matching for image recognition
  - テンプレートマッチングによる画像認識
- Pattern matching with configurable similarity threshold
  - 設定可能な類似度閾値によるパターンマッチング
- Multiple match detection
  - 複数マッチ検出
- OCR (Optical Character Recognition) via Tesseract
  - TesseractによるOCR（光学文字認識）
- Multi-language OCR support (English, Japanese, etc.)
  - 多言語OCRサポート（英語、日本語など）
- Region-based text extraction
  - 領域ベースのテキスト抽出

#### Python Module / Pythonモジュール
- Python environment detection (Python 2/3)
  - Python環境検出（Python 2/3）
- Script execution with output capture
  - 出力キャプチャ付きスクリプト実行
- Syntax analysis
  - 構文解析
- Execution state management
  - 実行状態管理

#### Debug Module / デバッグモジュール
- Breakpoint support
  - ブレークポイントサポート
- Step execution (into, over, out)
  - ステップ実行（イン、オーバー、アウト）
- Variable inspection
  - 変数表示
- Call stack tracking
  - コールスタック追跡

#### Settings Module / 設定モジュール
- Application settings management
  - アプリケーション設定管理
- Editor settings (theme, font, etc.)
  - エディタ設定（テーマ、フォント等）
- Execution settings
  - 実行設定
- Hotkey configuration with conflict detection
  - 競合検出付きホットキー設定
- Profile management
  - プロファイル管理

#### Plugin Module / プラグインモジュール
- Plugin loading and lifecycle management
  - プラグイン読み込みとライフサイクル管理
- Plugin dependency resolution
  - プラグイン依存関係解決
- Plugin event system
  - プラグインイベントシステム
- Permission model (13 permission types)
  - パーミッションモデル（13種類の権限）
- Auto-grant and manual approval permissions
  - 自動付与および手動承認パーミッション

#### Project Module / プロジェクトモジュール
- Project file structure (.sikuli)
  - プロジェクトファイル構造 (.sikuli)
- Image asset management
  - 画像アセット管理
- Project settings
  - プロジェクト設定

### Testing / テスト
- 99 unit tests passing
  - 99ユニットテストがパス
- 7 integration tests (with #[ignore])
  - 7統合テスト（#[ignore]付き）
- 7 doc tests passing
  - 7ドキュメントテストがパス

### Performance / パフォーマンス
- Release build optimizations (LTO enabled, opt-level=3)
  - リリースビルド最適化（LTO有効、opt-level=3）
- Zero clippy warnings
  - Clippy警告ゼロ

---

## Future / 今後の予定

### Planned for 1.0.0 / 1.0.0で予定
- [ ] macOS/Linux installer support
  - [ ] macOS/Linuxインストーラサポート
- [ ] Performance profiling and optimization
  - [ ] パフォーマンスプロファイリングと最適化
- [ ] Memory leak testing (24-hour run)
  - [ ] メモリリークテスト（24時間実行）
- [ ] Auto-update functionality
  - [ ] 自動更新機能

---

## Notes / 注意事項

- This is a pre-release version (0.x.x)
  - これはプレリリースバージョン（0.x.x）です
- API may change without notice before 1.0.0
  - 1.0.0より前にAPIが予告なく変更される場合があります
- For production use, please wait for 1.0.0 release
  - 本番使用は1.0.0リリースをお待ちください
