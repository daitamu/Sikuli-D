//! Image Library Module / 画像ライブラリモジュール
//!
//! Manages images within .sikuli bundles, providing:
//! .sikuliバンドル内の画像を管理し、以下を提供します：
//!
//! - Image listing and metadata extraction / 画像一覧とメタデータ抽出
//! - Thumbnail generation / サムネイル生成
//! - Image import/export / 画像のインポート/エクスポート
//! - Usage analysis / 使用状況分析
//! - Unused image detection / 未使用画像検出

use image::imageops::FilterType;
use image::{GenericImageView, ImageFormat};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::Path;

// ============================================================================
// Data Structures / データ構造
// ============================================================================

/// Image metadata and information
/// 画像メタデータと情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Full file path / 完全ファイルパス
    pub path: String,

    /// File name only / ファイル名のみ
    pub name: String,

    /// Image width in pixels / 画像幅（ピクセル）
    pub width: u32,

    /// Image height in pixels / 画像高さ（ピクセル）
    pub height: u32,

    /// File size in bytes / ファイルサイズ（バイト）
    pub file_size: u64,

    /// Creation timestamp (ISO 8601) / 作成タイムスタンプ（ISO 8601）
    pub created_at: String,

    /// Base64-encoded thumbnail (optional) / Base64エンコードされたサムネイル（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,

    /// Number of times referenced in scripts / スクリプト内での参照回数
    pub usage_count: u32,
}

/// Supported image file extensions
/// サポートされる画像ファイル拡張子
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp"];

// ============================================================================
// Core Functions / コア関数
// ============================================================================

/// List all images in a project bundle
/// プロジェクトバンドル内のすべての画像をリストします
///
/// # Arguments / 引数
/// * `project_path` - Path to .sikuli bundle / .sikuliバンドルへのパス
///
/// # Returns / 戻り値
/// Vector of ImageInfo structures / ImageInfo構造体のベクター
pub fn list_images(project_path: &str) -> Result<Vec<ImageInfo>, String> {
    let bundle_path = Path::new(project_path);

    if !bundle_path.exists() {
        return Err(format!("Project path does not exist: {}", project_path));
    }

    if !bundle_path.is_dir() {
        return Err(format!("Project path is not a directory: {}", project_path));
    }

    debug!("Scanning images in bundle: {:?}", bundle_path);

    let mut images = Vec::new();

    // Read directory entries / ディレクトリエントリを読み取る
    let entries = fs::read_dir(bundle_path).map_err(|e| {
        error!("Failed to read directory: {}", e);
        format!("Failed to read directory: {}", e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        // Check if it's an image file / 画像ファイルかチェック
        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        if let Some(ext) = extension {
            if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                match get_image_info(&path, project_path) {
                    Ok(info) => images.push(info),
                    Err(e) => {
                        warn!("Failed to get info for {:?}: {}", path, e);
                        continue;
                    }
                }
            }
        }
    }

    info!("Found {} images in bundle", images.len());
    Ok(images)
}

/// Get detailed information about a single image
/// 単一画像の詳細情報を取得します
fn get_image_info(image_path: &Path, project_path: &str) -> Result<ImageInfo, String> {
    let metadata =
        fs::metadata(image_path).map_err(|e| format!("Failed to read metadata: {}", e))?;

    let file_size = metadata.len();

    // Get creation time / 作成時刻を取得
    let created_at = metadata
        .created()
        .or_else(|_| metadata.modified())
        .map(|time| {
            let datetime: chrono::DateTime<chrono::Utc> = time.into();
            datetime.to_rfc3339()
        })
        .unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());

    // Load image to get dimensions / 画像を読み込んで寸法を取得
    let img = image::open(image_path).map_err(|e| format!("Failed to open image: {}", e))?;

    let (width, height) = img.dimensions();

    // Get usage count / 使用回数を取得
    let name = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    let usage_count = count_image_usage(project_path, &name)?;

    Ok(ImageInfo {
        path: image_path.to_string_lossy().to_string(),
        name,
        width,
        height,
        file_size,
        created_at,
        thumbnail: None,
        usage_count,
    })
}

