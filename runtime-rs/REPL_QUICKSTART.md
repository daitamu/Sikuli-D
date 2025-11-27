# SikuliX REPL Quick Start Guide
# SikuliX REPL クイックスタートガイド

## 30-Second Quick Start / 30秒クイックスタート

```bash
# Build
cd runtime-rs
cargo build --release

# Run REPL
./target/release/sikulix repl

# Try it
sikulix> from sikulix_api import *
sikulix> print("Hello SikuliX!")
sikulix> :exit
```

---

## What is REPL? / REPLとは？

REPL = **R**ead-**E**val-**P**rint **L**oop

An interactive shell where you can:
インタラクティブシェルで以下が可能：

- Type Python code line-by-line
  Python コードを1行ずつ入力
- See results immediately
  結果を即座に確認
- Test SikuliX scripts interactively
  SikuliX スクリプトをインタラクティブにテスト
- Experiment with image finding
  画像検索を実験

---

## Key Features / 主な機能

```
┌─────────────────────────────────────────────┐
│  🎯 Tab Completion                          │
│  Press TAB to autocomplete functions        │
│  TABでファンクションを自動補完              │
├─────────────────────────────────────────────┤
│  📝 Command History                         │
│  Up/Down arrows to navigate history         │
│  上下矢印で履歴をナビゲート                 │
├─────────────────────────────────────────────┤
│  🔀 Multiline Support                       │
│  Define functions and classes               │
│  関数やクラスを定義                         │
├─────────────────────────────────────────────┤
│  ⚡ Special Commands                        │
│  :help :exit :clear :history :vars          │
│  特殊コマンドで便利な操作                   │
└─────────────────────────────────────────────┘
```

---

## Basic Usage / 基本的な使用方法

### 1. Start REPL / REPLを開始

```bash
sikulix repl
```

### 2. Import API / APIをインポート

```python
sikulix> from sikulix_api import *
```

### 3. Try Commands / コマンドを試す

```python
# Find image
sikulix> m = find("button.png")

# Click
sikulix> click(m)

# Type text
sikulix> type("Hello")

# Press hotkey
sikulix> hotkey(Key.CTRL, "s")
```

### 4. Exit / 終了

```python
sikulix> :exit
```

Or press `Ctrl+D`
または `Ctrl+D` を押す

---

## Useful Commands / 便利なコマンド

| Command | Description | 説明 |
|---------|-------------|------|
| `TAB` | Auto-complete | 自動補完 |
| `↑` / `↓` | History | 履歴 |
| `Ctrl+C` | Interrupt | 中断 |
| `Ctrl+D` | Exit | 終了 |
| `:help` | Show help | ヘルプ |
| `:vars` | Show variables | 変数表示 |
| `:clear` | Clear screen | 画面クリア |

---

## Common Patterns / よくあるパターン

### Pattern 1: Quick Test / クイックテスト

```python
sikulix> m = exists("element.png", 2)
sikulix> if m:
...         print("Found!")
...         click(m)
...
Found!
```

### Pattern 2: Define Function / 関数を定義

```python
sikulix> def click_button():
...         m = wait("button.png", 5)
...         click(m)
...         return m
...
sikulix> result = click_button()
```

### Pattern 3: Loop / ループ

```python
sikulix> for i in range(3):
...         click(100, 100 + i * 50)
...         type(f"Item {i}")
...
```

---

## Tips / ヒント

### Tip 1: Use exists() for Optional Elements
### ヒント1: 任意の要素にはexists()を使用

```python
# ❌ May crash if not found
m = find("optional.png")

# ✓ Safe
m = exists("optional.png", 2)
if m:
    click(m)
```

### Tip 2: Use Helper Startup Script
### ヒント2: ヘルパー起動スクリプトを使用

```bash
sikulix repl --startup examples/repl_startup.py
```

Then use shortcuts:
その後ショートカットを使用：

```python
sikulix> qf("button.png")  # quick_find
sikulix> sc("icon.png")    # safe_click
sikulix> wt("field.png", "text")  # wait_and_type
```

### Tip 3: Check Variables
### ヒント3: 変数を確認

```python
sikulix> :vars
['__annotations__', '__builtins__', 'm', 'result', ...]
```

### Tip 4: Save Your Session
### ヒント4: セッションを保存

```python
sikulix> :history
  1: from sikulix_api import *
  2: m = find("button.png")
  3: click(m)
```

Copy important commands to a script file!
重要なコマンドをスクリプトファイルにコピー！

---

## Troubleshooting / トラブルシューティング

### Problem: Python Not Found
### 問題: Pythonが見つからない

```bash
# Solution: Specify Python path
sikulix repl --python /usr/bin/python3
```

### Problem: API Not Loading
### 問題: APIが読み込まれない

```bash
# Check sikulix_api location
sikulix info

# Ensure sikulix_api directory exists
ls runtime-rs/sikulix_api/
```

### Problem: Tab Completion Not Working
### 問題: Tab補完が機能しない

```bash
# Try typing full command
sikulix> find("button.png")

# Tab completion may not work in all terminals
```

### Problem: History Not Saving
### 問題: 履歴が保存されない

```bash
# Check history file
ls -la ~/.sikulix_history

# Fix permissions
chmod 644 ~/.sikulix_history
```

---

## Next Steps / 次のステップ

1. **Read Full Documentation**
   完全なドキュメントを読む
   - [README_REPL.md](README_REPL.md) - Detailed REPL guide
   - [USAGE.md](USAGE.md) - Runtime-rs usage guide

2. **Try Examples**
   例を試す
   - Use startup script: `sikulix repl --startup examples/repl_startup.py`
   - Follow examples in USAGE.md

3. **Write Scripts**
   スクリプトを書く
   - Prototype in REPL
   - Save working code to .py file
   - Run with: `sikulix run script.py`

4. **Learn More**
   さらに学ぶ
   - SikuliX API documentation
   - Python automation patterns
   - Image recognition techniques

---

## Help & Support / ヘルプとサポート

- **In REPL**: Type `:help`
- **Documentation**: See [README_REPL.md](README_REPL.md)
- **Examples**: Check `examples/` directory
- **Issues**: Check runtime-rs documentation

---

## Summary / まとめ

The SikuliX REPL provides a powerful interactive environment for:
SikuliX REPL は以下のための強力なインタラクティブ環境を提供します：

✓ Quick prototyping / クイックプロトタイピング
✓ Testing image finding / 画像検索のテスト
✓ Learning SikuliX API / SikuliX API の学習
✓ Debugging automation scripts / 自動化スクリプトのデバッグ

**Start exploring now!**
**今すぐ探索を始めましょう！**

```bash
sikulix repl
```

---

*For detailed documentation, see [README_REPL.md](README_REPL.md)*
*詳細なドキュメントについては、[README_REPL.md](README_REPL.md)を参照してください*
