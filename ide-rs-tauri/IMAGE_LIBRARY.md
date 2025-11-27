# Image Library Implementation / 画像ライブラリ実装

## Overview / 概要

The Image Library provides comprehensive image management functionality for `.sikuli` bundles, including:
画像ライブラリは `.sikuli` バンドル用の包括的な画像管理機能を提供します：

- **Image listing and metadata** / 画像一覧とメタデータ
- **Thumbnail generation** / サムネイル生成
- **Usage analysis** / 使用状況分析
- **Import/Export** / インポート/エクスポート
- **Unused image detection** / 未使用画像検出

---

## Architecture / アーキテクチャ

### Backend (Rust)

Located in: `src/image_library.rs`

#### Core Functions

1. **`list_images(project_path: &str)`**
   - Scans `.sikuli` bundle for image files
   - Extracts metadata (size, dimensions, creation date)
   - Analyzes script usage count
   - Returns `Vec<ImageInfo>`

2. **`get_image_thumbnail(path: &str, size: u32)`**
   - Loads image file
   - Resizes using Lanczos3 filter for quality
   - Encodes as PNG
   - Returns Base64 string

3. **`delete_image(path: &str)`**
   - Removes image file from filesystem
   - Returns `Result<(), String>`

4. **`rename_image(old_path: &str, new_name: &str)`**
   - Renames image within bundle
   - Validates target doesn't exist
   - Returns new path

5. **`find_unused_images(project_path: &str)`**
   - Scans all script files (.py, .js, .rb)
   - Counts image references
   - Returns list of images with 0 usage

6. **`import_images(paths: Vec<String>, project_path: &str)`**
   - Copies images to bundle
   - Returns list of successfully imported paths

#### Data Structure

```rust
pub struct ImageInfo {
    pub path: String,          // Full path
    pub name: String,          // Filename
    pub width: u32,            // Dimensions
    pub height: u32,
    pub file_size: u64,        // Bytes
    pub created_at: String,    // ISO 8601
    pub thumbnail: Option<String>, // Base64 PNG
    pub usage_count: u32,      // References in scripts
}
```

### Frontend (TypeScript/React)

Located in: `examples/ImageLibrary.tsx`

#### Features

1. **Grid View**
   - Thumbnail display
   - Image name and dimensions
   - Unused badge
   - Context menu (Insert, Rename, Delete)

2. **List View**
   - Tabular layout
   - Full metadata display
   - Action buttons

3. **Filtering**
   - Text search by name
   - Show unused only toggle
   - Sort by: name, size, date, usage

4. **Actions**
   - Import images
   - Delete images (with confirmation)
   - Rename images
   - Insert to editor

5. **Status Bar**
   - Total image count
   - Unused count
   - Total file size

---

## Usage / 使用方法

### Backend (Tauri Commands)

#### List Images

```rust
use tauri::command;

#[tauri::command]
async fn list_images_command(project_path: String) -> Result<Vec<ImageInfo>, String> {
    list_images(&project_path)
}
```

```typescript
const images = await invoke<ImageInfo[]>('list_images_command', {
  projectPath: '/path/to/project.sikuli'
});
```

#### Get Thumbnail

```typescript
const thumbnail = await invoke<string>('get_image_thumbnail_command', {
  path: '/path/to/image.png',
  size: 128
});

const imgSrc = `data:image/png;base64,${thumbnail}`;
```

#### Delete Image

```typescript
await invoke('delete_image_command', {
  path: '/path/to/image.png'
});
```

#### Rename Image

```typescript
const newPath = await invoke<string>('rename_image_command', {
  oldPath: '/path/to/old.png',
  newName: 'new.png'
});
```

#### Find Unused Images

```typescript
const unused = await invoke<string[]>('find_unused_images_command', {
  projectPath: '/path/to/project.sikuli'
});

console.log(`Found ${unused.length} unused images`);
```

#### Import Images

```typescript
const imported = await invoke<string[]>('import_images_command', {
  paths: ['/path/to/img1.png', '/path/to/img2.png'],
  projectPath: '/path/to/project.sikuli'
});
```

### Frontend Component

```tsx
import ImageLibrary from './examples/ImageLibrary';

function App() {
  const [projectPath, setProjectPath] = useState('/path/to/project.sikuli');

  const handleImageInsert = (imagePath: string) => {
    // Insert image reference into editor
    const fileName = imagePath.split('/').pop();
    editor.insert(`find("${fileName}")`);
  };

  return (
    <ImageLibrary
      projectPath={projectPath}
      onImageSelect={(path) => console.log('Selected:', path)}
      onImageInsert={handleImageInsert}
    />
  );
}
```

---

## Testing / テスト

### Backend Tests

Located in: `src/image_library.rs` (bottom of file)

Run tests:
```bash
cargo test --lib image_library
```

Tests cover:
- ✅ List images in bundle
- ✅ Count image usage in scripts
- ✅ Generate thumbnails
- ✅ Rename images
- ✅ Delete images
- ✅ Find unused images
- ✅ Import images

### Test Data Structure

Tests use `tempfile` crate to create temporary `.sikuli` bundles:

