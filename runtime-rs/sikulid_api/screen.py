"""
Screen class - Pure Python fallback implementation
Screen クラス - Pure Python フォールバック実装
"""

from .region import Region


class Screen(Region):
    """Represents a screen/monitor / 画面/モニターを表す

    Screen is a Region that covers the entire screen.
    Screen は画面全体をカバーする Region です。
    """

    def __init__(self, index=0):
        """Create a new Screen / 新しい Screen を作成

        Args:
            index: Screen/monitor index (0 = primary) /
                   画面/モニターインデックス (0 = プライマリ)
        """
        self.index = index
        # TODO: Get actual screen dimensions
        # TODO: 実際の画面サイズを取得
        super().__init__(0, 0, 1920, 1080)
        self._init_dimensions()

    def _init_dimensions(self):
        """Initialize screen dimensions / 画面サイズを初期化"""
        try:
            # Try to get actual screen size
            import subprocess
            import sys

            if sys.platform == "win32":
                # Windows
                result = subprocess.run(
                    ["powershell", "-Command",
                     "Add-Type -AssemblyName System.Windows.Forms; "
                     "[System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width,"
                     "[System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Height"],
                    capture_output=True, text=True
                )
                if result.returncode == 0:
                    lines = result.stdout.strip().split("\n")
                    if len(lines) >= 2:
                        self.w = int(lines[0])
                        self.h = int(lines[1])
            elif sys.platform == "darwin":
                # macOS
                result = subprocess.run(
                    ["system_profiler", "SPDisplaysDataType"],
                    capture_output=True, text=True
                )
                # Parse output for resolution
                # TODO: Implement proper parsing
            else:
                # Linux - try xrandr
                result = subprocess.run(
                    ["xrandr", "--current"],
                    capture_output=True, text=True
                )
                # Parse output for resolution
                # TODO: Implement proper parsing

        except Exception:
            # Use default dimensions
            pass

    @staticmethod
    def get_number_screens():
        """Get the number of screens/monitors / 画面/モニターの数を取得

        Returns:
            Number of screens / 画面の数
        """
        try:
            import subprocess
            import sys

            if sys.platform == "win32":
                # Windows - use PowerShell to get monitor count
                result = subprocess.run(
                    ["powershell", "-Command",
                     "Add-Type -AssemblyName System.Windows.Forms; "
                     "[System.Windows.Forms.Screen]::AllScreens.Count"],
                    capture_output=True, text=True
                )
                if result.returncode == 0:
                    return int(result.stdout.strip())
            elif sys.platform == "darwin":
                # macOS - use system_profiler
                result = subprocess.run(
                    ["system_profiler", "SPDisplaysDataType"],
                    capture_output=True, text=True
                )
                # Count "Resolution:" lines
                return max(1, result.stdout.count("Resolution:"))
            else:
                # Linux - use xrandr
                result = subprocess.run(
                    ["xrandr", "--query"],
                    capture_output=True, text=True
                )
                # Count connected displays
                return max(1, result.stdout.count(" connected"))
        except Exception:
            pass
        return 1  # Default to 1 screen

    @staticmethod
    def getNumberScreens():
        """Alias for get_number_screens() / get_number_screens() のエイリアス"""
        return Screen.get_number_screens()

    @staticmethod
    def get_primary():
        """Get primary screen / プライマリ画面を取得

        Returns:
            Screen object for primary monitor / プライマリモニターの Screen オブジェクト
        """
        return Screen(0)

    def capture(self, region=None):
        """Capture screen or region / 画面または領域をキャプチャ

        Note: This method requires the native sikulid module.
        注意: このメソッドはネイティブsikulidモジュールが必要です。

        Args:
            region: Optional region to capture (defaults to full screen) /
                    キャプチャするオプションの領域（デフォルトは全画面）

        Returns:
            Image data / 画像データ

        Raises:
            RuntimeError: Native module required / ネイティブモジュールが必要
        """
        raise RuntimeError(
            "Screen capture requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "画面キャプチャにはネイティブsikulidモジュールが必要です。"
        )

    def userCapture(self, message="Select a region"):
        """Interactive screen capture / インタラクティブ画面キャプチャ

        Note: This method requires the native sikulid module.
        注意: このメソッドはネイティブsikulidモジュールが必要です。

        Args:
            message: Message to display / 表示するメッセージ

        Returns:
            Captured region / キャプチャした領域

        Raises:
            RuntimeError: Native module required / ネイティブモジュールが必要
        """
        raise RuntimeError(
            "Interactive capture requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "インタラクティブキャプチャにはネイティブsikulidモジュールが必要です。"
        )

    def getW(self):
        """Get screen width / 画面幅を取得

        Returns:
            Screen width in pixels / 画面幅（ピクセル）
        """
        return self.w

    def getH(self):
        """Get screen height / 画面高さを取得

        Returns:
            Screen height in pixels / 画面高さ（ピクセル）
        """
        return self.h

    def dimensions(self):
        """Get screen dimensions as (width, height) / 画面サイズを (幅, 高さ) で取得

        Returns:
            Tuple of (width, height) / (幅, 高さ) のタプル
        """
        return (self.w, self.h)

    def getBounds(self):
        """Get screen bounds as (x, y, w, h) / 画面境界を (x, y, w, h) で取得

        Returns:
            Tuple of (x, y, width, height) / (x, y, 幅, 高さ) のタプル
        """
        return (self.x, self.y, self.w, self.h)

    def get_region(self):
        """Get screen as a Region / 画面をRegionとして取得

        Returns:
            Region covering the entire screen / 画面全体をカバーするRegion
        """
        return Region(self.x, self.y, self.w, self.h)

    def selectRegion(self, message="Select a region"):
        """Interactive region selection / インタラクティブ領域選択

        Note: This method requires the native sikulid module.
        注意: このメソッドはネイティブsikulidモジュールが必要です。

        Args:
            message: Message to display / 表示するメッセージ

        Returns:
            Selected Region / 選択した領域

        Raises:
            RuntimeError: Native module required / ネイティブモジュールが必要
        """
        raise RuntimeError(
            "Interactive selection requires native sikulid module. "
            "Install with: pip install sikulid\n"
            "インタラクティブ選択にはネイティブsikulidモジュールが必要です。"
        )

    def showMonitors(self):
        """Show monitor information / モニター情報を表示

        Displays information about all connected monitors.
        接続されている全モニターの情報を表示します。
        """
        num_screens = Screen.get_number_screens()
        print(f"=== Monitor Information / モニター情報 ===")
        print(f"Number of screens / 画面数: {num_screens}")
        for i in range(num_screens):
            s = Screen(i)
            print(f"  Screen {i}: {s.w}x{s.h} at ({s.x}, {s.y})")

    @staticmethod
    def resetMonitors():
        """Reset monitor configuration / モニター設定をリセット

        Re-detects all connected monitors.
        接続されている全モニターを再検出します。
        """
        # Re-initialize screen detection
        # In pure Python mode, this just returns the number of screens
        return Screen.get_number_screens()

    def getID(self):
        """Get screen ID / 画面IDを取得

        Returns:
            Screen index / 画面インデックス
        """
        return self.index

    def getIDEScreenID(self):
        """Get IDE screen ID (alias for getID) / IDE画面IDを取得（getIDのエイリアス）

        Returns:
            Screen index / 画面インデックス
        """
        return self.index

    def setROI(self, x=None, y=None, w=None, h=None, region=None):
        """Set Region of Interest / 関心領域を設定

        Args:
            x: X coordinate or Region / X座標またはRegion
            y: Y coordinate / Y座標
            w: Width / 幅
            h: Height / 高さ
            region: Region object (alternative to x,y,w,h) / Regionオブジェクト

        Returns:
            self for method chaining / メソッドチェーン用のself
        """
        if region is not None:
            self._roi = region
        elif x is not None and hasattr(x, 'x'):
            # x is actually a Region object
            self._roi = x
        elif x is not None and y is not None and w is not None and h is not None:
            self._roi = Region(x, y, w, h)
        else:
            self._roi = None
        return self

    def resetROI(self):
        """Reset Region of Interest to full screen / 関心領域を全画面にリセット

        Returns:
            self for method chaining / メソッドチェーン用のself
        """
        self._roi = None
        return self

    def getROI(self):
        """Get current Region of Interest / 現在の関心領域を取得

        Returns:
            Current ROI or full screen Region / 現在のROIまたは全画面Region
        """
        if hasattr(self, '_roi') and self._roi is not None:
            return self._roi
        return self.get_region()

    def setRect(self, x, y, w, h):
        """Set screen rectangle (for multi-monitor) / 画面矩形を設定（マルチモニター用）

        Args:
            x: X coordinate / X座標
            y: Y coordinate / Y座標
            w: Width / 幅
            h: Height / 高さ

        Returns:
            self for method chaining / メソッドチェーン用のself
        """
        self.x = x
        self.y = y
        self.w = w
        self.h = h
        return self

    def getRect(self):
        """Get screen rectangle / 画面矩形を取得

        Returns:
            Tuple of (x, y, w, h) / (x, y, w, h) のタプル
        """
        return (self.x, self.y, self.w, self.h)

    def getCenter(self):
        """Get center point of screen / 画面の中心点を取得

        Returns:
            Tuple (x, y) of center point / 中心点のタプル (x, y)
        """
        return self.center()

    def getTopLeft(self):
        """Get top-left corner of screen / 画面の左上隅を取得

        Returns:
            Tuple (x, y) / タプル (x, y)
        """
        return self.top_left()

    def getTopRight(self):
        """Get top-right corner of screen / 画面の右上隅を取得

        Returns:
            Tuple (x, y) / タプル (x, y)
        """
        return self.top_right()

    def getBottomLeft(self):
        """Get bottom-left corner of screen / 画面の左下隅を取得

        Returns:
            Tuple (x, y) / タプル (x, y)
        """
        return self.bottom_left()

    def getBottomRight(self):
        """Get bottom-right corner of screen / 画面の右下隅を取得

        Returns:
            Tuple (x, y) / タプル (x, y)
        """
        return self.bottom_right()

    def __repr__(self):
        return f"Screen({self.index})"


# Global screen instance
SCREEN = Screen(0)
