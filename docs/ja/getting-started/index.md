# 始め方

Sikuli-D へようこそ!このガイドでは、画像認識を使用した GUI 自動化の始め方を説明します。

## Sikuli-D とは?

Sikuli-D は、画像認識を使用して GUI 要素を識別および制御する最新の GUI 自動化ツールです。SikuliX の Rust ベース再実装であり、SikuliX スクリプトとの完全な互換性を維持しながら、より優れたパフォーマンスと低リソース使用量を提供します。

## 主な機能

- **画像認識**: スクリーンショットを使用して GUI 要素を検索および操作
- **OCR テキスト読み取り**: Tesseract OCR を使用して画面からテキストを抽出
- **Python スクリプト**: Python 2 または Python 3 で自動化スクリプトを記述
- **クロスプラットフォーム**: Windows、macOS、Linux で動作
- **モダン IDE**: 画像キャプチャとプレビュー機能を備えたビジュアルスクリプトエディタ
- **SikuliX 互換**: 既存の SikuliX スクリプトを変更なしで実行

## コアコンセプト

### Screen と Region

- **Screen**: ディスプレイ全体または特定のモニターを表します
- **Region**: 操作を実行する画面上の矩形領域

### Pattern と Match

- **Pattern**: 画面上で見つけたい画像
- **Match**: パターンが見つかった場所

### アクション

- **Find**: 画面上の画像を検索
- **Click**: 画像をクリック
- **Type**: テキストを入力 (オプションで画像をクリック後)
- **Wait**: 画像が表示されるまで待機
- **Observe**: 画像の変化を監視

## 次のステップ

1. [Sikuli-D をインストール](./installation.md)
2. [クイックスタートガイド](./quick-start.md)
3. [API リファレンスを探索](/ja/api/)

## ヘルプが必要ですか?

- [トラブルシューティング](/ja/troubleshooting/) ガイドを確認
- [GitHub リポジトリ](https://github.com/daitamu/Sikuli-D) を訪問
- [チュートリアル例](/ja/tutorials/) を確認
