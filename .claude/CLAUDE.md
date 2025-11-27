# Sikuli-D プロジェクト開発ルール

## プロジェクト概要

画像認識による GUI 自動化ツール。Rust で構築。
SikuliX API 互換で、Python 2/3 スクリプトをそのまま実行可能。

---

## 分散並列エージェント活用ルール

### 基本方針

- 複雑なタスクは積極的にサブエージェント（Task tool）を活用して並列処理
- 独立したタスクは同時に複数のエージェントを起動
- コードベース探索には `subagent_type=Explore` を使用

### エージェント使用ガイドライン

#### 並列実行すべきケース

- 複数ファイルの同時調査
- 独立した機能の同時実装
- テストと実装の並行作業
- ドキュメント調査と実装の並行

#### 逐次実行すべきケース

- 依存関係のあるタスク
- 前のタスクの結果が必要な場合

### エージェントタイプの選択

- `Explore`: コードベース探索、ファイル検索、構造理解
- `Plan`: 実装計画の策定
- `general-purpose`: 複雑なマルチステップタスク

---

## プロジェクト構造

```
Sikuli-D/
├── core-rs/            # Sikuli-D Core（共有コアライブラリ）
│   ├── src/            # Rustソースコード
│   └── Cargo.toml      # Rust依存関係
├── ide-rs-tauri/       # Sikuli-D IDE（Tauriデスクトップアプリ）
│   ├── src/            # Rustバックエンド
│   ├── dist/           # Webフロントエンド
│   └── tauri.conf.json # Tauri設定
├── runtime-rs/         # Sikuli-D Runtime（Python実行環境）
│   ├── src/            # Rustソースコード
│   └── sikulid_api/    # Pythonラッパーモジュール
├── pages/              # ドキュメントページ
├── VERSION             # バージョンファイル（自動更新）
├── LICENSE             # MITライセンス
└── README.md           # プロジェクト説明
```

---

## 技術スタック

- **言語**: Rust 1.70+
- **GUI フレームワーク**: Tauri 2.x
- **Python バインディング**: PyO3
- **画像認識**: OpenCV (image crate)
- **OCR**: Tesseract 5
- **スクリプト**: Python 2/3（自動変換対応）

---

## ビルドコマンド

### Core ライブラリビルド

```bash
cd core-rs
cargo build --release
```

### IDE ビルド（Tauri）

```bash
cd ide-rs-tauri
cargo tauri build
```

### Runtime ビルド（Python バインディング）

```bash
cd runtime-rs
pip install maturin
maturin build --release
```

### テスト実行

```bash
cargo test                    # 全テスト
cargo test -p sikulid-core    # core-rsのみ
```

### フォーマット・静的解析

```bash
cargo fmt --check
cargo clippy
```

---

## DOD (Definition of Done) ワークフロー

### フェーズ終了時の必須チェック

#### 1. コードレビュー

- 実装したコードの品質確認
- セキュリティ脆弱性チェック（OWASP Top 10）
- コーディング規約の遵守確認
- 不要なコード・コメントの削除

#### 2. ビルド検証

- `cargo build --release` が成功する
- ビルドエラー・警告の解消

#### 3. テスト検証

- `cargo test` が成功する
- 新機能には対応するテストを追加

### フェーズ完了の定義

以下が全て満たされた場合にフェーズ完了とする：

- [ ] コードが正しくビルドされる
- [ ] テストが全て通る
- [ ] 警告が許容範囲内
- [ ] セキュリティ問題がない
- [ ] 不要なコードが削除されている
- [ ] コミット可能な状態

---

## コミットルール / Commit Rules

### Conventional Commits 形式

バージョンは GitHub Actions で自動管理されます。
コミットメッセージは **Conventional Commits** 形式で記述してください：

Version is automatically managed by GitHub Actions.
Use **Conventional Commits** format for commit messages:

### コミットタイプ / Commit Types

