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

# Aliases for compatibility / 互換性のためのエイリアス
doubleClick = double_click
rightClick = right_click
findAll = find_all

# Global Screen instance / グローバルScreenインスタンス
SCREEN = None  # Initialized after Screen class is available

# FindFailed exception for SikuliX compatibility
# SikuliX互換性のためのFindFailed例外
class FindFailed(Exception):
    """Exception raised when target image is not found / ターゲット画像が見つからない時に発生する例外"""
    pass

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


# Import sleep from time module for SikuliX compatibility
# SikuliX互換性のためtimeモジュールからsleepをインポート
from time import sleep

# Initialize global SCREEN instance
# グローバルSCREENインスタンスを初期化
SCREEN = Screen(0)


def getScreen(index=0):
    """Get a Screen object for the specified monitor / 指定されたモニターの Screen オブジェクトを取得

    Args:
        index: Monitor index (0 = primary) / モニターインデックス (0 = プライマリ)

    Returns:
        Screen object for the specified monitor / 指定されたモニターの Screen オブジェクト

    Example:
        # Get primary screen / プライマリスクリーンを取得
        screen = getScreen()

        # Get second monitor / 2番目のモニターを取得
        screen2 = getScreen(1)
    """
    return Screen(index)


def getNumberScreens():
    """Get the number of connected screens/monitors / 接続されている画面/モニターの数を取得

    Returns:
        Number of screens / 画面数

    Example:
        num = getNumberScreens()
        for i in range(num):
            screen = getScreen(i)
    """
    return Screen.get_number_screens()


# Image path management / 画像パス管理
_bundle_path = None
_image_paths = []


def getBundlePath():
    """Get the current bundle path / 現在のバンドルパスを取得

    Returns:
        Current bundle path or None / 現在のバンドルパスまたはNone
    """
    return _bundle_path


def setBundlePath(path):
    """Set the bundle path for image searching / 画像検索用のバンドルパスを設定

    Args:
        path: Path to the image bundle / 画像バンドルへのパス
    """
    global _bundle_path
    _bundle_path = path


def getImagePath():
    """Get all image search paths / 全画像検索パスを取得

    Returns:
        List of image paths / 画像パスのリスト
    """
    paths = list(_image_paths)
    if _bundle_path:
        paths.insert(0, _bundle_path)
    return paths


def addImagePath(path):
    """Add a path to the image search paths / 画像検索パスにパスを追加

    Args:
        path: Path to add / 追加するパス
    """
    if path not in _image_paths:
        _image_paths.append(path)


def removeImagePath(path):
    """Remove a path from the image search paths / 画像検索パスからパスを削除

    Args:
        path: Path to remove / 削除するパス
    """
    if path in _image_paths:
        _image_paths.remove(path)


def resetImagePath():
    """Reset image search paths to empty / 画像検索パスを空にリセット"""
    global _image_paths, _bundle_path
    _image_paths = []
    _bundle_path = None


# Application management (basic implementations)
# アプリケーション管理（基本実装）
def run(command):
    """Run an external command / 外部コマンドを実行

    Args:
        command: Command to run / 実行するコマンド

    Returns:
        Process return code / プロセスの戻りコード
    """
    import subprocess
    return subprocess.call(command, shell=True)


def openApp(app_name):
    """Open an application / アプリケーションを開く

    Args:
        app_name: Application name or path / アプリケーション名またはパス

    Returns:
        True if successful / 成功した場合True
    """
    import subprocess
    import sys

    try:
        if sys.platform == "win32":
            subprocess.Popen(["start", "", app_name], shell=True)
        elif sys.platform == "darwin":
            subprocess.Popen(["open", app_name])
        else:
            subprocess.Popen([app_name])
        return True
    except Exception:
        return False


def switchApp(app_name):
    """Switch to an application / アプリケーションに切り替え

    Note: Full implementation requires native module.
    注意: 完全な実装にはネイティブモジュールが必要です。

    Args:
        app_name: Application name / アプリケーション名

    Returns:
        True if successful / 成功した場合True
    """
    # Basic implementation - just try to open the app
    return openApp(app_name)