```rust
#[test]
fn test_list_images() {
    let (_temp, bundle_path) = create_test_bundle();
    let images = list_images(bundle_path.to_str().unwrap()).unwrap();

    assert_eq!(images.len(), 1);
    assert_eq!(images[0].name, "button.png");
    assert_eq!(images[0].width, 10);
    assert_eq!(images[0].height, 10);
}
```

---

## Performance Considerations / パフォーマンス考慮事項

### Thumbnail Caching

Thumbnails are generated on-demand and cached in the component state:

```typescript
useEffect(() => {
  const imagesToLoad = filteredImages
    .filter((img) => !img.thumbnail)
    .slice(0, 20); // Limit to 20 at a time

  imagesToLoad.forEach((img) => {
    loadThumbnail(img.path);
  });
}, [filteredImages, loadThumbnail]);
```

### Large Bundles

For projects with many images (100+):
- Use virtual scrolling (e.g., `react-window`)
- Lazy load thumbnails
- Implement pagination

### File System Operations

All file operations are asynchronous to prevent UI blocking:

```rust
#[tauri::command]
async fn list_images_command(project_path: String) -> Result<Vec<ImageInfo>, String> {
    // Runs on Tauri's async runtime
    tokio::task::spawn_blocking(move || {
        list_images(&project_path)
    }).await.unwrap()
}
```

---

## Security / セキュリティ

### Path Validation

All file operations validate paths to prevent directory traversal:

```rust
pub fn delete_image(path: &str) -> Result<(), String> {
    let image_path = Path::new(path);

    if !image_path.exists() {
        return Err(format!("Image does not exist: {}", path));
    }

    // Additional validation: ensure path is within bundle
    // ...
}
```

### User Confirmation

Destructive operations require user confirmation:

```typescript
const handleDelete = async (imagePath: string) => {
  if (!confirm(`Delete ${imagePath.split(/[\\/]/).pop()}?`)) {
    return;
  }

  await invoke('delete_image_command', { path: imagePath });
};
```

---

## Future Enhancements / 将来の拡張

### Planned Features

1. **Image Preview Dialog**
   - Full-size display
   - Metadata panel
   - Usage location list
   - Pattern editor integration

2. **Drag & Drop Import**
   - Drop files onto library
   - Automatic import

3. **Image Comparison**
   - Side-by-side comparison
   - Similarity detection
   - Duplicate finder

4. **Batch Operations**
   - Multi-select
   - Bulk delete
   - Bulk rename

5. **Image Optimization**
   - PNG compression
   - Format conversion
   - Resize batch

6. **Search Enhancements**
   - Regex support
   - Filter by size range
   - Filter by date range

---

## Integration with Pattern Editor / パターンエディタ統合

The image library integrates with the pattern editor (Task 3-3D):

```typescript
// When user selects an image in library
const handleImageSelect = async (imagePath: string) => {
  // Load image into pattern editor
  await invoke('create_pattern', {
    imagePath,
    similarity: 0.7,
    targetOffset: { x: 0, y: 0 }
  });

  // Switch to pattern editor tab
  setActiveTab('pattern-editor');
};
```

---

## Dependencies / 依存関係

### Rust Crates

```toml
[dependencies]
image = "0.25"        # Image loading and manipulation
base64 = "0.22"       # Thumbnail encoding
serde = "1"           # Serialization
log = "0.4"           # Logging
chrono = "0.4"        # Date/time formatting

[dev-dependencies]
tempfile = "3"        # Temporary directories for tests
```

### TypeScript/React

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "react": "^18.0.0"
  }
}
```

---

## Troubleshooting / トラブルシューティング

### Common Issues

#### 1. Thumbnails not loading

**Problem:** Thumbnails show placeholder icon
**Solution:**
- Check image file exists
- Verify image format (PNG, JPG, BMP)
- Check console for errors

#### 2. Usage count is 0 for all images

**Problem:** All images show as unused
**Solution:**
- Ensure script files (.py, .js, .rb) exist in bundle
- Check script encoding (UTF-8)
- Verify image references match exact filename

#### 3. Import fails

**Problem:** Images don't appear after import
**Solution:**
- Check source files exist
- Verify write permissions on bundle directory
- Ensure filenames are valid (no special characters)

---

## API Reference / APIリファレンス

### Complete Type Definitions

See: `types/image-library.d.ts`

### Command Summary

| Command | Parameters | Returns | Description |
|---------|-----------|---------|-------------|
| `list_images_command` | `projectPath: string` | `ImageInfo[]` | List all images in bundle |
| `get_image_thumbnail_command` | `path: string, size: number` | `string` | Get Base64 thumbnail |
| `delete_image_command` | `path: string` | `void` | Delete image file |
| `rename_image_command` | `oldPath: string, newName: string` | `string` | Rename image, return new path |
| `find_unused_images_command` | `projectPath: string` | `string[]` | Find unused images |
| `import_images_command` | `paths: string[], projectPath: string` | `string[]` | Import images to bundle |

---

## License / ライセンス

MIT License - See LICENSE file for details

---

## Contributors / 貢献者

- Sikuli-D Team
- Generated with Claude Code

---

**Last Updated / 最終更新**: 2025-11-27
**Version / バージョン**: 0.1.0
