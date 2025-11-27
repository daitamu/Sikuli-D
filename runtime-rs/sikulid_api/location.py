"""
Location class - Pure Python implementation
Location クラス - Pure Python 実装
"""


class Location:
    """Represents a point on the screen / 画面上の点を表す

    A Location is a simple x, y coordinate pair.
    Location は単純な x, y 座標のペアです。
    """

    def __init__(self, x, y):
        """Create a new Location / 新しい Location を作成

        Args:
            x: X coordinate / X座標
            y: Y coordinate / Y座標
        """
        self.x = int(x)
        self.y = int(y)

    def getX(self):
        """Get X coordinate / X座標を取得

        Returns:
            X coordinate / X座標
        """
        return self.x

    def getY(self):
        """Get Y coordinate / Y座標を取得

        Returns:
            Y coordinate / Y座標
        """
        return self.y

    def setLocation(self, x, y):
        """Set location / 位置を設定

        Args:
            x: New X coordinate / 新しいX座標
            y: New Y coordinate / 新しいY座標

        Returns:
            Self for chaining / チェーン用の自身
        """
        self.x = int(x)
        self.y = int(y)
        return self

    def offset(self, dx, dy):
        """Create a new Location offset from this one / オフセットした新しい Location を作成

        Args:
            dx: X offset / Xオフセット
            dy: Y offset / Yオフセット

        Returns:
            New Location / 新しい Location
        """
        return Location(self.x + dx, self.y + dy)

    def above(self, dy):
        """Get location above this one / この上の位置を取得

        Args:
            dy: Distance above / 上方向の距離

        Returns:
            New Location / 新しい Location
        """
        return Location(self.x, self.y - dy)

    def below(self, dy):
        """Get location below this one / この下の位置を取得

        Args:
            dy: Distance below / 下方向の距離

        Returns:
            New Location / 新しい Location
        """
        return Location(self.x, self.y + dy)

    def left(self, dx):
        """Get location left of this one / この左の位置を取得

        Args:
            dx: Distance left / 左方向の距離

        Returns:
            New Location / 新しい Location
        """
        return Location(self.x - dx, self.y)

    def right(self, dx):
        """Get location right of this one / この右の位置を取得

        Args:
            dx: Distance right / 右方向の距離

        Returns:
            New Location / 新しい Location
        """
        return Location(self.x + dx, self.y)

    def grow(self, width, height=None):
        """Create a Region centered on this Location / この Location を中心とした Region を作成

        Args:
            width: Region width (or radius if height is None) / Region幅（heightがNoneなら半径）
            height: Region height (optional) / Region高さ（オプション）

        Returns:
            Region centered on this Location / この Location を中心とした Region
        """
        from .region import Region

        if height is None:
            height = width
        return Region(
            self.x - width // 2,
            self.y - height // 2,
            width,
            height
        )

    def __repr__(self):
        return f"Location({self.x}, {self.y})"

    def __eq__(self, other):
        if isinstance(other, Location):
            return self.x == other.x and self.y == other.y
        return False

    def __hash__(self):
        return hash((self.x, self.y))