def closeApp(app_name):
    """Close an application / アプリケーションを閉じる

    Note: Full implementation requires native module.
    注意: 完全な実装にはネイティブモジュールが必要です。

    Args:
        app_name: Application name / アプリケーション名

    Returns:
        True if successful / 成功した場合True
    """
    import subprocess
    import sys

    try:
        if sys.platform == "win32":
            subprocess.call(["taskkill", "/IM", app_name, "/F"], shell=True)
        else:
            subprocess.call(["pkill", "-f", app_name])
        return True
    except Exception:
        return False


def focusApp(app_name):
    """Focus an application window / アプリケーションウィンドウにフォーカス

    Alias for switchApp / switchAppのエイリアス
    """
    return switchApp(app_name)


# Keyboard modifier functions / キーボード修飾子関数
def keyDown(key):
    """Press and hold a key / キーを押し続ける

    Note: Full implementation requires native module.
    注意: 完全な実装にはネイティブモジュールが必要です。

    Args:
        key: Key to press / 押すキー
    """
    if not _NATIVE_AVAILABLE:
        raise RuntimeError(
            "keyDown requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "keyDownにはネイティブsikulidモジュールが必要です。"
        )


def keyUp(key):
    """Release a key / キーを離す

    Note: Full implementation requires native module.
    注意: 完全な実装にはネイティブモジュールが必要です。

    Args:
        key: Key to release / 離すキー
    """
    if not _NATIVE_AVAILABLE:
        raise RuntimeError(
            "keyUp requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "keyUpにはネイティブsikulidモジュールが必要です。"
        )


# Screen capture function / 画面キャプチャ関数
def capture(*args):
    """Capture screen or region / 画面または領域をキャプチャ

    Note: Full implementation requires native module.
    注意: 完全な実装にはネイティブモジュールが必要です。

    Args:
        *args: Optional region parameters / オプションの領域パラメータ

    Returns:
        Path to captured image / キャプチャした画像へのパス
    """
    if not _NATIVE_AVAILABLE:
        raise RuntimeError(
            "capture requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "captureにはネイティブsikulidモジュールが必要です。"
        )


def captureScreen(*args):
    """Alias for capture() / capture()のエイリアス"""
    return capture(*args)


# Debug/logging functions / デバッグ/ロギング関数
def Debug(level):
    """Set debug level (no-op in pure Python mode) / デバッグレベルを設定（純Pythonモードではno-op）

    Args:
        level: Debug level / デバッグレベル
    """
    pass


# Version info
__version__ = "0.1.0"
__all__ = [
    # Core classes / コアクラス
    "sleep",
    "Region",
    "Match",
    "Screen",
    "Location",
    "Pattern",
    "Observer",
    "Key",
    "Settings",
    "FindFailed",
    "SCREEN",
    "getScreen",
    "getNumberScreens",
    # Find functions / 検索関数
    "find",
    "find_all",
    "findAll",
    "wait",
    "waitVanish",
    "exists",
    # Click functions / クリック関数
    "click",
    "double_click",
    "doubleClick",
    "right_click",
    "rightClick",
    # Mouse functions / マウス関数
    "mouseMove",
    "hover",
    # Keyboard functions / キーボード関数
    "type_text",
    "paste",
    "hotkey",
    "keyDown",
    "keyUp",
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
    # UI functions / UI関数
    "highlight",
    "popup",
    "input",
    # Image path management / 画像パス管理
    "getBundlePath",
    "setBundlePath",
    "getImagePath",
    "addImagePath",
    "removeImagePath",
    "resetImagePath",
    # Application management / アプリケーション管理
    "run",
    "openApp",
    "switchApp",
    "closeApp",
    "focusApp",
    # Screen capture / 画面キャプチャ
    "capture",
    "captureScreen",
    # Debug / デバッグ
    "Debug",
]
