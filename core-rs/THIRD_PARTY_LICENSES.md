# Third-Party Licenses / サードパーティライセンス

This document lists the licenses of third-party dependencies used in sikulix-core.

このドキュメントは、sikulix-coreで使用しているサードパーティ依存ライブラリのライセンス一覧です。

---

## Core Dependencies / コア依存ライブラリ

### image
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/image-rs/image
- **Description**: Image processing library for Rust

### imageproc
- **License**: MIT
- **Repository**: https://github.com/image-rs/imageproc
- **Description**: Image processing operations

### serde
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/serde-rs/serde
- **Description**: Serialization framework

### serde_json
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/serde-rs/json
- **Description**: JSON serialization

### thiserror
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/dtolnay/thiserror
- **Description**: Derive macro for Error trait

### anyhow
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/dtolnay/anyhow
- **Description**: Flexible error handling

### uuid
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/uuid-rs/uuid
- **Description**: UUID generation and parsing

### chrono
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/chronotope/chrono
- **Description**: Date and time library

### tracing
- **License**: MIT
- **Repository**: https://github.com/tokio-rs/tracing
- **Description**: Application-level tracing

---

## Platform-Specific Dependencies / プラットフォーム固有依存ライブラリ

### Windows

#### windows
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/microsoft/windows-rs
- **Description**: Windows API bindings for Rust

### macOS

#### core-graphics
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/servo/core-foundation-rs
- **Description**: macOS Core Graphics bindings

#### core-foundation
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/servo/core-foundation-rs
- **Description**: macOS Core Foundation bindings

### Linux

#### x11rb
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/psychon/x11rb
- **Description**: X11 protocol bindings

---

## Optional Dependencies / オプション依存ライブラリ

### leptess (OCR feature)
- **License**: MIT
- **Repository**: https://github.com/houqp/leptess
- **Description**: Tesseract OCR bindings

### pyo3 (Python feature)
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/PyO3/pyo3
- **Description**: Python bindings for Rust

### tokio (async feature)
- **License**: MIT
- **Repository**: https://github.com/tokio-rs/tokio
- **Description**: Async runtime

---

## Development Dependencies / 開発用依存ライブラリ

### tempfile
- **License**: MIT / Apache-2.0
- **Repository**: https://github.com/Stebalien/tempfile
- **Description**: Temporary file management

---

## License Summary / ライセンスサマリー

All dependencies are licensed under permissive open-source licenses (MIT and/or Apache-2.0), which are compatible with the MIT license of this project.

全ての依存ライブラリは、本プロジェクトのMITライセンスと互換性のある寛容なオープンソースライセンス（MIT および/または Apache-2.0）の下でライセンスされています。

---

## Full License Texts / ライセンス全文

### MIT License

```
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Apache License 2.0

The Apache License 2.0 full text can be found at:
https://www.apache.org/licenses/LICENSE-2.0

Apache License 2.0の全文は上記URLを参照してください。
