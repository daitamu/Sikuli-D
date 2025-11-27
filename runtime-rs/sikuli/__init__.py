"""
SikuliX Compatibility Module for Sikuli-D
SikuliX 互換モジュール for Sikuli-D

This module provides compatibility with SikuliX Java-based scripts.
このモジュールはSikuliX Java版スクリプトとの互換性を提供します。

Usage:
    from sikuli import *
"""

import sys
import os

# Add parent directory to path if needed
# 必要に応じて親ディレクトリをパスに追加
sikuli_d_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if sikuli_d_dir not in sys.path:
    sys.path.insert(0, sikuli_d_dir)

# Re-export everything from sikulid_api
# sikulid_api から全てを再エクスポート
from sikulid_api import *
from sikulid_api import __version__, __all__
