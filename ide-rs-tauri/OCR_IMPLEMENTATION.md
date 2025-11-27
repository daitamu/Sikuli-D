# OCR Panel Implementation Documentation
# OCRパネル実装ドキュメント

**Version**: 1.0.0
**Date**: 2025-11-27
**Status**: Implemented / 実装完了

---

## Overview / 概要

This document describes the OCR (Optical Character Recognition) panel integration implemented for ide-rs-tauri.

このドキュメントは、ide-rs-tauriのために実装されたOCR（光学文字認識）パネル統合について説明します。

### Implementation Approach / 実装アプローチ

The OCR functionality is implemented by **calling the sikulix CLI** as a subprocess, following the design principle outlined in IDE-RS-TAURI-DESIGN.md:

OCR機能は、IDE-RS-TAURI-DESIGN.mdで概説された設計原則に従って、**sikulix CLIをサブプロセスとして呼び出す**ことで実装されています：

```
IDE (Tauri Backend) → sikulix CLI → core-rs (OCR execution)
```

This approach ensures:
- Single source of truth for OCR execution
- Consistent behavior between CLI and IDE
- Easier testing and maintenance

このアプローチは以下を保証します：
- OCR実行の単一真実源
- CLIとIDEの一貫した動作
- テストと保守の容易さ

---

## File Structure / ファイル構造

### Backend (Rust) / バックエンド（Rust）

```
ide-rs-tauri/src/
├── ocr.rs              # OCR module (NEW)
└── main.rs             # Updated with OCR command registration
```

#### ocr.rs Components / ocr.rsコンポーネント

```rust
// Data Types / データ型
- RegionDto: Region coordinates for OCR
- OcrOptions: Recognition options (language, confidence, PSM)
- OcrResult: Recognition result with text and confidence
- OcrWord: Individual word with bounding box
- OcrLanguage: Language information

// State Management / 状態管理
- OcrState: Manages current language and language cache

// Tauri Commands / Tauriコマンド
- ocr_recognize(region) -> OcrResult
- ocr_recognize_with_options(region, options) -> OcrResult
- get_available_languages() -> Vec<OcrLanguage>
- set_ocr_language(lang) -> Result<()>
- get_ocr_language() -> String
- check_ocr_available() -> bool
- get_ocr_info() -> HashMap<String, String>
```

### Frontend (TypeScript/React) - TO BE IMPLEMENTED / フロントエンド（TypeScript/React）- 実装予定

```
ide-rs-tauri/src/components/
└── OCR/
    ├── OcrPanel.tsx           # Main OCR panel component
    ├── RegionSelector.tsx     # Region selection UI
    ├── LanguageSelector.tsx   # Language dropdown
    └── OcrResultViewer.tsx    # Result display with word boxes
```

---

## Tauri Commands Reference / Tauriコマンドリファレンス

### 1. ocr_recognize

Basic OCR recognition with default options.

デフォルトオプションでの基本OCR認識。

**Signature:**
```rust
async fn ocr_recognize(region: RegionDto) -> Result<OcrResult, String>
```

**Parameters:**
```typescript
interface RegionDto {
    x: number;      // X coordinate
    y: number;      // Y coordinate
    width: number;  // Width in pixels
    height: number; // Height in pixels
}
```

**Returns:**
```typescript
interface OcrResult {
    text: string;           // Recognized text
    confidence: number;     // Overall confidence (0.0-1.0)
    words: OcrWord[];       // Individual words
}

interface OcrWord {
    text: string;
    region: RegionDto;
    confidence: number;
}
```

