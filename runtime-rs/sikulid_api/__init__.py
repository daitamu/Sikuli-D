"""
Sikuli-D Python API
Sikuli-D Python API

This module provides a Python interface compatible with SikuliX Java version.
このモジュールは SikuliX Java版と互換性のある Python インターフェースを提供します。

Usage / 使用方法:
    from sikulid import *

    # Find and click an image
    click("button.png")

    # Wait for image to appear
    wait("dialog.png", 10)

    # Type text
    type("Hello World")
"""

# Try to import native module (PyO3)
try:
    from sikulid import (
        Region,
        Match,
        Screen,
        Location,
        Pattern,
        Settings as NativeSettings,
        Observer,
        find,
        find_all,
        wait,
        exists as native_exists,
        click,
        double_click,
        right_click,
        type_text,
        mouseMove,
        hover,
        paste,
        hotkey,
        # Scroll functions / スクロール関数
        scroll,
        scroll_up,
        scroll_down,
        scroll_horizontal,
        scroll_left,
        scroll_right,
        # Mouse button functions / マウスボタン関数
        mouseDown,
        mouseUp,
        # Drag functions / ドラッグ関数
        drag,
        dragTo,
        dragDrop,
    )
    _NATIVE_AVAILABLE = True
    # Use native Settings singleton / ネイティブのSettings シングルトンを使用
    Settings = NativeSettings()
except ImportError:
    _NATIVE_AVAILABLE = False
    # Fallback to pure Python implementation
    from .region import Region
    from .match import Match
    from .screen import Screen
    from .location import Location
    from .pattern import Pattern
    from .finder import find, find_all, wait
    from .input import click, double_click, right_click, type_text, mouseMove, hover, paste, hotkey
    # Placeholder for Observer when native unavailable
    Observer = None

    # Error message for native-only functions
    # ネイティブ専用関数のエラーメッセージ
    _NATIVE_REQUIRED_MSG = (
        "This function requires native sikulid module. "
        "Install with: pip install sikulid\n"
        "この関数にはネイティブsikulidモジュールが必要です。"
    )

    # Placeholder functions for scroll/drag/mouseDown when native unavailable
    # ネイティブが利用不可の場合のプレースホルダー関数
    def scroll(clicks):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def scroll_up(clicks=3):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def scroll_down(clicks=3):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def scroll_horizontal(clicks):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def scroll_left(clicks=3):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def scroll_right(clicks=3):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def mouseDown():
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def mouseUp():
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def drag(from_x, from_y, to_x, to_y):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def dragTo(x, y):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    def dragDrop(start_x, start_y, end_x, end_y):
        raise RuntimeError(_NATIVE_REQUIRED_MSG)

    Settings = None  # Will be defined below

# Aliases for compatibility
doubleClick = double_click
rightClick = right_click

# Key constants
class Key:
    """Keyboard key constants / キーボードキー定数"""
    ENTER = "\n"
    TAB = "\t"
    ESC = "\x1b"
    BACKSPACE = "\b"
    DELETE = "\x7f"

    # Arrow keys (special handling needed)
    UP = "{UP}"
    DOWN = "{DOWN}"
    LEFT = "{LEFT}"
    RIGHT = "{RIGHT}"

    # Function keys
    F1 = "{F1}"
    F2 = "{F2}"
    F3 = "{F3}"
    F4 = "{F4}"
    F5 = "{F5}"
    F6 = "{F6}"
    F7 = "{F7}"
    F8 = "{F8}"
    F9 = "{F9}"
    F10 = "{F10}"
    F11 = "{F11}"
    F12 = "{F12}"

    # Modifiers
    CTRL = "{CTRL}"
    ALT = "{ALT}"
    SHIFT = "{SHIFT}"
    META = "{META}"
    WIN = "{WIN}"
    CMD = "{CMD}"

    # Special
    HOME = "{HOME}"
    END = "{END}"
    PAGE_UP = "{PGUP}"
    PAGE_DOWN = "{PGDN}"
    INSERT = "{INSERT}"
    SPACE = " "


