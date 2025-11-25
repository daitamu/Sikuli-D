"""
Sikuli-D Next Generation - Python API
=====================================

High-performance GUI automation with image recognition.

Usage:
    from sikulix import Screen, Pattern, Region

    screen = Screen()
    match = screen.find("button.png")
    if match:
        match.click()

Python 2/3 dual runtime support with automatic syntax detection.
"""

__version__ = "0.1.0"
__author__ = "Sikuli-D Team"

# Import from Rust extension when available
try:
    from sikulix_core import (
        Region as _Region,
        Match as _Match,
        Pattern as _Pattern,
        ImageMatcher as _ImageMatcher,
        Screen as _Screen,
        Mouse as _Mouse,
        Keyboard as _Keyboard,
        Key as _Key,
        PythonVersion,
        SyntaxAnalyzer,
    )
    _RUST_AVAILABLE = True
except ImportError:
    _RUST_AVAILABLE = False
    print("Warning: sikulix_core not available. Using pure Python fallback.")


class Region:
    """Represents a rectangular region on the screen."""

    def __init__(self, x=0, y=0, width=0, height=0):
        self.x = x
        self.y = y
        self.width = width
        self.height = height
        if _RUST_AVAILABLE:
            self._inner = _Region(x, y, width, height)

    def center(self):
        """Get the center point of the region."""
        return (
            self.x + self.width // 2,
            self.y + self.height // 2
        )

    def contains(self, x, y):
        """Check if a point is inside the region."""
        return (self.x <= x < self.x + self.width and
                self.y <= y < self.y + self.height)

    def click(self):
        """Click at the center of this region."""
        cx, cy = self.center()
        Mouse.click(cx, cy)

    def __repr__(self):
        return f"Region({self.x}, {self.y}, {self.width}, {self.height})"


class Match(Region):
    """Result from an image search, including match score."""

    def __init__(self, region, score):
        super().__init__(region.x, region.y, region.width, region.height)
        self.score = score

    def __repr__(self):
        return f"Match({self.x}, {self.y}, {self.width}, {self.height}, score={self.score:.3f})"


class Pattern:
    """Pattern for image matching."""

    def __init__(self, image_path_or_data):
        """
        Create a pattern from an image file path or bytes.

        Args:
            image_path_or_data: Path to PNG image or raw PNG bytes
        """
        if isinstance(image_path_or_data, str):
            with open(image_path_or_data, 'rb') as f:
                self.image_data = f.read()
        else:
            self.image_data = image_path_or_data

        self.similarity = 0.7
        self.target_offset = (0, 0)

    def similar(self, similarity):
        """Set similarity threshold (0.0 - 1.0)."""
        self.similarity = max(0.0, min(1.0, similarity))
        return self

    def targetOffset(self, x, y):
        """Set target offset from center."""
        self.target_offset = (x, y)
        return self


class Screen:
    """Screen capture and control."""

    def __init__(self, index=0):
        """
        Create a Screen for the given monitor index.

        Args:
            index: Monitor index (0 = primary)
        """
        self.index = index
        if _RUST_AVAILABLE:
            self._inner = _Screen(index)

    def capture(self, region=None):
        """
        Capture the screen or a region.

        Args:
            region: Optional Region to capture

        Returns:
            Captured image data as bytes
        """
        if not _RUST_AVAILABLE:
            raise RuntimeError("Screen capture requires sikulix_core")

        if region:
            return self._inner.capture_region(region._inner)
        return self._inner.capture()

    def find(self, pattern, timeout=3.0):
        """
        Find a pattern on the screen.

        Args:
            pattern: Pattern object or image path
            timeout: Maximum time to search (seconds)

        Returns:
            Match object if found, None otherwise
        """
        if not _RUST_AVAILABLE:
            raise RuntimeError("Image matching requires sikulix_core")

        if isinstance(pattern, str):
            pattern = Pattern(pattern)

        # TODO: Implement timeout loop
        result = self._inner.find(pattern.image_data, pattern.similarity)
        if result:
            region = Region(result.x, result.y, result.width, result.height)
            return Match(region, result.score)
        return None

    def findAll(self, pattern):
        """
        Find all occurrences of a pattern on the screen.

        Args:
            pattern: Pattern object or image path

        Returns:
            List of Match objects
        """
        if not _RUST_AVAILABLE:
            raise RuntimeError("Image matching requires sikulix_core")

        if isinstance(pattern, str):
            pattern = Pattern(pattern)

        results = self._inner.find_all(pattern.image_data, pattern.similarity)
        matches = []
        for r in results:
            region = Region(r.x, r.y, r.width, r.height)
            matches.append(Match(region, r.score))
        return matches

    def exists(self, pattern, timeout=3.0):
        """Check if a pattern exists on the screen."""
        return self.find(pattern, timeout) is not None

    def click(self, pattern_or_location, timeout=3.0):
        """Click on a pattern or at a location."""
        if isinstance(pattern_or_location, (tuple, list)):
            Mouse.click(*pattern_or_location)
        else:
            match = self.find(pattern_or_location, timeout)
            if match:
                match.click()
            else:
                raise RuntimeError(f"Pattern not found: {pattern_or_location}")