**Usage Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const region = { x: 100, y: 200, width: 400, height: 100 };
const result = await invoke<OcrResult>('ocr_recognize', { region });
console.log(`Recognized: ${result.text}`);
```

---

### 2. ocr_recognize_with_options

OCR recognition with custom options.

カスタムオプション付きOCR認識。

**Signature:**
```rust
async fn ocr_recognize_with_options(
    region: RegionDto,
    options: OcrOptions
) -> Result<OcrResult, String>
```

**Parameters:**
```typescript
interface OcrOptions {
    language: string;       // Language code (e.g., "eng", "jpn")
    min_confidence: number; // Minimum confidence threshold (0.0-1.0)
    psm?: number;           // Page Segmentation Mode (optional)
}
```

**Usage Example:**
```typescript
const region = { x: 100, y: 200, width: 400, height: 100 };
const options = {
    language: "jpn",
    min_confidence: 0.8,
    psm: 6  // Assume a single uniform block of text
};

const result = await invoke<OcrResult>(
    'ocr_recognize_with_options',
    { region, options }
);
```

---

### 3. get_available_languages

Get list of available OCR languages.

利用可能なOCR言語のリストを取得。

**Returns:**
```typescript
interface OcrLanguage {
    code: string;       // Language code (e.g., "eng", "jpn")
    name: string;       // Display name (e.g., "English", "Japanese")
    installed: boolean; // Whether language data is installed
}
```

**Usage Example:**
```typescript
const languages = await invoke<OcrLanguage[]>('get_available_languages');
languages.forEach(lang => {
    console.log(`${lang.name} (${lang.code}): ${lang.installed ? 'Installed' : 'Not installed'}`);
});
```

---

### 4. set_ocr_language / get_ocr_language

Set and get the current OCR language.

現在のOCR言語を設定・取得。

**Usage Example:**
```typescript
// Set language
await invoke('set_ocr_language', { lang: "jpn" });

// Get current language
const currentLang = await invoke<string>('get_ocr_language');
console.log(`Current language: ${currentLang}`);
```

---

### 5. check_ocr_available

Check if OCR functionality is available (i.e., sikulix CLI is installed).

OCR機能が利用可能か確認（すなわち、sikulix CLIがインストールされているか）。

**Usage Example:**
```typescript
const available = await invoke<boolean>('check_ocr_available');
if (!available) {
    alert("OCR is not available. Please install sikulix CLI.");
}
```

---

### 6. get_ocr_info

Get OCR engine information.

OCRエンジン情報を取得。

**Returns:**
```typescript
interface OcrInfo {
    [key: string]: string;
}
```

**Usage Example:**
```typescript
const info = await invoke<Record<string, string>>('get_ocr_info');
console.log(`Engine: ${info.engine}`);
console.log(`Version: ${info.version}`);
```

---

## Frontend Implementation Guide / フロントエンド実装ガイド

### OCR Panel Component Example / OCRパネルコンポーネント例

```tsx
import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface OcrPanelProps {
    captureImageUrl: string;  // URL of captured screen image
}

