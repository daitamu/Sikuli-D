# Test Fixtures / テストフィクスチャ

This directory contains test fixtures for integration tests.
このディレクトリには統合テスト用のテストフィクスチャが含まれています。

## Directory Structure / ディレクトリ構造

```
fixtures/
├── images/          # Test images for pattern matching
│                    # パターンマッチング用テスト画像
│   └── (Add .png files as needed)
│       (必要に応じて.pngファイルを追加)
│
└── scripts/         # Test Python scripts
                     # テストPythonスクリプト
    ├── test_simple.py      # Simple hello world script
    ├── test_sikuli.py      # SikuliX API test script
    ├── test_error.py       # Error handling test
    └── test_japanese.py    # Japanese text test
```

## Usage / 使用方法

### Image Fixtures / 画像フィクスチャ

To add test images:
テスト画像を追加するには:

1. Create small PNG images (e.g., 50x50, 100x100)
   小さなPNG画像を作成（例: 50x50、100x100）

2. Save them in `images/` directory
   `images/`ディレクトリに保存

3. Reference them in tests: `tests/fixtures/images/your_image.png`
   テストで参照: `tests/fixtures/images/your_image.png`

Example images needed:
必要な画像例:
- `button_100x50.png` - Standard button
- `icon_32x32.png` - Small icon
- `text_sample_en.png` - English text for OCR
- `text_sample_ja.png` - Japanese text for OCR

### Script Fixtures / スクリプトフィクスチャ

Python scripts in `scripts/` are used for:
`scripts/`内のPythonスクリプトの用途:

- Testing Python version detection
  Pythonバージョン検出テスト
- Testing script execution
  スクリプト実行テスト
- Testing error handling
  エラーハンドリングテスト
- Testing Japanese text support
  日本語テキストサポートテスト

## Notes / 注意事項

- Keep fixture files small (< 100KB each)
  フィクスチャファイルは小さく保つ（各100KB未満）

- Use descriptive filenames
  わかりやすいファイル名を使用

- Document any special requirements in test comments
  特別な要件はテストコメントに記載

- Binary files (images) are not committed by default
  バイナリファイル（画像）はデフォルトでコミットされない
  Add them explicitly if needed for tests
  テストに必要な場合は明示的に追加
