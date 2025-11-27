"""
Match class - Pure Python fallback implementation
Match クラス - Pure Python フォールバック実装
"""

from .region import Region


class Match(Region):
    """Represents a match result from image finding / 画像検索結果を表す

    A Match is a Region with additional score information.
    Match は追加のスコア情報を持つ Region です。
    """

    def __init__(self, x=0, y=0, w=0, h=0, score=0.0):
        """Create a new Match / 新しい Match を作成

        Args:
            x: X coordinate of top-left corner / 左上隅のX座標
            y: Y coordinate of top-left corner / 左上隅のY座標
            w: Width / 幅
            h: Height / 高さ
            score: Match score (0.0-1.0) / マッチスコア (0.0-1.0)
        """
        super().__init__(x, y, w, h)
        self.score = score
        self._target_offset = (0, 0)

    def get_score(self):
        """Get match score / マッチスコアを取得

        Returns:
            Score between 0.0 and 1.0 / 0.0から1.0の間のスコア
        """
        return self.score

    def target(self):
        """Get target location for clicking / クリック用のターゲット位置を取得

        Returns:
            Tuple (x, y) of target location / ターゲット位置のタプル (x, y)
        """
        cx, cy = self.center()
        return (cx + self._target_offset[0], cy + self._target_offset[1])

    def set_target_offset(self, dx, dy):
        """Set offset from center for click target / クリックターゲットの中心からのオフセットを設定

        Args:
            dx: X offset / Xオフセット
            dy: Y offset / Yオフセット
        """
        self._target_offset = (dx, dy)

    def click(self):
        """Click on this match / このマッチをクリック"""
        from . import click as global_click
        global_click(self.target())

    def double_click(self):
        """Double click on this match / このマッチをダブルクリック"""
        from . import double_click as global_double_click
        global_double_click(self.target())

    def right_click(self):
        """Right click on this match / このマッチを右クリック"""
        from . import right_click as global_right_click
        global_right_click(self.target())

    def highlight(self, seconds=None):
        """Highlight this match / このマッチをハイライト

        Args:
            seconds: Duration in seconds / 継続時間（秒）
        """
        from . import highlight as global_highlight
        global_highlight(self, seconds)

    def __repr__(self):
        return f"Match({self.x}, {self.y}, {self.w}, {self.h}, score={self.score:.3f})"
