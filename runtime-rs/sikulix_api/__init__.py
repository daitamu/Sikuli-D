"""
SikuliX Python API
SikuliX Python API

This module provides a Python interface compatible with SikuliX Java version.
このモジュールは SikuliX Java版と互換性のある Python インターフェースを提供します。

Usage / 使用方法:
    from sikulix import *

    # Find and click an image
    click("button.png")

    # Wait for image to appear
    wait("dialog.png", 10)

    # Type text
    type("Hello World")
"""

# Try to import native module (PyO3)
try:
    from sikulix_py import (
        Region,
        Match,
        Screen,
        find,
        find_all,
        wait,
        click,
        double_click,
        right_click,
        type as type_text,
    )
    _NATIVE_AVAILABLE = True
except ImportError:
    _NATIVE_AVAILABLE = False
    # Fallback to pure Python implementation
    from .region import Region
    from .match import Match
    from .screen import Screen
    from .finder import find, find_all, wait
    from .input import click, double_click, right_click, type_text

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


# Settings
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

    Args:
        target: Region, Match, or image path / Region、Match、または画像パス
        seconds: Duration in seconds / 継続時間（秒）
    """
    if seconds is None:
        seconds = Settings.DefaultHighlightTime

    # TODO: Implement highlight
    pass


def popup(message, title="SikuliX"):
    """Show a popup dialog / ポップアップダイアログを表示

    Args:
        message: Message to display / 表示するメッセージ
        title: Dialog title / ダイアログタイトル
    """
    print(f"[{title}] {message}")


def input(message="", default="", title="SikuliX"):
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
    "type_text",
    "highlight",
    "popup",
    "input",
]