# Settings - fallback to pure Python class if native module not available
# ネイティブモジュールが利用不可の場合はPythonクラスにフォールバック
if Settings is None:
    class Settings:
        """Global settings / グローバル設定"""
        MinSimilarity = 0.7
        AutoWaitTimeout = 3.0
        MoveMouseDelay = 0.3
        ClickDelay = 0.0
        TypeDelay = 0.0
        ObserveScanRate = 3.0
        WaitScanRate = 3.0

        # Highlight settings
        Highlight = False
        DefaultHighlightTime = 2.0


# Convenience functions
def exists(target, timeout=0):
    """Check if target exists on screen / ターゲットが画面上に存在するか確認

    Args:
        target: Image path or pattern / 画像パスまたはパターン
        timeout: Wait time in seconds / 待機時間（秒）

    Returns:
        Match object if found, None otherwise / 見つかればMatchオブジェクト、なければNone
    """
    try:
        return wait(target, timeout)
    except Exception:
        return None


def waitVanish(target, timeout=None):
    """Wait for target to disappear / ターゲットが消えるまで待機

    Args:
        target: Image path or pattern / 画像パスまたはパターン
        timeout: Wait time in seconds / 待機時間（秒）

    Returns:
        True if vanished, False if timeout / 消えたらTrue、タイムアウトならFalse
    """
    import time

    if timeout is None:
        timeout = Settings.AutoWaitTimeout

    start = time.time()
    while time.time() - start < timeout:
        if exists(target, 0) is None:
            return True
        time.sleep(1.0 / Settings.WaitScanRate)
    return False


def highlight(target, seconds=None):
    """Highlight a region on screen / 画面上の領域をハイライト

    Note: Full highlight functionality requires the native sikulid module.
    注意: 完全なハイライト機能にはネイティブsikulidモジュールが必要です。

    Args:
        target: Region, Match, or image path / Region、Match、または画像パス
        seconds: Duration in seconds / 継続時間（秒）
    """
    if seconds is None:
        seconds = Settings.DefaultHighlightTime if Settings else 2.0

    # In pure Python mode, just log the highlight request
    # 純Pythonモードでは、ハイライトリクエストをログに記録するのみ
    if hasattr(target, 'region'):
        r = target.region
        print(f"[highlight] Region({r.x}, {r.y}, {r.width}, {r.height}) for {seconds}s")
    elif hasattr(target, 'x') and hasattr(target, 'y'):
        print(f"[highlight] Region({target.x}, {target.y}, {target.w}, {target.h}) for {seconds}s")
    else:
        print(f"[highlight] {target} for {seconds}s")


def popup(message, title="Sikuli-D"):
    """Show a popup dialog / ポップアップダイアログを表示

    Args:
        message: Message to display / 表示するメッセージ
        title: Dialog title / ダイアログタイトル
    """
    print(f"[{title}] {message}")


def input(message="", default="", title="Sikuli-D"):
    """Show an input dialog / 入力ダイアログを表示

    Args:
        message: Prompt message / プロンプトメッセージ
        default: Default value / デフォルト値
        title: Dialog title / ダイアログタイトル

    Returns:
        User input string / ユーザー入力文字列
    """
    return __builtins__["input"](f"[{title}] {message} [{default}]: ") or default


# Version info
__version__ = "0.1.0"
__all__ = [
    "Region",
    "Match",
    "Screen",
    "Location",
    "Pattern",
    "Observer",
    "Key",
    "Settings",
    "find",
    "find_all",
    "wait",
    "waitVanish",
    "exists",
    "click",
    "double_click",
    "doubleClick",
    "right_click",
    "rightClick",
    "mouseMove",
    "hover",
    "type_text",
    "paste",
    "hotkey",
    # Scroll functions / スクロール関数
    "scroll",
    "scroll_up",
    "scroll_down",
    "scroll_horizontal",
    "scroll_left",
    "scroll_right",
    # Mouse button functions / マウスボタン関数
    "mouseDown",
    "mouseUp",
    # Drag functions / ドラッグ関数
    "drag",
    "dragTo",
    "dragDrop",
    "highlight",
    "popup",
    "input",
]
