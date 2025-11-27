# SikuliX Runtime Usage Guide
# SikuliX ランタイム使用ガイド

## Table of Contents / 目次

1. [Installation / インストール](#installation)
2. [Basic Commands / 基本コマンド](#basic-commands)
3. [REPL Mode / REPLモード](#repl-mode)
4. [Script Execution / スクリプト実行](#script-execution)
5. [Image Finding / 画像検索](#image-finding)
6. [Screen Capture / 画面キャプチャ](#screen-capture)
7. [Examples / 例](#examples)

---

## Installation / インストール

### Build from Source / ソースからビルド

```bash
cd runtime-rs
cargo build --release

# Binary will be at: target/release/sikulix
# バイナリの場所：target/release/sikulix
```

### Install / インストール

```bash
# Copy to PATH
sudo cp target/release/sikulix /usr/local/bin/

# Or on Windows
copy target\release\sikulix.exe C:\Windows\System32\
```

---

## Basic Commands / 基本コマンド

### Help / ヘルプ

```bash
# Show help
sikulix --help

# Show version
sikulix --version

# Show system info
sikulix info
```

### Log Levels / ログレベル

```bash
# Verbose mode
sikulix --verbose run script.py

# Set log level
sikulix --log-level debug run script.py
```

---

## REPL Mode / REPLモード

### Start REPL / REPL開始

```bash
# Basic REPL
sikulix repl

# With custom Python
sikulix repl --python python3.11

# With startup script
sikulix repl --startup examples/repl_startup.py

# Disable history
sikulix repl --no-history
```

### REPL Commands / REPLコマンド

Inside REPL:
REPL内で：

```python
# Import SikuliX API
from sikulix_api import *

# Find and click
m = find("button.png")
click(m)

# Type text
type("Hello World")

# Wait for element
m = wait("dialog.png", 5)

# Define function
def automate():
    m = find("start.png")
    click(m)
    type("Username")

# Show variables
:vars

# Show history
:history

# Get help
:help

# Exit
:exit
```

See [README_REPL.md](README_REPL.md) for more details.

---

## Script Execution / スクリプト実行

### Run Python Script / Pythonスクリプトを実行

```bash
# Basic execution
sikulix run script.py

# With arguments
sikulix run script.py arg1 arg2

# With timeout
sikulix run script.py --timeout 60

# Custom working directory
sikulix run script.py --workdir /path/to/images

# Verbose output
sikulix --verbose run script.py
```

### Run .sikuli Bundle / .sikuliバンドルを実行

```bash
# Run bundle
sikulix run mytest.sikuli

# Bundle structure:
# mytest.sikuli/
# ├── mytest.py
# ├── button.png
# └── icon.png
```

### Script Example / スクリプト例

Create `test.py`:

```python
#!/usr/bin/env python3
from sikulix_api import *

# Find and click button
m = wait("button.png", 5)
click(m)

# Type text
type("Hello SikuliX")

# Press Enter
hotkey(Key.ENTER)

print("Automation complete!")
```

Run:

```bash
sikulix run test.py
```

---

## Image Finding / 画像検索

### Find Image on Screen / 画面上で画像を検索

```bash
# Find single match
sikulix find button.png

# Find with high similarity
sikulix find button.png --similarity 0.95

# Find all matches
sikulix find icon.png --all

# Wait for image
sikulix find dialog.png --timeout 10
```

### Output Examples / 出力例

**Single match:**
```
Found at: (523, 304)
Score: 0.89 (89%)
Region: x=473, y=279, w=100, h=50
```

**Multiple matches:**
```
Found 3 matches:
1. (523, 304) - Score: 0.92
2. (700, 400) - Score: 0.88
3. (150, 200) - Score: 0.85
```

---

## Screen Capture / 画面キャプチャ

### Capture Screen / 画面をキャプチャ

```bash
# Capture full screen
sikulix capture

# Save to specific file
sikulix capture --output screenshot.png

# Capture region (x,y,width,height)
sikulix capture --region "100,100,400,300"

# Capture with delay
sikulix capture --delay 3

# Capture secondary monitor
sikulix capture --screen 1
```

---

## Examples / 例

### Example 1: Automated Login / 自動ログイン

```python
#!/usr/bin/env python3
"""
Automated login example
自動ログインの例
"""
from sikulix_api import *

# Wait for login window
login_window = wait("login_window.png", 10)

# Enter username
username_field = find("username_field.png")
click(username_field)
type("myusername")

# Tab to password
hotkey(Key.TAB)

# Enter password
paste("mypassword")  # Using paste for security

# Click login button
login_button = find("login_button.png")
click(login_button)

# Wait for success
if exists("success_message.png", 5):
    print("Login successful!")
else:
    print("Login failed!")
```

### Example 2: Form Filling / フォーム入力

```python
#!/usr/bin/env python3
"""
Form filling example
フォーム入力の例
"""
from sikulix_api import *

# Form data
form_data = {
    "name": "John Doe",
    "email": "john@example.com",
    "phone": "123-456-7890",
}

# Fill each field
for field_name, value in form_data.items():
    # Find and click field
    field = find(f"{field_name}_field.png")
    click(field)

    # Clear existing content
    hotkey(Key.CTRL, "a")

    # Type new value
    type(value)

    # Tab to next field
    hotkey(Key.TAB)

# Submit form
submit = find("submit_button.png")
click(submit)

print("Form submitted!")
```

### Example 3: Monitoring / 監視

```python
#!/usr/bin/env python3
"""
Screen monitoring example
画面監視の例
"""
from sikulix_api import *
import time

print("Monitoring for error dialog...")

while True:
    # Check for error dialog every 5 seconds
    error = exists("error_dialog.png", 0)

    if error:
        print("Error detected!")

        # Take screenshot
        screen = Screen()
        img = screen.capture()
        img.save("error_screenshot.png")

        # Click OK to dismiss
        ok_button = find("ok_button.png")
        click(ok_button)

        print("Error handled")
        break

    time.sleep(5)
```

### Example 4: Batch Processing / バッチ処理

```python
#!/usr/bin/env python3
"""
Batch processing example
バッチ処理の例
"""
from sikulix_api import *
import sys

# Get file list from arguments
files = sys.argv[1:]

if not files:
    print("Usage: sikulix run batch.py file1 file2 file3...")
    sys.exit(1)

print(f"Processing {len(files)} files...")

for i, filename in enumerate(files, 1):
    print(f"[{i}/{len(files)}] Processing: {filename}")

    # Open file menu
    hotkey(Key.CTRL, "o")
    wait("open_dialog.png", 5)

    # Type filename
    type(filename)
    hotkey(Key.ENTER)

    # Wait for file to load
    wait("file_loaded.png", 10)

    # Process (example: save as PDF)
    hotkey(Key.CTRL, "p")
    wait("print_dialog.png", 5)

    pdf_option = find("save_as_pdf.png")
    click(pdf_option)

    save_button = find("save_button.png")
    click(save_button)

    # Wait for save to complete
    wait("save_complete.png", 10)

    print(f"  → Saved as PDF")

print("Batch processing complete!")
```

### Example 5: REPL Workflow / REPLワークフロー

```bash
# Start REPL with helper functions
sikulix repl --startup examples/repl_startup.py

# In REPL:
sikulix> # Use helper functions
sikulix> sc("button.png")  # safe_click
✓ Clicked: button.png

sikulix> wt("username.png", "myuser")  # wait_and_type
✓ Typed into username.png: myuser

sikulix> # Define workflow
sikulix> def login(user, password):
...         sc("username_field.png")
...         type(user)
...         hotkey(Key.TAB)
...         type(password)
...         sc("login_button.png")
...
sikulix> # Run workflow
sikulix> login("admin", "secret")
✓ Clicked: username_field.png
✓ Clicked: login_button.png

sikulix> # Capture result
sikulix> capture_to_desktop("result.png")
✓ Saved to: /home/user/Desktop/result.png

sikulix> :exit
```

---

## Tips / ヒント

### 1. Image Similarity / 画像類似度

- Default: 0.7 (70%)
- デフォルト：0.7（70%）
- Increase for strict matching: 0.9-0.95
- 厳密なマッチング：0.9-0.95
- Decrease for fuzzy matching: 0.5-0.6
- 曖昧なマッチング：0.5-0.6

### 2. Timeout Values / タイムアウト値

- Quick checks: 0-1s
- クイックチェック：0-1秒
- Normal wait: 3-5s
- 通常の待機：3-5秒
- Slow operations: 10-30s
- 低速操作：10-30秒

### 3. Error Handling / エラー処理

```python
# Use exists() instead of find() for optional elements
if exists("optional.png", 2):
    click(Last Match())
else:
    print("Optional element not found")

# Use try-except for critical operations
try:
    m = wait("required.png", 10)
    click(m)
except FindFailed:
    print("Required element not found!")
    sys.exit(1)
```

### 4. Performance / パフォーマンス

- Reduce search regions when possible
- 可能な場合は検索領域を縮小
- Use higher similarity for faster matching
- より高い類似度でマッチングを高速化
- Cache commonly used images
- よく使用される画像をキャッシュ

---

## Troubleshooting / トラブルシューティング

### Python Not Found / Python が見つからない

```bash
# Check Python installation
python3 --version

# Specify Python path
sikulix run script.py --python /usr/bin/python3
```

### Image Not Found / 画像が見つからない

```bash
# Capture current screen for comparison
sikulix capture --output debug.png

# Test with lower similarity
sikulix find button.png --similarity 0.5

# Check if image path is correct
ls -la button.png
```

### Script Timeout / スクリプトタイムアウト

```bash
# Increase timeout
sikulix run script.py --timeout 300

# Or in script:
wait("element.png", 60)  # Wait up to 60 seconds
```

---

## More Information / 詳細情報

- REPL Mode: [README_REPL.md](README_REPL.md)
- Design Spec: [../../.local/doc/spec/RUNTIME-RS-DESIGN.md](../../.local/doc/spec/RUNTIME-RS-DESIGN.md)
- Core Library: [../core-rs/README.md](../core-rs/README.md)

---

**Last Updated / 最終更新**: 2025-11-27
