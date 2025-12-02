# API リファレンス

Sikuli-D API リファレンスへようこそ。このドキュメントでは、利用可能なすべてのクラス、メソッド、関数について説明します。

## コアクラス

### Screen
ディスプレイを表し、GUI 要素を検索および操作するためのメソッドを提供します。

[Screen API を見る →](./screen.md)

### Region
操作を実行できる画面上の矩形領域。

[Region API を見る →](./region.md)

### Pattern
マッチング設定とターゲットオフセットを持つ画像パターンを表します。

[Pattern API を見る →](./pattern.md)

### Match
位置とスコア情報を持つ成功した画像マッチを表します。

[Match API を見る →](./match.md)

## クイックリファレンス

### 要素の検索

```python
from sikulid import *

# 画像を見つける
match = find("button.png")

# 存在するか確認
if exists("icon.png"):
    print("見つかりました!")

# 表示されるまで待つ
wait("dialog.png", 10)

# 消えるまで待つ
waitVanish("loading.png", 30)
```

### マウスアクション

```python
# クリック
click("button.png")

# ダブルクリック
doubleClick("file.png")

# 右クリック
rightClick("menu_item.png")

# ドラッグアンドドロップ
drag("source.png")
dropAt("target.png")

# ホバー
hover("tooltip_trigger.png")
```

### キーボードアクション

```python
# テキストを入力
type("こんにちは、世界!")

# 特殊キー
type(Key.ENTER)
type(Key.TAB)
type(Key.ESC)

# 修飾キー
type("a", Key.CTRL)  # Ctrl+A
type("c", Key.CTRL)  # Ctrl+C
type("v", Key.CTRL)  # Ctrl+V

# 貼り付け
paste("クリップボードからのテキスト")
```

### OCR (テキスト認識)

```python
# 画面からテキストを読み取る
text = Screen().text()

# 領域からテキストを読み取る
region = Region(100, 100, 400, 300)
text = region.text()
```

### パターンマッチング

```python
# 類似度を調整
Pattern("button.png").similar(0.8)

# 完全一致
Pattern("icon.png").exact()

# ターゲットオフセット
Pattern("label.png").targetOffset(100, 0)
```

## 特殊キー

`Key` クラスで利用可能な一般的なキーボードキー:

| キー | 説明 |
|-----|------|
| `Key.ENTER` | Enter/Return キー |
| `Key.TAB` | Tab キー |
| `Key.ESC` | Escape キー |
| `Key.BACKSPACE` | Backspace キー |
| `Key.DELETE` | Delete キー |
| `Key.SPACE` | スペースバー |
| `Key.UP`, `Key.DOWN` | 矢印キー |
| `Key.LEFT`, `Key.RIGHT` | 矢印キー |
| `Key.PAGE_UP`, `Key.PAGE_DOWN` | ページナビゲーション |
| `Key.HOME`, `Key.END` | Home/End キー |
| `Key.F1` - `Key.F12` | ファンクションキー |

## 修飾キー

`KeyModifier` クラスで利用可能なキーボード修飾キー:

| 修飾キー | 説明 |
|---------|------|
| `Key.CTRL` | Control キー |
| `Key.ALT` | Alt キー |
| `Key.SHIFT` | Shift キー |
| `Key.META` | Windows/Command キー |

## マウスボタン

マウスボタン定数:

| ボタン | 説明 |
|-------|------|
| `Button.LEFT` | 左マウスボタン |
| `Button.MIDDLE` | 中央マウスボタン |
| `Button.RIGHT` | 右マウスボタン |

## 例外

### FindFailed

タイムアウト期間内に画像が見つからない場合に発生します。

```python
try:
    click("button.png")
except FindFailed as e:
    print(f"画像が見つかりませんでした: {e}")
```

### その他の例外

- `ImageFileNotFound`: 画像ファイルが存在しない
- `ScreenCaptureError`: 画面のキャプチャに失敗
- `OCRError`: OCR 処理に失敗

## 設定

### グローバル設定

```python
# 検索操作のデフォルトタイムアウトを設定
Settings.WaitForImageTimeout = 10.0  # 秒

# マッチの最小類似度を設定
Settings.MinSimilarity = 0.7

# 視覚効果を有効/無効化
Settings.ShowActions = True
Settings.ShowClick = True

# デフォルトの移動速度を設定
Settings.MoveMouseDelay = 0.5
```

## 互換性に関する注意

Sikuli-D は SikuliX API との完全な互換性を目指しています。ほとんどの SikuliX スクリプトは変更なしで動作します。

### SikuliX との違い

- Rust 実装による優れたパフォーマンス
- Tesseract 5 による改善された OCR 精度
- Python 3 のネイティブサポート (Python 2 に加えて)
- 強化されたエラーメッセージとデバッグ

## 次のステップ

- [Screen API ドキュメント](./screen.md)
- [Region API ドキュメント](./region.md)
- [Pattern API ドキュメント](./pattern.md)
- [Match API ドキュメント](./match.md)
- [チュートリアルを見る](/ja/tutorials/)