| タイプ             | 説明                       | バージョン変更 |
| ------------------ | -------------------------- | -------------- |
| `feat:`            | 新機能追加                 | MINOR +1       |
| `fix:`             | バグ修正                   | PATCH +1       |
| `feat!:` or `fix!:`| 破壊的変更                 | MAJOR +1       |
| `BREAKING CHANGE:` | 破壊的変更（本文に記載）   | MAJOR +1       |
| `docs:`            | ドキュメントのみ           | 変更なし       |
| `style:`           | フォーマット変更           | 変更なし       |
| `refactor:`        | リファクタリング           | 変更なし       |
| `test:`            | テスト追加・修正           | 変更なし       |
| `chore:`           | ビルド・ツール変更         | 変更なし       |

### メッセージ形式 / Message Format

```
<type>: <English summary>
<type>: <日本語の要約>

- English detail point
- Another English point

  日本語の詳細ポイント
  別の日本語ポイント

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### 例 / Examples

```
feat: Add Observer API for screen monitoring
feat: スクリーン監視用のObserver APIを追加

- Implement onAppear, onVanish, onChange handlers
- Add observe() method with timeout

  onAppear, onVanish, onChangeハンドラを実装
  タイムアウト付きのobserve()メソッドを追加

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

```
fix: Resolve Japanese text encoding in logs
fix: ログでの日本語テキストエンコーディングを修正

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## バージョン管理 / Version Management

### 自動バージョニング / Automatic Versioning

バージョンは **GitHub Actions** で **Conventional Commits** に基づいて自動管理されます。

Version is automatically managed by **GitHub Actions** based on **Conventional Commits**.

### バージョン形式: x.y.z (Semantic Versioning)

| 区分      | 変更トリガー                    | 自動判定基準              |
| --------- | ------------------------------- | ------------------------- |
| x (MAJOR) | 破壊的変更                      | `feat!:` or `BREAKING CHANGE:` |
| y (MINOR) | 新機能追加                      | `feat:` コミット          |
| z (PATCH) | バグ修正                        | `fix:` コミット           |

### 動作の仕組み

1. `master` ブランチへのプッシュ時に `.github/workflows/semantic-version.yml` が実行
2. コミットメッセージを解析してバージョンを自動計算
3. 以下のファイルを自動更新：
   - `VERSION`
   - `core-rs/Cargo.toml`
   - `ide-rs-tauri/Cargo.toml`
   - `ide-rs-tauri/tauri.conf.json`
4. 新しいバージョンタグ（例: `v1.2.3`）を自動作成
5. タグ作成により `release.yml` が起動し、リリースを自動公開

### 手動でのバージョン変更

通常は不要です。バージョンはコミットメッセージから自動決定されます。

**例外的に手動変更が必要な場合：**

- 初期バージョンの設定
- バージョン番号の強制リセット

```bash
# Dry runで確認
gh workflow run semantic-version.yml -f dry_run=true
```

---

## テスト方針 / Testing Policy

### 基本原則

**テストは極力自動化する。ユーザーが行うのは最終の総合テスト（UI 操作）のみ。**

### テスト自動化のためのアーキテクチャ原則

#### 1. ロジックと I/O の分離

```
❌ 悪い例（テスト困難）
fn capture_and_find(image_path: &str) -> Result<Point> {
    let screen = capture_screen();  // OS依存
    let template = load_image(image_path);  // ファイルI/O
    find_template(&screen, &template)
}

✅ 良い例（テスト容易）
fn find_template(screen: &Image, template: &Image) -> Result<Point> {
    // 純粋なロジックのみ
}

// I/Oは呼び出し側で
let screen = screen_capture.capture()?;
let template = image_loader.load(path)?;
let result = find_template(&screen, &template)?;
```

#### 2. トレイトによる抽象化（依存性注入）

```rust
// インターフェース定義
pub trait ScreenCapture {
    fn capture(&self) -> Result<Image>;
    fn capture_region(&self, region: &Region) -> Result<Image>;
}

// 本番実装
pub struct WindowsScreenCapture;
impl ScreenCapture for WindowsScreenCapture { ... }

