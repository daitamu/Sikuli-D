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

    def type(self, text):
        """Type text in this region / この領域内でテキストを入力

        Args:
            text: Text to type / 入力するテキスト
        """
        self.click()
        from . import type_text
        type_text(text)

    def __repr__(self):
        return f"Region({self.x}, {self.y}, {self.w}, {self.h})"

    def __eq__(self, other):
        if not isinstance(other, Region):
            return False
        return (self.x == other.x and self.y == other.y and
                self.w == other.w and self.h == other.h)
