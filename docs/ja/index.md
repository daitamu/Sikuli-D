---
layout: home

hero:
  name: "Sikuli-D"
  text: "GUI 自動化ツール"
  tagline: "Rust のパフォーマンスと画像認識による SikuliX 互換の自動化ツール"
  actions:
    - theme: brand
      text: 始め方
      link: /ja/getting-started/
    - theme: alt
      text: API リファレンス
      link: /ja/api/
    - theme: alt
      text: GitHub で見る
      link: https://github.com/daitamu/Sikuli-D

features:
  - icon: 🚀
    title: Rust のパフォーマンス
    details: Rust で構築された高速実行とメモリ効率
  - icon: 🖼️
    title: 画像認識
    details: OpenCV を使用した強力な画像マッチングによる信頼性の高い GUI 自動化
  - icon: 🐍
    title: Python 互換
    details: SikuliX Python 2/3 スクリプトを自動変換サポートで実行
  - icon: 🔍
    title: OCR サポート
    details: Tesseract 5 による高度な自動化のためのテキスト認識
  - icon: 💻
    title: クロスプラットフォーム
    details: Windows、macOS、Linux でネイティブパフォーマンスで動作
  - icon: 🛠️
    title: モダン IDE
    details: Monaco エディタとインライン画像プレビューを備えた Tauri ベースの IDE
---

## クイック例

```python
from sikulid import *

# 画面上の画像を見つけてクリック
click("button.png")

# 画像が表示されるまで待ってテキストを入力
wait("input_field.png", 5)
type("こんにちは、Sikuli-D!")

# OCR を使って画面からテキストを読み取る
text = Screen().text()
print(text)
```

## アーキテクチャ

Sikuli-D は 3 つの主要コンポーネントで構成されています:

- **コアライブラリ** (`core-rs`): 画像認識と自動化のための共有 Rust ライブラリ
- **IDE** (`ide-rs-tauri`): スクリプト開発用の Tauri で構築されたデスクトップアプリケーション
- **ランタイム** (`runtime-rs`): PyO3 バインディングを持つ Python 実行環境

## なぜ Sikuli-D?

Sikuli-D は SikuliX のモダンな再実装であり、以下を提供します:

- **優れたパフォーマンス**: Java ベースの SikuliX よりも高速な実行
- **低メモリ使用量**: JVM オーバーヘッドなしの効率的なメモリ管理
- **モダンツール**: 最新技術で構築 (Tauri 2.x、PyO3)
- **アクティブな開発**: 定期的な更新と改善
- **完全な互換性**: 既存の SikuliX スクリプトを変更なしで実行

## ライセンス

Sikuli-D は [Apache License 2.0](https://github.com/daitamu/Sikuli-D/blob/master/LICENSE) でリリースされています。
