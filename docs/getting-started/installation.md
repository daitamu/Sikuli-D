# Installation

This guide covers how to install Sikuli-D on your system.

## Prerequisites

### For Users

- **Operating System**: Windows 10+, macOS 10.15+, or Linux (Ubuntu 20.04+)
- **Python**: Python 3.8 or later (Python 2.7 also supported)
- **Tesseract OCR**: Required for text recognition features

### For Developers

- **Rust**: 1.70 or later
- **Node.js**: 18 or later (for IDE development)
- **Cargo**: Included with Rust installation

## Installing Sikuli-D

### Option 1: Download Pre-built Binaries (Recommended)

1. Visit the [Releases page](https://github.com/daitamu/Sikuli-D/releases)
2. Download the appropriate package for your platform:
   - Windows: `sikuli-d-ide-windows-x64.exe`
   - macOS: `sikuli-d-ide-macos-universal.dmg`
   - Linux: `sikuli-d-ide-linux-x64.AppImage`
3. Install or run the application

### Option 2: Install Python Runtime Only

If you only need to run Sikuli-D scripts without the IDE:

```bash
pip install sikulid
```

### Option 3: Build from Source

For developers who want to build from source:

```bash
# Clone the repository
git clone https://github.com/daitamu/Sikuli-D.git
cd Sikuli-D

# Build core library
cd core-rs
cargo build --release

# Build IDE
cd ../ide-rs-tauri
cargo tauri build

# Build Python runtime
cd ../runtime-rs
pip install maturin
maturin build --release
```

## Installing Tesseract OCR

Tesseract is required for OCR (text recognition) features.

### Windows

Download and install from [GitHub Releases](https://github.com/UB-Mannheim/tesseract/wiki):

```powershell
# Or use Chocolatey
choco install tesseract
```

### macOS

```bash
brew install tesseract
```

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install tesseract-ocr
```

## Verifying Installation

### Verify IDE Installation

Launch the Sikuli-D IDE application. You should see the main window with an editor pane.

### Verify Python Runtime

```python
from sikulid import *

# Test screen capture
print(Screen())
# Output: Screen(0)[0,0 1920x1080]

# Test OCR
text = Screen().text()
print("OCR is working!")
```

### Verify Tesseract

```bash
tesseract --version
# Should output version 5.x or later
```

## Troubleshooting

### Tesseract not found

If you get "Tesseract not found" errors:

1. Verify Tesseract is installed: `tesseract --version`
2. Add Tesseract to your PATH environment variable
3. On Windows, default installation path is: `C:\Program Files\Tesseract-OCR`

### Python import errors

If `from sikulid import *` fails:

1. Verify Python version: `python --version`
2. Reinstall the package: `pip install --force-reinstall sikulid`
3. Check pip installation directory: `pip show sikulid`

### IDE won't start

1. Check system requirements are met
2. Try running from terminal to see error messages
3. Check for conflicting applications (screen readers, accessibility tools)

## Next Steps

- [Quick Start Guide](./quick-start.md) - Your first Sikuli-D script
- [API Reference](/api/) - Complete API documentation
- [Tutorials](/tutorials/) - Step-by-step examples
