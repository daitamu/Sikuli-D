# API Reference

Welcome to the Sikuli-D API Reference. This documentation covers all available classes, methods, and functions.

## Core Classes

### Screen
Represents your display and provides methods for finding and interacting with GUI elements.

[View Screen API →](./screen.md)

### Region
A rectangular area on the screen where you can perform operations.

[View Region API →](./region.md)

### Pattern
Represents an image pattern with matching settings and target offset.

[View Pattern API →](./pattern.md)

### Match
Represents a successful image match with location and score information.

[View Match API →](./match.md)

## Quick Reference

### Finding Elements

```python
from sikulid import *

# Find an image
match = find("button.png")

# Check if exists
if exists("icon.png"):
    print("Found!")

# Wait for appearance
wait("dialog.png", 10)

# Wait for disappearance
waitVanish("loading.png", 30)
```

### Mouse Actions

```python
# Click
click("button.png")

# Double click
doubleClick("file.png")

# Right click
rightClick("menu_item.png")

# Drag and drop
drag("source.png")
dropAt("target.png")

# Hover
hover("tooltip_trigger.png")
```

### Keyboard Actions

```python
# Type text
type("Hello, World!")

# Special keys
type(Key.ENTER)
type(Key.TAB)
type(Key.ESC)

# Modifiers
type("a", Key.CTRL)  # Ctrl+A
type("c", Key.CTRL)  # Ctrl+C
type("v", Key.CTRL)  # Ctrl+V

# Paste
paste("text from clipboard")
```

### OCR (Text Recognition)

```python
# Read text from screen
text = Screen().text()

# Read text from region
region = Region(100, 100, 400, 300)
text = region.text()
```

### Pattern Matching

```python
# Adjust similarity
Pattern("button.png").similar(0.8)

# Exact match
Pattern("icon.png").exact()

# Target offset
Pattern("label.png").targetOffset(100, 0)
```

## Special Keys

Common keyboard keys available in `Key` class:

| Key | Description |
|-----|-------------|
| `Key.ENTER` | Enter/Return key |
| `Key.TAB` | Tab key |
| `Key.ESC` | Escape key |
| `Key.BACKSPACE` | Backspace key |
| `Key.DELETE` | Delete key |
| `Key.SPACE` | Space bar |
| `Key.UP`, `Key.DOWN` | Arrow keys |
| `Key.LEFT`, `Key.RIGHT` | Arrow keys |
| `Key.PAGE_UP`, `Key.PAGE_DOWN` | Page navigation |
| `Key.HOME`, `Key.END` | Home/End keys |
| `Key.F1` - `Key.F12` | Function keys |

## Modifiers

Keyboard modifiers available in `KeyModifier` class:

| Modifier | Description |
|----------|-------------|
| `Key.CTRL` | Control key |
| `Key.ALT` | Alt key |
| `Key.SHIFT` | Shift key |
| `Key.META` | Windows/Command key |

## Mouse Buttons

Mouse button constants:

| Button | Description |
|--------|-------------|
| `Button.LEFT` | Left mouse button |
| `Button.MIDDLE` | Middle mouse button |
| `Button.RIGHT` | Right mouse button |

## Exceptions

### FindFailed

Raised when an image cannot be found within the timeout period.

```python
try:
    click("button.png")
except FindFailed as e:
    print(f"Could not find image: {e}")
```

### Other Exceptions

- `ImageFileNotFound`: Image file doesn't exist
- `ScreenCaptureError`: Failed to capture screen
- `OCRError`: OCR processing failed

## Settings

### Global Settings

```python
# Set default timeout for find operations
Settings.WaitForImageTimeout = 10.0  # seconds

# Set minimum similarity for matches
Settings.MinSimilarity = 0.7

# Enable/disable visual effects
Settings.ShowActions = True
Settings.ShowClick = True

# Set default move speed
Settings.MoveMouseDelay = 0.5
```

## Compatibility Notes

Sikuli-D aims for full compatibility with SikuliX API. Most SikuliX scripts will work without modification.

### Differences from SikuliX

- Better performance with Rust implementation
- Improved OCR accuracy with Tesseract 5
- Native support for Python 3 (in addition to Python 2)
- Enhanced error messages and debugging

## Next Steps

- [Screen API Documentation](./screen.md)
- [Region API Documentation](./region.md)
- [Pattern API Documentation](./pattern.md)
- [Match API Documentation](./match.md)
- [View Tutorials](/tutorials/)