export function OcrPanel({ captureImageUrl }: OcrPanelProps) {
    const [languages, setLanguages] = useState<OcrLanguage[]>([]);
    const [selectedLang, setSelectedLang] = useState("eng");
    const [selectedRegion, setSelectedRegion] = useState<RegionDto | null>(null);
    const [ocrResult, setOcrResult] = useState<OcrResult | null>(null);
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        loadLanguages();
    }, []);

    const loadLanguages = async () => {
        const langs = await invoke<OcrLanguage[]>('get_available_languages');
        setLanguages(langs);
    };

    const handleRecognize = async () => {
        if (!selectedRegion) {
            alert("Please select a region");
            return;
        }

        setLoading(true);
        try {
            const options: OcrOptions = {
                language: selectedLang,
                min_confidence: 0.7
            };

            const result = await invoke<OcrResult>(
                'ocr_recognize_with_options',
                { region: selectedRegion, options }
            );

            setOcrResult(result);
        } catch (error) {
            console.error("OCR failed:", error);
            alert(`OCR failed: ${error}`);
        } finally {
            setLoading(false);
        }
    };

    const handleInsertToEditor = () => {
        if (ocrResult) {
            // TODO: Implement editor insertion
            console.log("Insert to editor:", ocrResult.text);
        }
    };

    return (
        <div className="ocr-panel">
            <div className="ocr-header">
                <h3>OCR Text Recognition</h3>
            </div>

            {/* Language Selector */}
            <div className="language-selector">
                <label>Language:</label>
                <select
                    value={selectedLang}
                    onChange={(e) => setSelectedLang(e.target.value)}
                >
                    {languages.map(lang => (
                        <option key={lang.code} value={lang.code}>
                            {lang.name}
                        </option>
                    ))}
                </select>
            </div>

            {/* Region Selector */}
            <div className="region-selector">
                <img src={captureImageUrl} alt="Screen capture" />
                {/* TODO: Implement drag-to-select region */}
            </div>

            {/* Recognize Button */}
            <button
                onClick={handleRecognize}
                disabled={!selectedRegion || loading}
            >
                {loading ? "Recognizing..." : "Recognize Text"}
            </button>

            {/* Result Display */}
            {ocrResult && (
                <div className="ocr-result">
                    <h4>Result:</h4>
                    <div className="result-text">{ocrResult.text}</div>
                    <div className="result-confidence">
                        Confidence: {(ocrResult.confidence * 100).toFixed(1)}%
                    </div>

                    {/* Word Boxes */}
                    <div className="word-boxes">
                        {ocrResult.words.map((word, idx) => (
                            <div key={idx} className="word-box">
                                <span>{word.text}</span>
                                <span className="word-confidence">
                                    {(word.confidence * 100).toFixed(0)}%
                                </span>
                            </div>
                        ))}
                    </div>

                    {/* Actions */}
                    <div className="result-actions">
                        <button onClick={handleInsertToEditor}>
                            Insert to Editor
                        </button>
                        <button onClick={() => navigator.clipboard.writeText(ocrResult.text)}>
                            Copy Text
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
}
```

---

## CLI Command Format / CLIコマンド形式

The OCR module calls the sikulix CLI with the following format:

OCRモジュールは以下の形式でsikulix CLIを呼び出します：

```bash
sikulix ocr \
    --region "x,y,width,height" \
    --language "eng" \
    --min-confidence 0.7 \
    --output json
```

**Optional parameters:**
- `--psm <mode>`: Page Segmentation Mode

**Expected JSON output:**
```json
{
    "text": "Recognized text here",
    "confidence": 0.95,
    "words": [
        {
            "text": "Recognized",
            "region": {"x": 100, "y": 200, "width": 80, "height": 20},
            "confidence": 0.96
        },
        {
            "text": "text",
            "region": {"x": 190, "y": 200, "width": 40, "height": 20},
            "confidence": 0.94
        }
    ]
}
```

---

## Supported Languages / サポートされる言語

Currently, the following languages are defined in the module:

現在、モジュールには以下の言語が定義されています：

| Code | Language | 日本語名 |
|------|----------|----------|
| eng | English | 英語 |
| jpn | Japanese | 日本語 |
| chi_sim | Chinese (Simplified) | 簡体字中国語 |
| chi_tra | Chinese (Traditional) | 繁体字中国語 |
| kor | Korean | 韓国語 |
| fra | French | フランス語 |
| deu | German | ドイツ語 |
| spa | Spanish | スペイン語 |

**Note:** Language availability depends on Tesseract language data installation.

**注意：** 言語の可用性はTesseract言語データのインストールに依存します。

---

## Error Handling / エラーハンドリング

### Error Cases / エラーケース

1. **sikulix CLI not found**
   ```
   Error: "sikulix CLI not found. Please ensure it's installed and in PATH."
   ```

2. **OCR command failed**
   ```
   Error: "OCR command failed: <stderr output>"
   ```

3. **Invalid language code**
   ```
   Error: "Unknown language code: xyz"
   ```

4. **Parse error**
   ```
   Error: "Failed to parse OCR output: <details>"
   ```

### Frontend Error Handling Example / フロントエンドエラーハンドリング例

```typescript
try {
    const result = await invoke<OcrResult>('ocr_recognize', { region });
    // Handle success
} catch (error) {
    if (error.includes("not found")) {
        alert("Please install sikulix CLI to use OCR functionality.");
    } else if (error.includes("command failed")) {
        alert("OCR recognition failed. Please try a different region.");
    } else {
        alert(`Unexpected error: ${error}`);
    }
}
```

---

## Testing / テスト

### Unit Tests / ユニットテスト

Unit tests are included in `ocr.rs`:

ユニットテストは`ocr.rs`に含まれています：

```bash
cargo test --lib ocr
```

**Test coverage:**
- ✅ RegionDto serialization
- ✅ OcrOptions default values
- ✅ OcrResult serialization
- ✅ JSON parsing
- ✅ Plain text fallback parsing
- ✅ Known languages list
- ✅ OcrState initialization

### Integration Testing / 統合テスト

To test OCR functionality end-to-end:

OCR機能をエンドツーエンドでテストするには：

1. Ensure sikulix CLI is installed
2. Run the IDE: `cargo tauri dev`
3. Open OCR panel
4. Select a screen region with text
5. Click "Recognize Text"
6. Verify results are displayed correctly

---

## Future Enhancements / 将来の拡張

### High Priority / 高優先度

1. **Frontend UI Implementation / フロントエンドUI実装**
   - Region selection with drag-to-select
   - Visual word bounding boxes overlay
   - Real-time preview

2. **Language Data Management / 言語データ管理**
   - Query actual installed Tesseract languages
   - Download/install language data from UI

3. **Result Export / 結果エクスポート**
   - Export to JSON/CSV
   - Save annotated image with bounding boxes

### Medium Priority / 中優先度

4. **Batch OCR / バッチOCR**
   - Process multiple regions at once
   - Save multiple results

5. **OCR History / OCR履歴**
   - Store recent OCR results
   - Search through history

6. **Advanced Options / 高度なオプション**
   - Custom PSM modes UI
   - Whitelist/blacklist characters
   - Custom confidence thresholds per word

### Low Priority / 低優先度

7. **Performance Optimization / パフォーマンス最適化**
   - Cache language models
   - Parallel batch processing

8. **OCR Profiles / OCRプロファイル**
   - Save/load OCR configurations
   - Different profiles for different use cases

---

## Dependencies / 依存関係

### Rust Dependencies / Rust依存関係

```toml
[dependencies]
tauri = "2.x"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
```

### External Dependencies / 外部依存関係

- **sikulix CLI** - Must be installed and available in PATH
- **Tesseract OCR** - Required by sikulix for OCR functionality
- **Tesseract language data** - For each language to be recognized

---

## Troubleshooting / トラブルシューティング

### Issue: "sikulix CLI not found"

**Solution:**
1. Install sikulix CLI
2. Add to system PATH
3. Verify: `sikulix --version`

### Issue: "OCR returns empty text"

**Possible causes:**
- Low image quality
- Wrong language selected
- Text too small or distorted

**Solutions:**
- Increase capture region size
- Ensure correct language is selected
- Try adjusting min_confidence threshold

### Issue: "Language not available"

**Solution:**
1. Install Tesseract language data
2. Verify installation: `tesseract --list-langs`
3. Restart IDE

---

## References / 参考資料

- **Design Document**: `IDE-RS-TAURI-DESIGN.md`
- **API Specification**: `L1-L2-API-SPEC.md`
- **Tesseract Documentation**: https://tesseract-ocr.github.io/
- **Tauri Documentation**: https://tauri.app/

---

## Changelog / 変更履歴

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-27 | Initial OCR module implementation / 初期OCRモジュール実装 |

---

**END OF DOCUMENT / ドキュメント終了**
