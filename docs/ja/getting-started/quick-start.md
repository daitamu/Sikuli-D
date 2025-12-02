# クイックスタート

このクイックチュートリアルで Sikuli-D の基本を学びましょう。

## 最初のスクリプト

メモ帳を開いてテキストを入力する簡単な自動化スクリプトを作成しましょう。

### ステップ 1: スクリーンショットをキャプチャ

スクリプトを書く前に、操作したい GUI 要素の画像をキャプチャする必要があります:

1. Sikuli-D IDE を開く
2. カメラアイコンをクリックするか `Ctrl+Shift+2` を押してスクリーンショットをキャプチャ
3. キャプチャしたいエリアを選択 (例: スタートボタン)
4. 画像を `start_button.png` として保存
5. 必要な他の GUI 要素についても繰り返す

### ステップ 2: スクリプトを書く

新しい Python ファイル `hello_sikuli.py` を作成:

```python
from sikulid import *

# Windows スタートボタンをクリック
click("start_button.png")

# メニューが表示されるまで 1 秒待つ
wait(1)

# "notepad" と入力してメモ帳を検索
type("notepad" + Key.ENTER)

# メモ帳ウィンドウが表示されるまで待つ
wait("notepad_window.png", 5)

# テキストを入力
type("こんにちは、Sikuli-D!")
```

### ステップ 3: スクリプトを実行

IDE またはコマンドラインからスクリプトを実行:

```bash
python hello_sikuli.py
```

Sikuli-D が自動的に以下を実行するのを見てください:
1. スタートボタンをクリック
2. "notepad" と入力して Enter を押す
3. メモ帳が開くまで待つ
4. テキストエディタに "こんにちは、Sikuli-D!" と入力

## 基本操作

### 画像を見つける

```python
# 画面上の画像を見つける
if exists("button.png"):
    print("ボタンが見つかりました!")
else:
    print("ボタンが見つかりませんでした")
```

### クリック

```python
# シンプルなクリック
click("button.png")

# ダブルクリック
doubleClick("file.png")

# 右クリック
rightClick("item.png")
```

### テキスト入力

```python
# テキストを入力
type("こんにちは、世界!")

# 特殊キーを使ってテキストを入力
type("username" + Key.TAB + "password" + Key.ENTER)

# クリップボードから貼り付け
paste("貼り付けるテキスト")
```

### 待機

```python
# 画像が表示されるまで待つ (タイムアウト: 3 秒)
wait("loading.png", 3)

# 画像が消えるまで待つ
waitVanish("loading.png", 10)
```

### Region (領域)

```python
# 画面全体を取得
screen = Screen()

# カスタム領域を定義
region = Region(100, 100, 400, 300)  # x, y, 幅, 高さ

# 領域内で検索
match = region.find("button.png")
region.click("button.png")
```

### テキスト読み取り (OCR)

```python
# 画面からすべてのテキストを読み取る
text = Screen().text()
print(text)

# 特定の領域からテキストを読み取る
region = Region(100, 100, 400, 300)
text = region.text()
print(text)
```

## パターンマッチング

画像のマッチング精度を制御:

```python
# 完全一致 (類似度: 0.99)
click(Pattern("button.png").exact())

# 緩い一致 (類似度: 0.7)
click(Pattern("button.png").similar(0.7))

# 画像中心からオフセットしてクリック
click(Pattern("icon.png").targetOffset(50, 20))
```

## エラー処理

```python
try:
    click("button.png")
except FindFailed:
    print("button.png が見つかりませんでした")
    # エラーを適切に処理
```

## ベストプラクティス

1. **一意の画像を使用**: 誤検出を避けるために特徴的な GUI 要素をキャプチャ
2. **適切なタイムアウトを設定**: アプリケーションが応答する時間を与える
3. **エラーを処理**: 堅牢なスクリプトのために try/except ブロックを使用
4. **領域をテスト**: 高速実行のために特定の画面エリアに検索を限定
5. **類似度を調整**: 変化する GUI 要素には低い類似度 (0.7-0.8) を使用

## 一般的なパターン

### 待機とクリックのパターン

```python
# 要素を待ってからクリック
wait("button.png", 10)
click("button.png")

# または 1 行で組み合わせる
click(wait("button.png", 10))
```

### 見つかるまでループ

```python
# 見つかるまで試行を続ける
while not exists("ready.png"):
    wait(1)
print("アプリケーションの準備ができました!")
```

### 複数画面の処理

```python
# 特定のモニターで作業
screen1 = Screen(0)
screen2 = Screen(1)

screen1.click("button.png")
screen2.type("2 番目のモニターのテキスト")
```

## 次のステップ

- 完全なドキュメントについては [API リファレンス](/ja/api/) を参照
- より多くの例については [チュートリアル](/ja/tutorials/) を確認
- [高度なパターンとテクニック](/ja/tutorials/image-recognition) について学ぶ

## ヘルプを取得

- 一般的な問題については [トラブルシューティング](/ja/troubleshooting/) を確認
- よくある質問については [FAQ](/ja/troubleshooting/faq) を参照
- [GitHub](https://github.com/daitamu/Sikuli-D) でディスカッションに参加
