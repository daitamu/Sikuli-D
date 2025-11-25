# Sikuli-D プロジェクト開発ルール

## プロジェクト概要

SikuliX 2.0.5をベースに日本語対応や独自機能を追加したバージョン。
画面上の画像認識によるGUI自動化ツール。

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

### APIのみビルド
```bash
mvn clean install -pl API
```

### IDEのみビルド
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

## コミットルール

- コミットメッセージは変更内容を簡潔に記述
- フッターに以下を付与：
  ```
  🤖 Generated with [Claude Code](https://claude.ai/claude-code)

  Co-Authored-By: Claude <noreply@anthropic.com>
  ```

---

## 日本語対応開発メモ

（日本語対応の実装時にここに記録を追加）

---

## 参考リンク

- 元リポジトリ: https://github.com/RaiMan/SikuliX1
- SikuliX公式ドキュメント: https://sikulix.github.io/
- ライセンス: MIT License
