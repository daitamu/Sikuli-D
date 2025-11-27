"""
Input functions - Pure Python fallback implementation
入力関数 - Pure Python フォールバック実装

Note: This fallback uses subprocess to call the sikulix CLI or system tools.
      For best performance, use the native module.
注意: このフォールバックは sikulix CLI またはシステムツールをサブプロセスで呼び出します。
      最高のパフォーマンスにはネイティブモジュールを使用してください。
"""

import subprocess
import sys
import time

# Default settings (avoid circular import)
_DEFAULT_MOVE_MOUSE_DELAY = 0.3
_DEFAULT_CLICK_DELAY = 0.0
_DEFAULT_TYPE_DELAY = 0.0


def _get_settings():
    """Get settings lazily to avoid circular import."""
    try:
        from . import Settings
        return Settings
    except ImportError:
        # Fallback to defaults
        class DefaultSettings:
            MoveMouseDelay = _DEFAULT_MOVE_MOUSE_DELAY
            ClickDelay = _DEFAULT_CLICK_DELAY
            TypeDelay = _DEFAULT_TYPE_DELAY
        return DefaultSettings


def mouseMove(target):
    """Move mouse to target / マウスをターゲットに移動

    Args:
        target: (x, y) tuple, Location, Match object, or image path /
                (x, y) タプル、Location、Match オブジェクト、または画像パス
    """
    x, y = _resolve_target(target)
    _do_mouse_move(x, y)


def hover(target):
    """Alias for mouseMove / mouseMoveのエイリアス"""
    mouseMove(target)


def click(target):
    """Click on target / ターゲットをクリック

    Args:
        target: (x, y) tuple, Match object, or image path /
                (x, y) タプル、Match オブジェクト、または画像パス
    """
    x, y = _resolve_target(target)
    _do_click(x, y, 1)


def double_click(target):
    """Double click on target / ターゲットをダブルクリック

    Args:
        target: (x, y) tuple, Match object, or image path /
                (x, y) タプル、Match オブジェクト、または画像パス
    """
    x, y = _resolve_target(target)
    _do_click(x, y, 2)


def right_click(target):
    """Right click on target / ターゲットを右クリック

    Args:
        target: (x, y) tuple, Match object, or image path /
                (x, y) タプル、Match オブジェクト、または画像パス
    """
    x, y = _resolve_target(target)
    _do_right_click(x, y)


def type_text(text, modifiers=None):
    """Type text / テキストを入力

    Args:
        text: Text to type / 入力するテキスト
        modifiers: Optional key modifiers / オプションのキー修飾子
    """
    Settings = _get_settings()
    if Settings.TypeDelay > 0:
        for char in text:
            _type_char(char)
            time.sleep(Settings.TypeDelay)
    else:
        _type_string(text)


def _resolve_target(target):
    """Resolve target to (x, y) coordinates / ターゲットを (x, y) 座標に解決

    Args:
        target: (x, y) tuple, Location, Match/Region object, or image path

    Returns:
        Tuple (x, y) / タプル (x, y)
    """
    if isinstance(target, tuple) and len(target) == 2:
        return target

    # Check for Location object (has x and y attributes but not w/h)
    if hasattr(target, "x") and hasattr(target, "y") and not hasattr(target, "w"):
        return (target.x, target.y)

    if hasattr(target, "center"):
        return target.center()

    if hasattr(target, "target"):
        return target.target()

    if isinstance(target, str):
        # It's an image path - find it first
        from .finder import find
        match = find(target)
        if match:
            return match.target()
        else:
            from .finder import FindFailed
            raise FindFailed(f"FindFailed: {target}")

    raise ValueError(f"Invalid target type: {type(target)}")


def _do_click(x, y, clicks=1):
    """Perform click at coordinates / 座標でクリックを実行

    Args:
        x: X coordinate / X座標
        y: Y coordinate / Y座標
        clicks: Number of clicks (1 or 2) / クリック回数 (1 または 2)
    """
    Settings = _get_settings()
    # Move mouse delay
    if Settings.MoveMouseDelay > 0:
        time.sleep(Settings.MoveMouseDelay)

    if sys.platform == "win32":
        _windows_click(x, y, clicks)
    elif sys.platform == "darwin":
        _macos_click(x, y, clicks)
    else:
        _linux_click(x, y, clicks)

    # Click delay
    if Settings.ClickDelay > 0:
        time.sleep(Settings.ClickDelay)


def _do_right_click(x, y):
    """Perform right click at coordinates / 座標で右クリックを実行"""
    Settings = _get_settings()
    if Settings.MoveMouseDelay > 0:
        time.sleep(Settings.MoveMouseDelay)

    if sys.platform == "win32":
        _windows_right_click(x, y)
    elif sys.platform == "darwin":
        _macos_right_click(x, y)
    else:
        _linux_right_click(x, y)

    if Settings.ClickDelay > 0:
        time.sleep(Settings.ClickDelay)


def _do_mouse_move(x, y):
    """Move mouse to coordinates / マウスを座標に移動"""
    if sys.platform == "win32":
        _windows_mouse_move(x, y)
    elif sys.platform == "darwin":
        _macos_mouse_move(x, y)
    else:
        _linux_mouse_move(x, y)


