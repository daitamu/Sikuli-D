//! SikuliX bundle (.sikuli) handling
//! SikuliX バンドル (.sikuli) 処理

use std::path::{Path, PathBuf};
use anyhow::{Result, bail, Context};

/// SikuliX Bundle structure
/// SikuliX バンドル構造
///
/// A .sikuli bundle is a directory containing:
/// - A Python script (same name as directory or *.py)
/// - Image files used by the script
///
/// .sikuli バンドルは以下を含むディレクトリ:
/// - Pythonスクリプト (ディレクトリと同名または *.py)
/// - スクリプトが使用する画像ファイル
#[derive(Debug)]
#[allow(dead_code)]
pub struct Bundle {
    /// Bundle directory path / バンドルディレクトリパス
    pub path: PathBuf,
    /// Main script file / メインスクリプトファイル
    pub main_script: PathBuf,
    /// Image files in bundle / バンドル内の画像ファイル
    pub images: Vec<PathBuf>,
}

#[allow(dead_code)]
impl Bundle {
    /// Load a bundle from directory
    /// ディレクトリからバンドルを読み込む
    pub fn load(path: &Path) -> Result<Self> {
        if !path.is_dir() {
            bail!("Bundle path is not a directory: {}", path.display());
        }

        let main_script = find_main_script(path)?;
        let images = find_images(path)?;

        Ok(Bundle {
            path: path.to_path_buf(),
            main_script,
            images,
        })
    }

    /// Get the base name of the bundle
    /// バンドルのベース名を取得
    pub fn name(&self) -> Option<&str> {
        self.path.file_stem()?.to_str()
    }
}

/// Find the main script in a bundle
/// バンドル内のメインスクリプトを検索
pub fn find_main_script(bundle_path: &Path) -> Result<PathBuf> {
    // First, try to find script with same name as bundle
    // まず、バンドルと同名のスクリプトを検索
    if let Some(bundle_name) = bundle_path.file_stem() {
        let script_name = format!("{}.py", bundle_name.to_string_lossy());
        let script_path = bundle_path.join(&script_name);
        if script_path.exists() {
            return Ok(script_path);
        }
    }

    // Otherwise, find first .py file
    // なければ、最初の .py ファイルを検索
    for entry in std::fs::read_dir(bundle_path)
        .context("Failed to read bundle directory")?
    {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "py").unwrap_or(false) {
            return Ok(path);
        }
    }

    bail!("No Python script found in bundle: {}", bundle_path.display())
}

/// Find all image files in a bundle
/// バンドル内の全画像ファイルを検索
#[allow(dead_code)]
pub fn find_images(bundle_path: &Path) -> Result<Vec<PathBuf>> {
    let mut images = Vec::new();
    let image_extensions = ["png", "jpg", "jpeg", "gif", "bmp"];

    for entry in std::fs::read_dir(bundle_path)
        .context("Failed to read bundle directory")?
    {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if image_extensions.contains(&ext_lower.as_str()) {
                images.push(path);
            }
        }
    }

    Ok(images)
}

/// Resolve image path relative to bundle or script
/// バンドルまたはスクリプトからの相対パスで画像を解決
#[allow(dead_code)]
pub fn resolve_image_path(image_name: &str, script_path: &Path) -> Option<PathBuf> {
    // Try relative to script directory
    // スクリプトディレクトリからの相対パスを試行
    if let Some(script_dir) = script_path.parent() {
        let relative_path = script_dir.join(image_name);
        if relative_path.exists() {
            return Some(relative_path);
        }

        // Try in parent .sikuli directory
        // 親の .sikuli ディレクトリを試行
        if let Some(parent) = script_dir.parent() {
            if parent.extension().map(|e| e == "sikuli").unwrap_or(false) {
                let bundle_path = parent.join(image_name);
                if bundle_path.exists() {
                    return Some(bundle_path);
                }
            }
        }
    }

    // Try as absolute path
    // 絶対パスとして試行
    let absolute_path = PathBuf::from(image_name);
    if absolute_path.exists() {
        return Some(absolute_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_find_main_script_same_name() {
        let temp = tempdir().unwrap();
        let bundle = temp.path().join("test.sikuli");
        fs::create_dir(&bundle).unwrap();
        fs::write(bundle.join("test.py"), "print('hello')").unwrap();

        let result = find_main_script(&bundle).unwrap();
        assert!(result.ends_with("test.py"));
    }

    #[test]
    fn test_find_main_script_any_py() {
        let temp = tempdir().unwrap();
        let bundle = temp.path().join("test.sikuli");
        fs::create_dir(&bundle).unwrap();
        fs::write(bundle.join("main.py"), "print('hello')").unwrap();

        let result = find_main_script(&bundle).unwrap();
        // PathBuf::ends_with checks path components, not string suffix
        assert_eq!(result.extension().and_then(|e| e.to_str()), Some("py"));
    }

    #[test]
    fn test_find_images() {
        let temp = tempdir().unwrap();
        let bundle = temp.path().join("test.sikuli");
        fs::create_dir(&bundle).unwrap();
        fs::write(bundle.join("button.png"), &[0u8; 10]).unwrap();
        fs::write(bundle.join("icon.jpg"), &[0u8; 10]).unwrap();
        fs::write(bundle.join("script.py"), "").unwrap();

        let images = find_images(&bundle).unwrap();
        assert_eq!(images.len(), 2);
    }
}