// テスト用モック
pub struct MockScreenCapture {
    pub mock_image: Image,
}
impl ScreenCapture for MockScreenCapture {
    fn capture(&self) -> Result<Image> {
        Ok(self.mock_image.clone())
    }
}
```

#### 3. レイヤー分離

```
┌─────────────────────────────────────┐
│  ide-rs-tauri (UI/Presentation)     │ ← UIテスト（最小限）
├─────────────────────────────────────┤
│  Application Service Layer          │ ← 統合テスト
├─────────────────────────────────────┤
│  core-rs (Domain/Business Logic)    │ ← ユニットテスト（最重要）
├─────────────────────────────────────┤
│  Infrastructure (OS/File/Network)   │ ← モック化対象
└─────────────────────────────────────┘
```

#### 4. 外部依存のモック化

| 依存              | 本番       | テスト                     |
| ----------------- | ---------- | -------------------------- |
| 画面キャプチャ    | OS API     | 画像ファイル読み込み       |
| マウス/キーボード | OS API     | イベントログ記録           |
| ファイル I/O      | 実ファイル | 一時ディレクトリ or メモリ |
| Python 実行       | 実プロセス | 出力のシミュレーション     |

#### 5. テストヘルパー

```rust
// core-rs/src/test_utils.rs
#[cfg(test)]
pub mod test_utils {
    pub fn load_test_image(name: &str) -> Image {
        let path = format!("tests/fixtures/{}", name);
        load_image(&path).unwrap()
    }

    pub fn create_mock_screen(width: u32, height: u32) -> Image {
        Image::new_blank(width, height)
    }
}
```

### テストの種類と担当

| テスト種別               | 担当     | 自動化     | 説明                       |
| ------------------------ | -------- | ---------- | -------------------------- |
| ユニットテスト           | Claude   | 必須       | 関数・モジュール単位       |
| 統合テスト               | Claude   | 必須       | コンポーネント間連携       |
| E2E テスト               | Claude   | 可能な限り | 画面操作のシミュレーション |
| 総合テスト（受入テスト） | ユーザー | 手動       | 最終確認のみ               |

### 正常系・異常系の両方を実施

すべてのテストで以下を網羅する：

#### 正常系

- 期待通りの入力での動作確認
- 境界値テスト

#### 異常系

- 無効な入力
- null/undefined/空文字
- ファイル不在
- ネットワークエラー（該当する場合）
- タイムアウト
- 権限エラー

### Rust テストコマンド

```bash
# 全テスト実行
cargo test

# 特定モジュールのテスト
cargo test module_name

# 詳細出力
cargo test -- --nocapture

# カバレッジ（tarpaulin使用）
cargo tarpaulin --out Html
```

### テストファイル構成

```
core-rs/
├── src/
│   ├── lib.rs
│   └── module/
│       ├── mod.rs
│       └── tests.rs      # 各モジュールのテスト
└── tests/
    └── integration/      # 統合テスト
        ├── screen_test.rs
        └── input_test.rs

ide-rs-tauri/
└── tests/
    └── e2e/              # E2Eテスト
```

### CI/CD での自動テスト

GitHub Actions で以下を自動実行：

- `cargo test` - 全ユニットテスト
- `cargo clippy` - 静的解析
- `cargo fmt --check` - フォーマットチェック

### ユーザーへの報告

フェーズ完了時に Claude は以下を報告する：

```
=== テスト結果 ===
ユニットテスト: 42/42 passed
統合テスト: 8/8 passed
カバレッジ: 85%

[ユーザー確認が必要な項目]
- UI操作: ファイルを開いて保存できるか確認してください
- UI操作: 設定ダイアログが開くか確認してください
```

---

## 日本語対応開発メモ

（日本語対応の実装時にここに記録を追加）

---

## 参考リンク

- 元リポジトリ: https://github.com/RaiMan/SikuliX1
- SikuliX 公式ドキュメント: https://sikulix.github.io/
- ライセンス: MIT License
