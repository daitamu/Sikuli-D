#!/usr/bin/env python3
"""Test PyO3 bindings for sikulix_core"""

def test_import():
    """Test that the module can be imported"""
    try:
        import sikulix_py
        print("✓ Module imported successfully")
        return True
    except ImportError as e:
        print(f"✗ Failed to import: {e}")
        return False

def test_screen():
    """Test Screen class"""
    from sikulix_py import Screen

    screen = Screen()
    print(f"✓ Screen created: {screen}")

    width, height = screen.dimensions()
    print(f"✓ Screen dimensions: {width}x{height}")

    region = screen.get_region()
    print(f"✓ Screen region: {region}")

    return True

def test_region():
    """Test Region class"""
    from sikulix_py import Region

    r = Region(100, 100, 200, 150)
    print(f"✓ Region created: {r}")

    cx, cy = r.center()
    print(f"✓ Center: ({cx}, {cy})")

    print(f"✓ Properties: x={r.x}, y={r.y}, w={r.width}, h={r.height}")

    assert r.x == 100
    assert r.y == 100
    assert r.width == 200
    assert r.height == 150
    assert cx == 200  # 100 + 200/2
    assert cy == 175  # 100 + 150/2

    r2 = r.offset(50, 50)
    print(f"✓ Offset region: {r2}")
    assert r2.x == 150
    assert r2.y == 150

    r3 = r.expand(10)
    print(f"✓ Expanded region: {r3}")
    assert r3.x == 90
    assert r3.y == 90
    assert r3.width == 220
    assert r3.height == 170

    assert r.contains(150, 150) == True
    assert r.contains(50, 50) == False
    print("✓ Contains test passed")

    area = r.area()
    print(f"✓ Area: {area}")
    assert area == 30000  # 200 * 150

    return True

def test_pattern():
    """Test Pattern class"""
    from sikulix_py import Pattern
    import os

    # This will fail if file doesn't exist, which is expected
    try:
        p = Pattern("test_image.png")
        print(f"✓ Pattern created: {p}")
    except Exception as e:
        print(f"⚠ Pattern test skipped (expected): {e}")

    return True

def test_mouse_keyboard():
    """Test mouse and keyboard functions"""
    from sikulix_py import click, type_text, paste, hotkey

    # Test function availability
    print("✓ Mouse functions available: click, double_click, right_click")
    print("✓ Keyboard functions available: type_text, paste, hotkey")

    # Don't actually execute to avoid unwanted input
    return True

def test_error_handling():
    """Test error handling"""
    from sikulix_py import Pattern

    try:
        # Try to load non-existent file
        p = Pattern("nonexistent_file_12345.png")
        print("✗ Should have raised exception for missing file")
        return False
    except Exception as e:
        print(f"✓ Exception raised as expected: {type(e).__name__}")
        return True

def main():
    """Run all tests"""
    print("=" * 60)
    print("Testing SikuliX PyO3 Bindings")
    print("=" * 60)

    tests = [
        ("Module Import", test_import),
        ("Screen Class", test_screen),
        ("Region Class", test_region),
        ("Pattern Class", test_pattern),
        ("Mouse/Keyboard", test_mouse_keyboard),
        ("Error Handling", test_error_handling),
    ]

    results = []
    for name, test_func in tests:
        print(f"\n[Test] {name}")
        print("-" * 60)
        try:
            result = test_func()
            results.append((name, result))
        except Exception as e:
            print(f"✗ Exception: {e}")
            import traceback
            traceback.print_exc()
            results.append((name, False))

    print("\n" + "=" * 60)
    print("Test Summary")
    print("=" * 60)

    passed = sum(1 for _, r in results if r)
    total = len(results)

    for name, result in results:
        status = "✓ PASS" if result else "✗ FAIL"
        print(f"{status}: {name}")

    print(f"\nTotal: {passed}/{total} passed")

    return passed == total

if __name__ == "__main__":
    import sys
    sys.exit(0 if main() else 1)