/// Generate a thumbnail for an image
/// 画像のサムネイルを生成します
///
/// # Arguments / 引数
/// * `path` - Path to image file / 画像ファイルへのパス
/// * `size` - Maximum thumbnail size (width/height) / サムネイルの最大サイズ（幅/高さ）
///
/// # Returns / 戻り値
/// Base64-encoded PNG thumbnail / Base64エンコードされたPNGサムネイル
pub fn get_image_thumbnail(path: &str, size: u32) -> Result<String, String> {
    debug!("Generating thumbnail for: {}, size: {}", path, size);

    let img = image::open(path).map_err(|e| {
        error!("Failed to open image {}: {}", path, e);
        format!("Failed to open image: {}", e)
    })?;

    // Resize to thumbnail / サムネイルにリサイズ
    let thumbnail = img.resize(size, size, FilterType::Lanczos3);

    // Encode as PNG to buffer / PNGとしてバッファにエンコード
    let mut buffer = Cursor::new(Vec::new());
    thumbnail
        .write_to(&mut buffer, ImageFormat::Png)
        .map_err(|e| {
            error!("Failed to encode thumbnail: {}", e);
            format!("Failed to encode thumbnail: {}", e)
        })?;

    // Convert to Base64 / Base64に変換
    let base64 = base64_encode(buffer.get_ref());

    debug!(
        "Thumbnail generated successfully, size: {} bytes",
        base64.len()
    );
    Ok(base64)
}

/// Delete an image file
/// 画像ファイルを削除します
///
/// # Arguments / 引数
/// * `path` - Path to image file to delete / 削除する画像ファイルへのパス
pub fn delete_image(path: &str) -> Result<(), String> {
    info!("Deleting image: {}", path);

    let image_path = Path::new(path);

    if !image_path.exists() {
        return Err(format!("Image does not exist: {}", path));
    }

    fs::remove_file(image_path).map_err(|e| {
        error!("Failed to delete image: {}", e);
        format!("Failed to delete image: {}", e)
    })?;

    info!("Image deleted successfully");
    Ok(())
}

/// Rename an image file
/// 画像ファイルの名前を変更します
///
/// # Arguments / 引数
/// * `old_path` - Current image path / 現在の画像パス
/// * `new_name` - New file name (without directory) / 新しいファイル名（ディレクトリなし）
///
/// # Returns / 戻り値
/// New full path / 新しい完全パス
pub fn rename_image(old_path: &str, new_name: &str) -> Result<String, String> {
    info!("Renaming image: {} -> {}", old_path, new_name);

    let old_image_path = Path::new(old_path);

    if !old_image_path.exists() {
        return Err(format!("Image does not exist: {}", old_path));
    }

    let parent = old_image_path
        .parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;

    let new_path = parent.join(new_name);

    if new_path.exists() {
        return Err(format!("Target file already exists: {:?}", new_path));
    }

    fs::rename(old_image_path, &new_path).map_err(|e| {
        error!("Failed to rename image: {}", e);
        format!("Failed to rename image: {}", e)
    })?;

    let new_path_str = new_path.to_string_lossy().to_string();
    info!("Image renamed successfully to: {}", new_path_str);
    Ok(new_path_str)
}

/// Find unused images in a project
/// プロジェクト内の未使用画像を検索します
///
/// # Arguments / 引数
/// * `project_path` - Path to .sikuli bundle / .sikuliバンドルへのパス
///
/// # Returns / 戻り値
/// List of unused image paths / 未使用画像パスのリスト
pub fn find_unused_images(project_path: &str) -> Result<Vec<String>, String> {
    info!("Finding unused images in: {}", project_path);

    let images = list_images(project_path)?;

    let unused: Vec<String> = images
        .into_iter()
        .filter(|img| img.usage_count == 0)
        .map(|img| img.path)
        .collect();

    info!("Found {} unused images", unused.len());
    Ok(unused)
}

