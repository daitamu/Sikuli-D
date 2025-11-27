# Sikuli-D プロジェクト開発ルール

## プロジェクト概要

SikuliX 2.0.5 をベースに日本語対応や独自機能を追加したバージョン。
画面上の画像認識による GUI 自動化ツール。

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
├── API/                # SikuliX API（コアライブラリ）
│   ├── src/main/java/  # Javaソースコード
│   └── pom.xml         # API用Maven設定
├── IDE/                # SikuliX IDE（GUI環境）
│   ├── src/main/java/  # IDEソースコード
│   └── pom.xml         # IDE用Maven設定
├── Support/            # サポートファイル・ツール
│   ├── commands/       # コマンドスクリプト
│   └── experiments/    # 実験的機能
├── pages/              # ドキュメントページ
├── pom.xml             # 親Maven設定（マルチモジュール）
├── LICENSE             # MITライセンス
└── README.md           # プロジェクト説明
```

---

## 技術スタック

- **言語**: Java 17+ (LTS)
- **ビルドツール**: Maven
- **画像認識**: OpenCV
- **OCR**: Tesseract 5
- **スクリプト**: Python (Jython), JavaScript, Ruby

---

## ビルドコマンド

### 全体ビルド

```bash
mvn clean install
```

### API のみビルド

```bash
mvn clean install -pl API
```

### IDE のみビルド

```bash
mvn clean install -pl IDE
```

### テスト実行

```bash
mvn test
```

### テストスキップビルド

```bash
mvn clean install -DskipTests
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

- `mvn clean install` が成功する
- ビルドエラー・警告の解消

#### 3. テスト検証

- `mvn test` が成功する
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

### メッセージ形式 / Message Format

コミットメッセージは **日本語/英語の併記** で記述する：
Commit messages should be written in **bilingual (Japanese/English)** format:

```
English summary line
日本語の要約行

- English detail point
  日本語の詳細ポイント
- Another English point
  別の日本語ポイント

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

### 例 / Example

```
Update README with bilingual documentation
README を日英併記のドキュメントに更新

- Add Japanese/English descriptions
- Update requirements section

- 日本語/英語の説明を追加
- 要件セクションを更新

🤖 Generated with [Claude Code](https://claude.com/claude-code)

Co-Authored-By: Claude <noreply@anthropic.com>
```

---

## バージョン管理 / Version Management

### バージョン形式: x.y.z

| 区分      | 変更タイミング              | 担当     |
| --------- | --------------------------- | -------- |
| x (MAJOR) | ユーザーが指示した時のみ    | ユーザー |
| y (MINOR) | 機能追加時（Claude が判断） | Claude   |
| z (PATCH) | ビルド時                    | 自動     |

### ファイル構成

```
Sikuli-D/
├── VERSION                              # MAJOR.MINOR（共通マスター）
├── core-rs/
│   ├── Cargo.toml                       # version = "x.y.z"
│   └── version.txt                      # PATCH（自動）
└── ide-rs-tauri/
    ├── Cargo.toml                       # version = "x.y.z"
    ├── tauri.conf.json                  # version: "x.y.z"
    └── version.txt                      # PATCH（自動）
```

### MINOR バージョンの運用（Claude 向け）

**以下の場合に MINOR を+1 する：**

- 新機能を追加した時
- GitHub で機能追加コミットをした時
- core-rs / ide-rs-tauri それぞれ、コードに変更があった方の MINOR を上げる

**MINOR を上げる時は PATCH を 0 にリセット**

### PATCH 自動インクリメント

**Rust (cargo tauri build 時):**

- `scripts/increment_patch.sh` を使用
- version.txt に保存
- Cargo.toml と tauri.conf.json を自動更新

### ビルドスクリプト使用方法

```bash
# 通常ビルド（PATCH自動+1）
./scripts/build.sh

# リリースビルド
./scripts/build.sh --release
```

### 重要：MAJOR は勝手に上げない

MAJOR はユーザーからの明示的な指示があった場合のみ変更する。
Claude が自己判断で MAJOR を上げることは禁止。

### バージョン同期チェック

コミット前に以下のファイルのバージョンが一致していることを確認：

- `VERSION`
- `core-rs/Cargo.toml`
- `ide-rs-tauri/Cargo.toml`
- `ide-rs-tauri/tauri.conf.json`

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
