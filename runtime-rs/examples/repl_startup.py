#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
SikuliX REPL Startup Script Example
SikuliX REPL 起動スクリプトの例

This script is automatically executed when starting the REPL with --startup option.
このスクリプトは --startup オプションで REPL を起動すると自動的に実行されます。

Usage:
    sikulix repl --startup examples/repl_startup.py
"""

import sys
import os
from pathlib import Path

# Auto-import SikuliX API
try:
    from sikulix_api import *
    print("✓ SikuliX API imported")
except ImportError as e:
    print(f"✗ Failed to import SikuliX API: {e}")

# Set up common paths
HOME = Path.home()
DESKTOP = HOME / "Desktop"
DOCUMENTS = HOME / "Documents"

print(f"Home: {HOME}")
print(f"Desktop: {DESKTOP}")

# Helper functions
def quick_find(image, timeout=3):
    """
    Quick find with error handling
    エラー処理付きクイック検索

    Args:
        image: Image filename or path
        timeout: Wait timeout in seconds

    Returns:
        Match object or None
    """
    try:
        return wait(image, timeout)
    except Exception as e:
        print(f"✗ Not found: {image}")
        return None

def safe_click(image, timeout=3):
    """
    Find and click with error handling
    エラー処理付き検索とクリック

    Args:
        image: Image filename or path
        timeout: Wait timeout in seconds

    Returns:
        True if clicked, False otherwise
    """
    m = quick_find(image, timeout)
    if m:
        click(m)
        print(f"✓ Clicked: {image}")
        return True
    else:
        print(f"✗ Failed to click: {image}")
        return False

def wait_and_type(image, text, timeout=3):
    """
    Find element and type text
    要素を見つけてテキストを入力

    Args:
        image: Image filename or path
        text: Text to type
        timeout: Wait timeout in seconds

    Returns:
        True if successful, False otherwise
    """
    m = quick_find(image, timeout)
    if m:
        click(m)
        type(text)
        print(f"✓ Typed into {image}: {text}")
        return True
    else:
        print(f"✗ Failed to find: {image}")
        return False

def capture_to_desktop(name="capture.png"):
    """
    Capture screen to desktop
    画面をデスクトップにキャプチャ

    Args:
        name: Filename for capture

    Returns:
        Path to saved file
    """
    output = DESKTOP / name
    screen = Screen()
    img = screen.capture()
    img.save(str(output))
    print(f"✓ Saved to: {output}")
    return output

def list_vars():
    """
    List user-defined variables
    ユーザー定義変数をリスト
    """
    print("\n=== User Variables ===")
    user_vars = {k: v for k, v in globals().items()
                 if not k.startswith('_') and k not in dir(__builtins__)}
    for name, value in sorted(user_vars.items()):
        value_str = str(value)
        if len(value_str) > 50:
            value_str = value_str[:50] + "..."
        print(f"  {name:20} = {value_str}")

# Aliases for convenience
qf = quick_find
sc = safe_click
wt = wait_and_type

# Print available helper functions
print("\n=== Helper Functions ===")
print("  quick_find(img, timeout=3)     - Find with error handling")
print("  safe_click(img, timeout=3)     - Click with error handling")
print("  wait_and_type(img, text, t=3)  - Type into element")
print("  capture_to_desktop(name)       - Capture screen")
print("  list_vars()                    - Show user variables")
print("\n  Aliases: qf, sc, wt")
print("\nType :help for REPL commands")
print("Ready for automation!\n")
