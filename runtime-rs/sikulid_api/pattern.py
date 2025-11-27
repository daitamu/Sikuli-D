# -*- coding: utf-8 -*-
"""
Pattern module - Image pattern for matching
パターンモジュール - マッチング用の画像パターン

This module provides the Pattern class for image pattern matching.
このモジュールは画像パターンマッチングのためのPatternクラスを提供します。
"""

import os


class Pattern:
    """
    A pattern to be used in image search operations.
    画像検索操作で使用されるパターン。

    Attributes:
        path (str): Path to the image file / 画像ファイルへのパス
        similarity (float): Minimum similarity threshold (0.0-1.0) / 最小類似度閾値 (0.0-1.0)
        offset (tuple): Target offset (x, y) from center / 中心からのターゲットオフセット (x, y)

    Example / 例:
        pattern = Pattern("button.png")
        pattern = Pattern("button.png", 0.9)
        pattern = pattern.similar(0.8).targetOffset(10, 0)
    """

    def __init__(self, path, similarity=None):
        """
        Create a new Pattern.
        新しいPatternを作成。

        Args:
            path (str): Path to the image file / 画像ファイルへのパス
            similarity (float, optional): Similarity threshold (0.0-1.0) / 類似度閾値 (0.0-1.0)
        """
        self._path = str(path)
        self._similarity = similarity if similarity is not None else 0.7
        self._offset = (0, 0)

    @property
    def path(self):
        """Get image path / 画像パスを取得"""
        return self._path

    @property
    def similarity(self):
        """Get similarity threshold / 類似度閾値を取得"""
        return self._similarity

    def similar(self, similarity):
        """
        Create a new Pattern with different similarity threshold.
        異なる類似度閾値で新しいPatternを作成。

        Args:
            similarity (float): New similarity threshold (0.0-1.0) / 新しい類似度閾値 (0.0-1.0)

        Returns:
            Pattern: New pattern with updated similarity / 更新された類似度の新しいパターン
        """
        new_pattern = Pattern(self._path, max(0.0, min(1.0, similarity)))
        new_pattern._offset = self._offset
        return new_pattern

    def targetOffset(self, x, y):
        """
        Create a new Pattern with target offset.
        ターゲットオフセット付きの新しいPatternを作成。

        Args:
            x (int): X offset from center / 中心からのXオフセット
            y (int): Y offset from center / 中心からのYオフセット

        Returns:
            Pattern: New pattern with offset / オフセット付きの新しいパターン
        """
        new_pattern = Pattern(self._path, self._similarity)
        new_pattern._offset = (x, y)
        return new_pattern

    def getFilename(self):
        """
        Get the filename without directory.
        ディレクトリなしのファイル名を取得。

        Returns:
            str: Filename / ファイル名
        """
        return os.path.basename(self._path)

    def getTargetOffset(self):
        """
        Get the target offset.
        ターゲットオフセットを取得。

        Returns:
            tuple: (x, y) offset / (x, y) オフセット
        """
        return self._offset

    def isValid(self):
        """
        Check if pattern image file exists.
        パターン画像ファイルが存在するか確認。

        Returns:
            bool: True if file exists / ファイルが存在すればTrue
        """
        return os.path.isfile(self._path)

    def __repr__(self):
        """String representation / 文字列表現"""
        return "Pattern('{}', {:.2f})".format(self._path, self._similarity)

    def __str__(self):
        """String representation / 文字列表現"""
        return self.__repr__()