/// Import images to a project bundle
/// プロジェクトバンドルに画像をインポートします
///
/// # Arguments / 引数
/// * `paths` - List of image paths to import / インポートする画像パスのリスト
/// * `project_path` - Target .sikuli bundle path / ターゲット.sikuliバンドルパス
///
/// # Returns / 戻り値
/// List of imported image paths in bundle / バンドル内のインポートされた画像パスのリスト
pub fn import_images(paths: Vec<String>, project_path: &str) -> Result<Vec<String>, String> {
    info!("Importing {} images to: {}", paths.len(), project_path);

    let bundle_path = Path::new(project_path);

    if !bundle_path.exists() || !bundle_path.is_dir() {
        return Err(format!("Invalid project path: {}", project_path));
    }

    let mut imported = Vec::new();

    for source_path in paths {
        let source = Path::new(&source_path);

        if !source.exists() || !source.is_file() {
            warn!("Skipping invalid source: {}", source_path);
            continue;
        }

        let file_name = source
            .file_name()
            .ok_or_else(|| format!("Invalid file name: {}", source_path))?;

        let target = bundle_path.join(file_name);

        // Copy file / ファイルをコピー
        match fs::copy(&source, &target) {
            Ok(_) => {
                let target_str = target.to_string_lossy().to_string();
                info!("Imported: {}", target_str);
                imported.push(target_str);
            }
            Err(e) => {
                warn!("Failed to import {}: {}", source_path, e);
                continue;
            }
        }
    }

    info!("Successfully imported {} images", imported.len());
    Ok(imported)
}

// ============================================================================
// Helper Functions / ヘルパー関数
// ============================================================================

/// Count how many times an image is referenced in project scripts
/// プロジェクトスクリプト内で画像が参照されている回数をカウントします
fn count_image_usage(project_path: &str, image_name: &str) -> Result<u32, String> {
    let bundle_path = Path::new(project_path);
    let mut count = 0;

    // Find script files / スクリプトファイルを検索
    let script_extensions = ["py", "js", "rb"];

    let entries =
        fs::read_dir(bundle_path).map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase());

        if let Some(ext) = extension {
            if script_extensions.contains(&ext.as_str()) {
                // Read script content / スクリプト内容を読み取る
                if let Ok(content) = fs::read_to_string(&path) {
                    // Count occurrences / 出現回数をカウント
                    count += content.matches(image_name).count() as u32;
                }
            }
        }
    }

    Ok(count)
}

/// Base64 encode data
/// データをBase64エンコードします
fn base64_encode(data: &[u8]) -> String {
    use base64::{engine::general_purpose, Engine as _};
    general_purpose::STANDARD.encode(data)
}

// ============================================================================
// Tauri Commands / Tauriコマンド
// ============================================================================

/// Tauri command: List all images in a project
/// Tauriコマンド：プロジェクト内のすべての画像をリスト
#[tauri::command]
pub fn list_images_command(project_path: String) -> Result<Vec<ImageInfo>, String> {
    list_images(&project_path)
}

/// Tauri command: Get image thumbnail
/// Tauriコマンド：画像サムネイルを取得
#[tauri::command]
pub fn get_image_thumbnail_command(path: String, size: u32) -> Result<String, String> {
    get_image_thumbnail(&path, size)
}

/// Tauri command: Delete an image
/// Tauriコマンド：画像を削除
#[tauri::command]
pub fn delete_image_command(path: String) -> Result<(), String> {
    delete_image(&path)
}

/// Tauri command: Rename an image
/// Tauriコマンド：画像の名前を変更
#[tauri::command]
pub fn rename_image_command(old_path: String, new_name: String) -> Result<String, String> {
    rename_image(&old_path, &new_name)
}

/// Tauri command: Find unused images
/// Tauriコマンド：未使用画像を検索
#[tauri::command]
pub fn find_unused_images_command(project_path: String) -> Result<Vec<String>, String> {
    find_unused_images(&project_path)
}

