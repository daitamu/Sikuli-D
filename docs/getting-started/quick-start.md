# Quick Start

Learn the basics of Sikuli-D with this quick tutorial.

## Your First Script

Let's create a simple automation script that opens Notepad and types some text.

### Step 1: Capture Screenshots

Before writing the script, you need to capture images of the GUI elements you want to interact with:

1. Open the Sikuli-D IDE
2. Click the camera icon or press `Ctrl+Shift+2` to capture a screenshot
3. Select the area you want to capture (e.g., the Start button)
4. Save the image as `start_button.png`
5. Repeat for any other GUI elements you need

### Step 2: Write the Script

Create a new Python file `hello_sikuli.py`:

```python
from sikulid import *

# Click the Windows Start button
click("start_button.png")

# Wait 1 second for menu to appear
wait(1)

# Type "notepad" to search for Notepad
type("notepad" + Key.ENTER)

# Wait for Notepad window to appear
wait("notepad_window.png", 5)

# Type some text
type("Hello, Sikuli-D!")
```

### Step 3: Run the Script

Run your script from the IDE or command line:

```bash
python hello_sikuli.py
```

Watch as Sikuli-D automatically:
1. Clicks the Start button
2. Types "notepad" and presses Enter
3. Waits for Notepad to open
4. Types "Hello, Sikuli-D!" in the text editor

## Basic Operations

### Finding Images

```python
# Find an image on screen
if exists("button.png"):
    print("Button found!")
else:
    print("Button not found")
```

### Clicking

```python
# Simple click
click("button.png")

# Double click
doubleClick("file.png")

# Right click
rightClick("item.png")
```

### Typing Text

```python
# Type text
type("Hello, World!")

# Type text with special keys
type("username" + Key.TAB + "password" + Key.ENTER)

# Paste from clipboard
paste("text to paste")
```

### Waiting

```python
# Wait for image to appear (timeout: 3 seconds)
wait("loading.png", 3)

# Wait for image to disappear
waitVanish("loading.png", 10)
```

### Regions

```python
# Get the entire screen
screen = Screen()

# Define a custom region
region = Region(100, 100, 400, 300)  # x, y, width, height

# Find within a region
match = region.find("button.png")
region.click("button.png")
```

### Reading Text (OCR)

```python
# Read all text from screen
text = Screen().text()
print(text)

# Read text from a specific region
region = Region(100, 100, 400, 300)
text = region.text()
print(text)
```

## Pattern Matching

Control how strictly images are matched:

```python
# Exact match (similarity: 0.99)
click(Pattern("button.png").exact())

# Loose match (similarity: 0.7)
click(Pattern("button.png").similar(0.7))

# Click offset from image center
click(Pattern("icon.png").targetOffset(50, 20))
```

## Error Handling

```python
try:
    click("button.png")
except FindFailed:
    print("Could not find button.png")
    # Handle the error gracefully
```

## Best Practices

1. **Use Unique Images**: Capture distinctive GUI elements to avoid false matches
2. **Set Appropriate Timeouts**: Give applications time to respond
3. **Handle Errors**: Use try/except blocks for robust scripts
4. **Test Regions**: Limit search to specific screen areas for faster execution
5. **Adjust Similarity**: Lower similarity (0.7-0.8) for changing GUI elements

## Common Patterns

### Wait and Click Pattern

```python
# Wait for element, then click
wait("button.png", 10)
click("button.png")

# Or combine in one line
click(wait("button.png", 10))
```

### Loop Until Found

```python
# Keep trying until found
while not exists("ready.png"):
    wait(1)
print("Application is ready!")
```

### Multiple Screen Handling

```python
# Work with specific monitor
screen1 = Screen(0)
screen2 = Screen(1)

screen1.click("button.png")
screen2.type("text on second monitor")
```

## Next Steps

- Explore the [API Reference](/api/) for complete documentation
- Check out [Tutorials](/tutorials/) for more examples
- Learn about [advanced patterns and techniques](/tutorials/image-recognition)

## Getting Help

- Review [Troubleshooting](/troubleshooting/) for common issues
- Visit [FAQ](/troubleshooting/faq) for frequently asked questions
- Join discussions on [GitHub](https://github.com/daitamu/Sikuli-D)