def _type_char(char):
    """Type a single character / 単一の文字を入力"""
    if sys.platform == "win32":
        _windows_type(char)
    elif sys.platform == "darwin":
        _macos_type(char)
    else:
        _linux_type(char)


def _type_string(text):
    """Type a string / 文字列を入力"""
    if sys.platform == "win32":
        _windows_type(text)
    elif sys.platform == "darwin":
        _macos_type(text)
    else:
        _linux_type(text)


# Windows implementations
def _windows_click(x, y, clicks):
    """Windows click using PowerShell"""
    script = f'''
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({x}, {y})
$signature = @"
[DllImport("user32.dll")]
public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo);
"@
$mouse = Add-Type -MemberDefinition $signature -Name "Win32Mouse" -Namespace "Win32" -PassThru
for ($i = 0; $i -lt {clicks}; $i++) {{
    $mouse::mouse_event(0x0002, 0, 0, 0, 0)  # LEFTDOWN
    $mouse::mouse_event(0x0004, 0, 0, 0, 0)  # LEFTUP
    Start-Sleep -Milliseconds 50
}}
'''
    subprocess.run(["powershell", "-Command", script], capture_output=True)


def _windows_right_click(x, y):
    """Windows right click using PowerShell"""
    script = f'''
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({x}, {y})
$signature = @"
[DllImport("user32.dll")]
public static extern void mouse_event(int dwFlags, int dx, int dy, int dwData, int dwExtraInfo);
"@
$mouse = Add-Type -MemberDefinition $signature -Name "Win32Mouse" -Namespace "Win32" -PassThru
$mouse::mouse_event(0x0008, 0, 0, 0, 0)  # RIGHTDOWN
$mouse::mouse_event(0x0010, 0, 0, 0, 0)  # RIGHTUP
'''
    subprocess.run(["powershell", "-Command", script], capture_output=True)


def _windows_type(text):
    """Windows type using PowerShell"""
    # Escape special characters
    escaped = text.replace("'", "''").replace("`", "``")
    script = f'''
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.SendKeys]::SendWait('{escaped}')
'''
    subprocess.run(["powershell", "-Command", script], capture_output=True)


def _windows_mouse_move(x, y):
    """Windows mouse move using PowerShell"""
    script = f'''
Add-Type -AssemblyName System.Windows.Forms
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point({x}, {y})
'''
    subprocess.run(["powershell", "-Command", script], capture_output=True)


# macOS implementations
def _macos_click(x, y, clicks):
    """macOS click using cliclick or AppleScript"""
    try:
        # Try cliclick first (if installed)
        click_cmd = "dc" if clicks == 2 else "c"
        subprocess.run(["cliclick", f"{click_cmd}:{x},{y}"], capture_output=True)
    except FileNotFoundError:
        # Fallback to AppleScript
        script = f'''
tell application "System Events"
    click at {{{x}, {y}}}
end tell
'''
        if clicks == 2:
            script = f'''
tell application "System Events"
    click at {{{x}, {y}}}
    click at {{{x}, {y}}}
end tell
'''
        subprocess.run(["osascript", "-e", script], capture_output=True)


def _macos_right_click(x, y):
    """macOS right click"""
    try:
        subprocess.run(["cliclick", f"rc:{x},{y}"], capture_output=True)
    except FileNotFoundError:
        # Fallback to AppleScript (limited support)
        script = f'''
tell application "System Events"
    -- Right click simulation
end tell
'''
        subprocess.run(["osascript", "-e", script], capture_output=True)


def _macos_type(text):
    """macOS type using AppleScript"""
    escaped = text.replace('"', '\\"')
    script = f'tell application "System Events" to keystroke "{escaped}"'
    subprocess.run(["osascript", "-e", script], capture_output=True)


def _macos_mouse_move(x, y):
    """macOS mouse move using cliclick or AppleScript"""
    try:
        subprocess.run(["cliclick", f"m:{x},{y}"], capture_output=True)
    except FileNotFoundError:
        # Fallback to AppleScript with Quartz (limited)
        script = f'''
do shell script "python3 -c 'from Quartz.CoreGraphics import CGEventCreateMouseEvent, CGEventPost, kCGEventMouseMoved, kCGMouseButtonLeft, kCGHIDEventTap; e = CGEventCreateMouseEvent(None, kCGEventMouseMoved, ({x}, {y}), kCGMouseButtonLeft); CGEventPost(kCGHIDEventTap, e)'"
'''
        subprocess.run(["osascript", "-e", script], capture_output=True)


# Linux implementations
def _linux_click(x, y, clicks):
    """Linux click using xdotool"""
    click_arg = "--repeat 2" if clicks == 2 else ""
    subprocess.run(f"xdotool mousemove {x} {y} click {click_arg} 1", shell=True, capture_output=True)


def _linux_right_click(x, y):
    """Linux right click using xdotool"""
    subprocess.run(f"xdotool mousemove {x} {y} click 3", shell=True, capture_output=True)


def _linux_type(text):
    """Linux type using xdotool"""
    subprocess.run(["xdotool", "type", "--", text], capture_output=True)


def _linux_mouse_move(x, y):
    """Linux mouse move using xdotool"""
    subprocess.run(f"xdotool mousemove {x} {y}", shell=True, capture_output=True)