/// Tauri command: Import images to project
/// Tauriコマンド：プロジェクトに画像をインポート
#[tauri::command]
pub fn import_images_command(
    paths: Vec<String>,
    project_path: String,
) -> Result<Vec<String>, String> {
    import_images(paths, &project_path)
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Create a test .sikuli bundle with images
    /// 画像付きテスト.sikuliバンドルを作成
    fn create_test_bundle() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("test.sikuli");
        fs::create_dir(&bundle_path).unwrap();

        // Create a simple 10x10 red PNG image
        let img = image::RgbImage::from_fn(10, 10, |_, _| image::Rgb([255, 0, 0]));
        img.save(bundle_path.join("button.png")).unwrap();

        // Create a Python script that uses the image
        let script_content = r#"
from sikuli import *
find("button.png")
click("button.png")
"#;
        let mut script_file = File::create(bundle_path.join("test.py")).unwrap();
        script_file.write_all(script_content.as_bytes()).unwrap();

        (temp_dir, bundle_path)
    }

    #[test]
    fn test_list_images() {
        let (_temp, bundle_path) = create_test_bundle();
        let bundle_str = bundle_path.to_str().unwrap();

        let images = list_images(bundle_str).unwrap();
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].name, "button.png");
        assert_eq!(images[0].width, 10);
        assert_eq!(images[0].height, 10);
    }

    #[test]
    fn test_image_usage_count() {
        let (_temp, bundle_path) = create_test_bundle();
        let bundle_str = bundle_path.to_str().unwrap();

        let images = list_images(bundle_str).unwrap();
        assert_eq!(images[0].usage_count, 2); // Referenced twice in script
    }

    #[test]
    fn test_get_thumbnail() {
        let (_temp, bundle_path) = create_test_bundle();
        let image_path = bundle_path.join("button.png");
        let image_str = image_path.to_str().unwrap();

        let thumbnail = get_image_thumbnail(image_str, 32).unwrap();
        assert!(!thumbnail.is_empty());
        assert!(thumbnail.starts_with("iVBOR")); // PNG Base64 signature
    }

    #[test]
    fn test_rename_image() {
        let (_temp, bundle_path) = create_test_bundle();
        let old_path = bundle_path.join("button.png");
        let old_str = old_path.to_str().unwrap();

        let new_path = rename_image(old_str, "new_button.png").unwrap();
        assert!(Path::new(&new_path).exists());
        assert!(!old_path.exists());
    }

    #[test]
    fn test_delete_image() {
        let (_temp, bundle_path) = create_test_bundle();
        let image_path = bundle_path.join("button.png");
        let image_str = image_path.to_str().unwrap();

        delete_image(image_str).unwrap();
        assert!(!image_path.exists());
    }

    #[test]
    fn test_find_unused_images() {
        let (_temp, bundle_path) = create_test_bundle();

        // Create an unused image
        let img = image::RgbImage::from_fn(10, 10, |_, _| image::Rgb([0, 255, 0]));
        img.save(bundle_path.join("unused.png")).unwrap();

        let bundle_str = bundle_path.to_str().unwrap();
        let unused = find_unused_images(bundle_str).unwrap();

        assert_eq!(unused.len(), 1);
        assert!(unused[0].contains("unused.png"));
    }

    #[test]
    fn test_import_images() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("target.sikuli");
        fs::create_dir(&bundle_path).unwrap();

        // Create source images
        let source_dir = temp_dir.path().join("source");
        fs::create_dir(&source_dir).unwrap();

        let img = image::RgbImage::from_fn(10, 10, |_, _| image::Rgb([0, 0, 255]));
        let source_path = source_dir.join("import_me.png");
        img.save(&source_path).unwrap();

        let bundle_str = bundle_path.to_str().unwrap();
        let source_str = source_path.to_str().unwrap().to_string();

        let imported = import_images(vec![source_str], bundle_str).unwrap();
        assert_eq!(imported.len(), 1);

        let target_file = bundle_path.join("import_me.png");
        assert!(target_file.exists());
    }
}