class Mouse:
    """Mouse control."""

    @staticmethod
    def move(x, y):
        """Move mouse to position."""
        if _RUST_AVAILABLE:
            _Mouse.move_to(x, y)

    @staticmethod
    def click(x=None, y=None):
        """Click at position (or current position if not specified)."""
        if x is not None and y is not None:
            Mouse.move(x, y)
        if _RUST_AVAILABLE:
            _Mouse.click()

    @staticmethod
    def doubleClick(x=None, y=None):
        """Double-click at position."""
        if x is not None and y is not None:
            Mouse.move(x, y)
        if _RUST_AVAILABLE:
            _Mouse.double_click()

    @staticmethod
    def rightClick(x=None, y=None):
        """Right-click at position."""
        if x is not None and y is not None:
            Mouse.move(x, y)
        if _RUST_AVAILABLE:
            _Mouse.right_click()

    @staticmethod
    def position():
        """Get current mouse position."""
        if _RUST_AVAILABLE:
            return _Mouse.position()
        return (0, 0)


class Keyboard:
    """Keyboard control."""

    @staticmethod
    def type(text):
        """Type a string."""
        if _RUST_AVAILABLE:
            _Keyboard.type_text(text)

    @staticmethod
    def press(key):
        """Press a key."""
        if _RUST_AVAILABLE:
            _Keyboard.press(key)

    @staticmethod
    def release(key):
        """Release a key."""
        if _RUST_AVAILABLE:
            _Keyboard.release(key)

    @staticmethod
    def hotkey(*keys):
        """Press a key combination."""
        if _RUST_AVAILABLE:
            _Keyboard.hotkey(list(keys))


# Key constants (re-exported from Rust)
if _RUST_AVAILABLE:
    Key = _Key
else:
    class Key:
        """Key codes for keyboard input."""
        SHIFT = "shift"
        CTRL = "ctrl"
        ALT = "alt"
        META = "meta"
        ENTER = "enter"
        TAB = "tab"
        SPACE = "space"
        BACKSPACE = "backspace"
        DELETE = "delete"
        ESCAPE = "escape"
        # ... more keys


def main():
    """Entry point for standalone executable."""
    import sys

    if len(sys.argv) < 2:
        print(f"Sikuli-D Next Generation v{__version__}")
        print("Usage: sikulix <script.py>")
        return

    script_path = sys.argv[1]

    # Check Python version of the script
    with open(script_path, 'r', encoding='utf-8') as f:
        source = f.read()

    if _RUST_AVAILABLE:
        version = SyntaxAnalyzer.detect_version(source)
        if version == PythonVersion.Mixed:
            print("Error: Script contains mixed Python 2 and 3 syntax")
            sys.exit(1)
        elif version == PythonVersion.Python2:
            print("Warning: Python 2 syntax detected. Please update to Python 3.")
        print(f"Detected Python version: {version}")

    # Execute the script
    exec(compile(source, script_path, 'exec'), {
        '__name__': '__main__',
        '__file__': script_path,
        'Screen': Screen,
        'Region': Region,
        'Pattern': Pattern,
        'Match': Match,
        'Mouse': Mouse,
        'Keyboard': Keyboard,
        'Key': Key,
    })


# Convenience exports
__all__ = [
    'Screen',
    'Region',
    'Pattern',
    'Match',
    'Mouse',
    'Keyboard',
    'Key',
    'PythonVersion',
    'SyntaxAnalyzer',
    'main',
]
