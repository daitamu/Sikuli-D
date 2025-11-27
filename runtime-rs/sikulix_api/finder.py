"""
Finder functions - Pure Python fallback implementation
検索関数 - Pure Python フォールバック実装

Note: This fallback uses subprocess to call the sikulix CLI.
      For best performance, use the native module.
注意: このフォールバックは sikulix CLI をサブプロセスで呼び出します。
      最高のパフォーマンスにはネイティブモジュールを使用してください。
"""

import subprocess
import time
import json
from .match import Match
from . import Settings


def find(target, similarity=None):
    """Find target on screen / 画面上でターゲットを検索

    Args:
        target: Image path or pattern / 画像パスまたはパターン
        similarity: Minimum similarity (0.0-1.0) / 最小類似度

    Returns:
        Match object if found, None otherwise / 見つかれば Match、なければ None
    """
    if similarity is None:
        similarity = Settings.MinSimilarity

    try:
        # Call sikulix CLI
        result = subprocess.run(
            ["sikulix", "find", str(target), "--similarity", str(similarity)],
            capture_output=True,
            text=True,
            timeout=30
        )

        if result.returncode == 0:
            # Parse output: "Found: x=100, y=200, w=50, h=50, score=0.95"
            output = result.stdout.strip()
            if output.startswith("Found:"):
                parts = output.replace("Found:", "").strip().split(",")
                data = {}
                for part in parts:
                    key, val = part.strip().split("=")
                    data[key.strip()] = float(val.strip())

                return Match(
                    x=int(data.get("x", 0)),
                    y=int(data.get("y", 0)),
                    w=int(data.get("w", 0)),
                    h=int(data.get("h", 0)),
                    score=data.get("score", 0.0)
                )

        return None

    except FileNotFoundError:
        raise RuntimeError(
            "sikulix CLI not found. Please install the SikuliX runtime "
            "or use the native Python module."
        )
    except subprocess.TimeoutExpired:
        raise RuntimeError("Find operation timed out")


def find_all(target, similarity=None):
    """Find all occurrences of target on screen / 画面上のターゲットの全出現を検索

    Args:
        target: Image path or pattern / 画像パスまたはパターン
        similarity: Minimum similarity (0.0-1.0) / 最小類似度

    Returns:
        List of Match objects / Match オブジェクトのリスト
    """
    if similarity is None:
        similarity = Settings.MinSimilarity

    try:
        result = subprocess.run(
            ["sikulix", "find", str(target), "--similarity", str(similarity), "--all"],
            capture_output=True,
            text=True,
            timeout=30
        )

        matches = []
        if result.returncode == 0:
            # Parse output: "Found N matches:\n  [1] x=..., y=..., ..."
            lines = result.stdout.strip().split("\n")
            for line in lines[1:]:  # Skip "Found N matches:" line
                line = line.strip()
                if line.startswith("["):
                    # Parse "[N] x=100, y=200, w=50, h=50, score=0.95"
                    parts = line.split("]", 1)[1].strip().split(",")
                    data = {}
                    for part in parts:
                        key, val = part.strip().split("=")
                        data[key.strip()] = float(val.strip())

                    matches.append(Match(
                        x=int(data.get("x", 0)),
                        y=int(data.get("y", 0)),
                        w=int(data.get("w", 0)),
                        h=int(data.get("h", 0)),
                        score=data.get("score", 0.0)
                    ))

        return matches

    except FileNotFoundError:
        raise RuntimeError(
            "sikulix CLI not found. Please install the SikuliX runtime "
            "or use the native Python module."
        )
    except subprocess.TimeoutExpired:
        raise RuntimeError("FindAll operation timed out")


def wait(target, timeout=None, similarity=None):
    """Wait for target to appear on screen / ターゲットが画面に表示されるまで待機

    Args:
        target: Image path or pattern / 画像パスまたはパターン
        timeout: Wait time in seconds / 待機時間（秒）
        similarity: Minimum similarity (0.0-1.0) / 最小類似度

    Returns:
        Match object if found / 見つかれば Match オブジェクト

    Raises:
        FindFailed: If target not found within timeout / タイムアウト内に見つからなければ
    """
    if timeout is None:
        timeout = Settings.AutoWaitTimeout
    if similarity is None:
        similarity = Settings.MinSimilarity

    start_time = time.time()
    scan_interval = 1.0 / Settings.WaitScanRate

    while True:
        match = find(target, similarity)
        if match:
            return match

        elapsed = time.time() - start_time
        if elapsed >= timeout:
            raise FindFailed(f"FindFailed: {target} not found within {timeout} seconds")

        time.sleep(scan_interval)


class FindFailed(Exception):
    """Exception raised when find operation fails / 検索操作が失敗した時に発生する例外"""
    pass
