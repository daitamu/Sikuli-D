"""
Region class - Pure Python fallback implementation
Region クラス - Pure Python フォールバック実装
"""


class Region:
    """Represents a rectangular region on screen / 画面上の矩形領域を表す

    A Region is defined by its top-left corner (x, y) and dimensions (w, h).
    Region はその左上隅 (x, y) と寸法 (w, h) で定義されます。
    """

    def __init__(self, x=0, y=0, w=0, h=0):
        """Create a new Region / 新しい Region を作成

        Args:
            x: X coordinate of top-left corner / 左上隅のX座標
            y: Y coordinate of top-left corner / 左上隅のY座標
            w: Width / 幅
            h: Height / 高さ
        """
        self.x = x
        self.y = y
        self.w = w
        self.h = h

    @property
    def width(self):
        return self.w

    @property
    def height(self):
        return self.h

    def center(self):
        """Get center point / 中心点を取得

        Returns:
            Tuple (x, y) of center point / 中心点のタプル (x, y)
        """
        return (self.x + self.w // 2, self.y + self.h // 2)

    def get_center(self):
        """Alias for center() / center() のエイリアス"""
        return self.center()

    def top_left(self):
        """Get top-left corner / 左上隅を取得"""
        return (self.x, self.y)

    def top_right(self):
        """Get top-right corner / 右上隅を取得"""
        return (self.x + self.w, self.y)

    def bottom_left(self):
        """Get bottom-left corner / 左下隅を取得"""
        return (self.x, self.y + self.h)

    def bottom_right(self):
        """Get bottom-right corner / 右下隅を取得"""
        return (self.x + self.w, self.y + self.h)

    def contains(self, x, y):
        """Check if point is inside region / 点が領域内にあるか確認

        Args:
            x: X coordinate / X座標
            y: Y coordinate / Y座標

        Returns:
            True if point is inside / 内部ならTrue
        """
        return (self.x <= x < self.x + self.w and
                self.y <= y < self.y + self.h)

    def intersection(self, other):
        """Get intersection with another region / 別の領域との交差を取得

        Args:
            other: Another Region / 別の Region

        Returns:
            Region of intersection, or None if no intersection /
            交差の Region、交差がなければ None
        """
        x1 = max(self.x, other.x)
        y1 = max(self.y, other.y)
        x2 = min(self.x + self.w, other.x + other.w)
        y2 = min(self.y + self.h, other.y + other.h)

        if x1 < x2 and y1 < y2:
            return Region(x1, y1, x2 - x1, y2 - y1)
        return None

    def offset(self, dx, dy):
        """Create new region offset from this one / この領域からオフセットした新しい領域を作成

        Args:
            dx: X offset / Xオフセット
            dy: Y offset / Yオフセット

        Returns:
            New Region / 新しい Region
        """
        return Region(self.x + dx, self.y + dy, self.w, self.h)

    def area(self):
        """Calculate area of this region / この領域の面積を計算

        Returns:
            Area in pixels squared / ピクセル二乗の面積
        """
        return self.w * self.h

    def grow(self, amount):
        """Create new region grown by amount / 指定量だけ拡大した新しい領域を作成

        Args:
            amount: Pixels to grow by (negative to shrink) /
                    拡大するピクセル数（負の値で縮小）

        Returns:
            New Region / 新しい Region
        """
        return Region(
            self.x - amount,
            self.y - amount,
            self.w + amount * 2,
            self.h + amount * 2
        )

    def expand(self, amount):
        """Alias for grow() - expand region by amount on all sides / grow()のエイリアス

        Args:
            amount: Pixels to expand by / 拡大するピクセル数

        Returns:
            New expanded Region / 拡大された新しい Region
        """
        return self.grow(amount)

    def find(self, target):
        """Find target in this region / この領域内でターゲットを検索

        Args:
            target: Image path or pattern / 画像パスまたはパターン

        Returns:
            Match object if found / 見つかれば Match オブジェクト
        """
        from . import find as global_find
        # TODO: Implement region-restricted search
        return global_find(target)

    def exists(self, target, timeout=0):
        """Check if target exists in this region / この領域内にターゲットが存在するか確認

        Args:
            target: Image path or pattern / 画像パスまたはパターン
            timeout: Wait time in seconds / 待機時間（秒）

        Returns:
            Match object if found, None otherwise / 見つかればMatchオブジェクト、なければNone
        """
        from . import wait as global_wait
        try:
            return global_wait(target, timeout)
        except Exception:
            return None

    def wait(self, target, timeout=None):
        """Wait for target to appear in this region / この領域内にターゲットが現れるまで待機

        Args:
            target: Image path or pattern / 画像パスまたはパターン
            timeout: Wait time in seconds / 待機時間（秒）

        Returns:
            Match object if found / 見つかればMatchオブジェクト

        Raises:
            FindFailed: If target not found within timeout / タイムアウト内にターゲットが見つからない場合
        """
        from . import wait as global_wait
        if timeout is None:
            from . import Settings
            timeout = Settings.AutoWaitTimeout if Settings else 3.0
        return global_wait(target, timeout)

    def waitVanish(self, target, timeout=None):
        """Wait for target to disappear from this region / この領域からターゲットが消えるまで待機

        Args:
            target: Image path or pattern / 画像パスまたはパターン
            timeout: Wait time in seconds / 待機時間（秒）

        Returns:
            True if vanished, False if timeout / 消えたらTrue、タイムアウトならFalse
        """
        import time
        from . import Settings
        if timeout is None:
            timeout = Settings.AutoWaitTimeout if Settings else 3.0

        start = time.time()
        scan_rate = Settings.WaitScanRate if Settings else 3.0
        while time.time() - start < timeout:
            if self.exists(target, 0) is None:
                return True
            time.sleep(1.0 / scan_rate)
        return False

    def click(self, target=None):
        """Click in this region / この領域内でクリック

        Args:
            target: Optional target to find and click /
                    検索してクリックするオプションのターゲット
        """
        from . import click as global_click
        if target:
            match = self.find(target)
            if match:
                global_click(match)
        else:
            x, y = self.center()
            global_click((x, y))

    def type(self, text, modifiers=None):
        """Type text in this region / この領域内でテキストを入力

        Args:
            text: Text to type / 入力するテキスト
            modifiers: Optional key modifiers / オプションのキー修飾子
        """
        self.click()
        from . import type_text
        type_text(text)

    def findAll(self, target):
        """Find all occurrences of target in this region / この領域内でターゲットの全出現を検索

        Args:
            target: Image path or pattern / 画像パスまたはパターン

        Returns:
            List of Match objects / Matchオブジェクトのリスト
        """
        from . import find_all as global_find_all
        return global_find_all(target)

    def highlight(self, seconds=None):
        """Highlight this region on screen / この領域を画面上でハイライト

        Args:
            seconds: Duration in seconds / 表示時間（秒）
        """
        from . import highlight as global_highlight
        global_highlight(self, seconds)

    def doubleClick(self, target=None):
        """Double-click in this region / この領域内でダブルクリック

        Args:
            target: Optional target to find and double-click /
                    検索してダブルクリックするオプションのターゲット
        """
        from . import double_click as global_double_click
        if target:
            match = self.find(target)
            if match:
                global_double_click(match)
        else:
            x, y = self.center()
            global_double_click((x, y))

    def rightClick(self, target=None):
        """Right-click in this region / この領域内で右クリック

        Args:
            target: Optional target to find and right-click /
                    検索して右クリックするオプションのターゲット
        """
        from . import right_click as global_right_click
        if target:
            match = self.find(target)
            if match:
                global_right_click(match)
        else:
            x, y = self.center()
            global_right_click((x, y))

    def hover(self, target=None):
        """Move mouse to this region / マウスをこの領域に移動

        Args:
            target: Optional target to find and hover over /
                    検索してホバーするオプションのターゲット
        """
        from . import hover as global_hover
        if target:
            match = self.find(target)
            if match:
                global_hover(match)
        else:
            x, y = self.center()
            global_hover((x, y))

    def dragDrop(self, target1, target2=None):
        """Drag from one location to another / ある場所から別の場所へドラッグ

        Args:
            target1: Start location or target / 開始位置またはターゲット
            target2: End location or target (if None, target1 is end and self is start) /
                     終了位置またはターゲット（Noneの場合、target1が終了でselfが開始）
        """
        from . import dragDrop as global_dragDrop
        if target2 is None:
            # Drag from this region to target1
            start_x, start_y = self.center()
            if hasattr(target1, 'center'):
                end_x, end_y = target1.center()
            elif isinstance(target1, (tuple, list)):
                end_x, end_y = target1[0], target1[1]
            else:
                match = self.find(target1)
                if match:
                    end_x, end_y = match.center()
                else:
                    return
            global_dragDrop(start_x, start_y, end_x, end_y)
        else:
            # Drag from target1 to target2
            if hasattr(target1, 'center'):
                start_x, start_y = target1.center()
            elif isinstance(target1, (tuple, list)):
                start_x, start_y = target1[0], target1[1]
            else:
                match = self.find(target1)
                if match:
                    start_x, start_y = match.center()
                else:
                    return

            if hasattr(target2, 'center'):
                end_x, end_y = target2.center()
            elif isinstance(target2, (tuple, list)):
                end_x, end_y = target2[0], target2[1]
            else:
                match = self.find(target2)
                if match:
                    end_x, end_y = match.center()
                else:
                    return
            global_dragDrop(start_x, start_y, end_x, end_y)

    def paste(self, text):
        """Paste text in this region / この領域内でテキストを貼り付け

        Args:
            text: Text to paste / 貼り付けるテキスト
        """
        self.click()
        from . import paste as global_paste
        global_paste(text)

    # Getter methods for SikuliX compatibility
    # SikuliX互換性のためのゲッターメソッド
    def getX(self):
        """Get X coordinate / X座標を取得"""
        return self.x

    def getY(self):
        """Get Y coordinate / Y座標を取得"""
        return self.y

    def getW(self):
        """Get width / 幅を取得"""
        return self.w

    def getH(self):
        """Get height / 高さを取得"""
        return self.h

    # Setter methods for SikuliX compatibility
    # SikuliX互換性のためのセッターメソッド
    def setX(self, x):
        """Set X coordinate / X座標を設定"""
        self.x = x
        return self

    def setY(self, y):
        """Set Y coordinate / Y座標を設定"""
        self.y = y
        return self

    def setW(self, w):
        """Set width / 幅を設定"""
        self.w = w
        return self

    def setH(self, h):
        """Set height / 高さを設定"""
        self.h = h
        return self

    # Relative region methods / 相対領域メソッド
    def above(self, height=None):
        """Get region above this one / この領域の上の領域を取得

        Args:
            height: Height of new region (default: same as this) /
                    新しい領域の高さ（デフォルト: この領域と同じ）

        Returns:
            New Region above this one / この領域の上の新しいRegion
        """
        if height is None:
            height = self.h
        return Region(self.x, self.y - height, self.w, height)

    def below(self, height=None):
        """Get region below this one / この領域の下の領域を取得

        Args:
            height: Height of new region (default: same as this) /
                    新しい領域の高さ（デフォルト: この領域と同じ）

        Returns:
            New Region below this one / この領域の下の新しいRegion
        """
        if height is None:
            height = self.h
        return Region(self.x, self.y + self.h, self.w, height)

    def left(self, width=None):
        """Get region to the left of this one / この領域の左の領域を取得

        Args:
            width: Width of new region (default: same as this) /
                   新しい領域の幅（デフォルト: この領域と同じ）

        Returns:
            New Region to the left / 左の新しいRegion
        """
        if width is None:
            width = self.w
        return Region(self.x - width, self.y, width, self.h)

    def right(self, width=None):
        """Get region to the right of this one / この領域の右の領域を取得

        Args:
            width: Width of new region (default: same as this) /
                   新しい領域の幅（デフォルト: この領域と同じ）

        Returns:
            New Region to the right / 右の新しいRegion
        """
        if width is None:
            width = self.w
        return Region(self.x + self.w, self.y, width, self.h)

    def nearby(self, amount=50):
        """Get region expanded by amount in all directions / 全方向に拡張した領域を取得

        Args:
            amount: Pixels to expand / 拡張するピクセル数

        Returns:
            New expanded Region / 拡張された新しいRegion
        """
        return self.grow(amount)

    def inside(self):
        """Return self (for SikuliX compatibility) / selfを返す（SikuliX互換性のため）"""
        return self

    def getScreen(self):
        """Get the screen containing this region / この領域を含む画面を取得

        Returns:
            Screen object / Screenオブジェクト
        """
        from .screen import Screen
        # Return screen 0 for now (TODO: determine actual screen)
        return Screen(0)

    def getLastMatch(self):
        """Get the last match found in this region / この領域で見つかった最後のマッチを取得

        Returns:
            Last Match object or None / 最後のMatchオブジェクトまたはNone
        """
        return getattr(self, '_lastMatch', None)

    def setLastMatch(self, match):
        """Set the last match / 最後のマッチを設定"""
        self._lastMatch = match

    def getLastMatches(self):
        """Get all last matches found / 見つかった全ての最後のマッチを取得

        Returns:
            List of Match objects / Matchオブジェクトのリスト
        """
        return getattr(self, '_lastMatches', [])

    def setLastMatches(self, matches):
        """Set last matches / 最後のマッチリストを設定"""
        self._lastMatches = list(matches) if matches else []

    def __repr__(self):
        return f"Region({self.x}, {self.y}, {self.w}, {self.h})"

    def __eq__(self, other):
        if not isinstance(other, Region):
            return False
        return (self.x == other.x and self.y == other.y and
                self.w == other.w and self.h == other.h)
