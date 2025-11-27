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

    def __repr__(self):
        return f"Screen({self.index})"


# Global screen instance
SCREEN = Screen(0)
