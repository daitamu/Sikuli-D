---
layout: home

hero:
  name: "Sikuli-D"
  text: "GUI Automation Tool"
  tagline: "SikuliX-compatible automation with Rust performance and image recognition"
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started/
    - theme: alt
      text: API Reference
      link: /api/
    - theme: alt
      text: View on GitHub
      link: https://github.com/daitamu/Sikuli-D

features:
  - icon: 🚀
    title: Rust Performance
    details: Built with Rust for high-speed execution and memory efficiency
  - icon: 🖼️
    title: Image Recognition
    details: Powerful image matching using OpenCV for reliable GUI automation
  - icon: 🐍
    title: Python Compatible
    details: Run SikuliX Python 2/3 scripts with automatic conversion support
  - icon: 🔍
    title: OCR Support
    details: Text recognition powered by Tesseract 5 for advanced automation
  - icon: 💻
    title: Cross-Platform
    details: Works on Windows, macOS, and Linux with native performance
  - icon: 🛠️
    title: Modern IDE
    details: Tauri-based IDE with Monaco editor and inline image preview
---

## Quick Example

```python
from sikulid import *

# Find and click an image on screen
click("button.png")

# Wait for image to appear and type text
wait("input_field.png", 5)
type("Hello, Sikuli-D!")

# Read text from screen using OCR
text = Screen().text()
print(text)
```

## Architecture

Sikuli-D consists of three main components:

- **Core Library** (`core-rs`): Shared Rust library for image recognition and automation
- **IDE** (`ide-rs-tauri`): Desktop application built with Tauri for script development
- **Runtime** (`runtime-rs`): Python execution environment with PyO3 bindings

## Why Sikuli-D?

Sikuli-D is a modern reimplementation of SikuliX, offering:

- **Better Performance**: Rust provides faster execution than Java-based SikuliX
- **Lower Memory Usage**: Efficient memory management without JVM overhead
- **Modern Tooling**: Built with latest technologies (Tauri 2.x, PyO3)
- **Active Development**: Regular updates and improvements
- **Full Compatibility**: Run existing SikuliX scripts without modification

## License

Sikuli-D is released under the [Apache License 2.0](https://github.com/daitamu/Sikuli-D/blob/master/LICENSE).
