# インストール

このガイドでは、システムに Sikuli-D をインストールする方法について説明します。

## 前提条件

### ユーザー向け

- **オペレーティングシステム**: Windows 10+、macOS 10.15+、または Linux (Ubuntu 20.04+)
- **Python**: Python 3.8 以降 (Python 2.7 もサポート)
- **Tesseract OCR**: テキスト認識機能に必要

### 開発者向け

- **Rust**: 1.70 以降
- **Node.js**: 18 以降 (IDE 開発用)
- **Cargo**: Rust インストールに含まれる

## Sikuli-D のインストール

### オプション 1: ビルド済みバイナリをダウンロード (推奨)

1. [リリースページ](https://github.com/daitamu/Sikuli-D/releases) にアクセス
2. プラットフォームに適したパッケージをダウンロード:
   - Windows: `sikuli-d-ide-windows-x64.exe`
   - macOS: `sikuli-d-ide-macos-universal.dmg`
   - Linux: `sikuli-d-ide-linux-x64.AppImage`
3. アプリケーションをインストールまたは実行

### オプション 2: Python ランタイムのみをインストール

IDE なしで Sikuli-D スクリプトを実行する場合:

```bash
pip install sikulid
```

### オプション 3: ソースからビルド

ソースからビルドする開発者向け:

```bash
# リポジトリをクローン
git clone https://github.com/daitamu/Sikuli-D.git
cd Sikuli-D

# コアライブラリをビルド
cd core-rs
cargo build --release

# IDE をビルド
cd ../ide-rs-tauri
cargo tauri build

# Python ランタイムをビルド
cd ../runtime-rs
pip install maturin
maturin build --release
```

## Tesseract OCR のインストール

OCR (テキスト認識) 機能には Tesseract が必要です。

### Windows

[GitHub Releases](https://github.com/UB-Mannheim/tesseract/wiki) からダウンロードしてインストール:

```powershell
# または Chocolatey を使用
choco install tesseract
```

### macOS

```bash
brew install tesseract
```

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install tesseract-ocr
```

## インストールの確認

### IDE インストールの確認

Sikuli-D IDE アプリケーションを起動します。エディタペインを持つメインウィンドウが表示されます。

### Python ランタイムの確認

```python
from sikulid import *

# 画面キャプチャをテスト
print(Screen())
# 出力: Screen(0)[0,0 1920x1080]

# OCR をテスト
text = Screen().text()
print("OCR が動作しています!")
```

### Tesseract の確認

```bash
tesseract --version
# バージョン 5.x 以降が出力されるはずです
```

## トラブルシューティング

### Tesseract が見つからない

"Tesseract not found" エラーが発生した場合:

1. Tesseract がインストールされていることを確認: `tesseract --version`
2. PATH 環境変数に Tesseract を追加
3. Windows では、デフォルトのインストールパスは: `C:\Program Files\Tesseract-OCR`

### Python インポートエラー

`from sikulid import *` が失敗する場合:

1. Python バージョンを確認: `python --version`
2. パッケージを再インストール: `pip install --force-reinstall sikulid`
3. pip インストールディレクトリを確認: `pip show sikulid`

### IDE が起動しない

1. システム要件が満たされていることを確認
2. ターミナルから実行してエラーメッセージを確認
3. 競合するアプリケーション (スクリーンリーダー、アクセシビリティツール) を確認

## 次のステップ

- [クイックスタートガイド](./quick-start.md) - 最初の Sikuli-D スクリプト
- [API リファレンス](/ja/api/) - 完全な API ドキュメント
- [チュートリアル](/ja/tutorials/) - ステップバイステップの例
