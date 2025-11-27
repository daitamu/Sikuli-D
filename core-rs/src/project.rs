//! SikuliX Project Management
//! SikuliX プロジェクト管理
//!
//! Handles .sikuli project directory structure and management.
//! .sikuliプロジェクトのディレクトリ構造と管理を担当。

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::SikulixError;

/// Project settings stored in project.json
/// project.json に保存されるプロジェクト設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// Default similarity threshold for image matching / 画像マッチングのデフォルト類似度閾値
    #[serde(default = "default_similarity")]
    pub similarity: f64,
    /// OCR language settings / OCR言語設定
    #[serde(default)]
    pub ocr_language: String,
    /// Auto save interval in seconds (0 = disabled) / 自動保存間隔（秒、0=無効）
    #[serde(default)]
    pub auto_save_interval: u32,
}

fn default_similarity() -> f64 {
    0.7
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            similarity: default_similarity(),
            ocr_language: String::new(),
            auto_save_interval: 0,
        }
    }
}

/// Image asset in a project
/// プロジェクト内の画像アセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAsset {
    /// File name of the image / 画像のファイル名
    pub name: String,
    /// Relative path from project root / プロジェクトルートからの相対パス
    pub path: String,
    /// Optional description / オプションの説明
    #[serde(default)]
    pub description: String,
    /// Default similarity for this image / この画像のデフォルト類似度
    #[serde(default = "default_similarity")]
    pub similarity: f64,
}

/// SikuliX Project structure
/// SikuliX プロジェクト構造
///
/// Represents a .sikuli project directory with the following structure:
/// 以下の構造を持つ.sikuliプロジェクトディレクトリを表します:
///
/// ```text
/// project.sikuli/
/// ├── project.py        # Main script / メインスクリプト
/// ├── project.json      # Project settings / プロジェクト設定
/// └── images/           # Captured images / キャプチャ画像
///     ├── button1.png
///     └── icon2.png
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Project name / プロジェクト名
    pub name: String,
    /// Project version / プロジェクトバージョン
    #[serde(default = "default_version")]
    pub version: String,
    /// Absolute path to project directory / プロジェクトディレクトリの絶対パス
    #[serde(skip)]
    pub path: PathBuf,
    /// Main script file name / メインスクリプトファイル名
    #[serde(default = "default_main_script")]
    pub main_script: String,
    /// Project settings / プロジェクト設定
    #[serde(default)]
    pub settings: ProjectSettings,
    /// Image assets / 画像アセット
    #[serde(default)]
    pub images: Vec<ImageAsset>,
}

fn default_version() -> String {
    "1.0.0".to_string()
}

fn default_main_script() -> String {
    "main.py".to_string()
}

impl Default for Project {
    fn default() -> Self {
        Self {
            name: "Untitled".to_string(),
            version: default_version(),
            path: PathBuf::new(),
            main_script: default_main_script(),
            settings: ProjectSettings::default(),
            images: Vec::new(),
        }
    }
}

impl Project {
    /// Create a new project with the given name
    /// 指定された名前で新しいプロジェクトを作成
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Load a project from a .sikuli directory
    /// .sikuliディレクトリからプロジェクトを読み込み
    ///
    /// # Arguments
    /// * `path` - Path to the .sikuli directory / .sikuliディレクトリへのパス
    ///
    /// # Returns
    /// * `Result<Project>` - The loaded project or an error
    pub fn load(path: &Path) -> Result<Self, SikulixError> {
        // Validate path exists and is a directory
        if !path.exists() {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Project not found: {}", path.display()),
            )));
        }

        if !path.is_dir() {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Not a directory: {}", path.display()),
            )));
        }

        // Try to load project.json
        let config_path = path.join("project.json");
        let mut project = if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).map_err(|e| {
                SikulixError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Failed to parse project.json: {}", e),
                ))
            })?
        } else {
            // Create default project from directory name
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string();
            Project::new(&name)
        };

        project.path = path.to_path_buf();

        // Scan for images
        project.scan_images()?;

        Ok(project)
    }

    /// Save the project to its directory
    /// プロジェクトをディレクトリに保存
    pub fn save(&self) -> Result<(), SikulixError> {
        if self.path.as_os_str().is_empty() {
            return Err(SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Project path is not set",
            )));
        }

        // Create directory if it doesn't exist
        if !self.path.exists() {
            fs::create_dir_all(&self.path)?;
        }

        // Create images directory
        let images_dir = self.path.join("images");
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)?;
        }

        // Save project.json
        let config_path = self.path.join("project.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            SikulixError::IoError(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize project: {}", e),
            ))
        })?;
        fs::write(&config_path, content)?;

        Ok(())
    }

    /// Create a new project in the specified directory
    /// 指定されたディレクトリに新しいプロジェクトを作成
    pub fn create(path: &Path, name: &str) -> Result<Self, SikulixError> {
        let mut project = Project::new(name);
        project.path = path.to_path_buf();

        // Create the directory structure
        fs::create_dir_all(&project.path)?;
        fs::create_dir_all(project.path.join("images"))?;

        // Create empty main script
        let script_path = project.path.join(&project.main_script);
        fs::write(
            &script_path,
            format!(
                "# {}\n# Created by SikuliX IDE\n\nfrom sikuli import *\n\n# Your code here\n",
                name
            ),
        )?;

        // Save project config
        project.save()?;

        Ok(project)
    }

    /// Get the path to the main script
    /// メインスクリプトへのパスを取得
    pub fn main_script_path(&self) -> PathBuf {
        self.path.join(&self.main_script)
    }

    /// Get the path to the images directory
    /// 画像ディレクトリへのパスを取得
    pub fn images_dir(&self) -> PathBuf {
        self.path.join("images")
    }

    /// Scan the images directory and update the images list
    /// 画像ディレクトリをスキャンして画像リストを更新
    pub fn scan_images(&mut self) -> Result<(), SikulixError> {
        let images_dir = self.images_dir();
        if !images_dir.exists() {
            return Ok(());
        }

        self.images.clear();

        for entry in fs::read_dir(&images_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ext == "png" || ext == "jpg" || ext == "jpeg" {
                        if let Some(name) = path.file_name() {
                            self.images.push(ImageAsset {
                                name: name.to_string_lossy().to_string(),
                                path: format!("images/{}", name.to_string_lossy()),
                                description: String::new(),
                                similarity: self.settings.similarity,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Add an image to the project
    /// プロジェクトに画像を追加
    ///
    /// # Arguments
    /// * `source_path` - Path to the source image file
    /// * `name` - Name for the image in the project
    pub fn add_image(&mut self, source_path: &Path, name: &str) -> Result<(), SikulixError> {
        let images_dir = self.images_dir();
        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)?;
        }

        let dest_path = images_dir.join(name);
        fs::copy(source_path, &dest_path)?;

        self.images.push(ImageAsset {
            name: name.to_string(),
            path: format!("images/{}", name),
            description: String::new(),
            similarity: self.settings.similarity,
        });

        Ok(())
    }

    /// Remove an image from the project
    /// プロジェクトから画像を削除
    pub fn remove_image(&mut self, name: &str) -> Result<(), SikulixError> {
        let image_path = self.images_dir().join(name);
        if image_path.exists() {
            fs::remove_file(&image_path)?;
        }

        self.images.retain(|img| img.name != name);
        Ok(())
    }

    /// Get an image by name
    /// 名前で画像を取得
    pub fn get_image(&self, name: &str) -> Option<&ImageAsset> {
        self.images.iter().find(|img| img.name == name)
    }

    /// Read the main script content
    /// メインスクリプトの内容を読み込み
    pub fn read_script(&self) -> Result<String, SikulixError> {
        let script_path = self.main_script_path();
        if !script_path.exists() {
            return Ok(String::new());
        }
        Ok(fs::read_to_string(script_path)?)
    }

    /// Write the main script content
    /// メインスクリプトの内容を書き込み
    pub fn write_script(&self, content: &str) -> Result<(), SikulixError> {
        let script_path = self.main_script_path();
        Ok(fs::write(script_path, content)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_project_new() {
        let project = Project::new("TestProject");
        assert_eq!(project.name, "TestProject");
        assert_eq!(project.version, "1.0.0");
        assert_eq!(project.main_script, "main.py");
    }

    #[test]
    fn test_project_default() {
        let project = Project::default();
        assert_eq!(project.name, "Untitled");
        assert!((project.settings.similarity - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_project_settings_default() {
        let settings = ProjectSettings::default();
        assert!((settings.similarity - 0.7).abs() < f64::EPSILON);
        assert!(settings.ocr_language.is_empty());
        assert_eq!(settings.auto_save_interval, 0);
    }

    #[test]
    fn test_project_serialization() {
        let project = Project::new("SerializeTest");
        let json = serde_json::to_string(&project).unwrap();
        assert!(json.contains("SerializeTest"));
        assert!(json.contains("1.0.0"));

        let parsed: Project = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "SerializeTest");
    }

    #[test]
    fn test_project_create_and_load() {
        let temp_dir = env::temp_dir().join("sikulix_test_project");
        let project_dir = temp_dir.join("test.sikuli");

        // Clean up from previous test
        let _ = fs::remove_dir_all(&project_dir);

        // Create project
        let project = Project::create(&project_dir, "TestProject").unwrap();
        assert_eq!(project.name, "TestProject");
        assert!(project_dir.exists());
        assert!(project_dir.join("project.json").exists());
        assert!(project_dir.join("images").exists());
        assert!(project_dir.join("main.py").exists());

        // Load project
        let loaded = Project::load(&project_dir).unwrap();
        assert_eq!(loaded.name, "TestProject");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_paths() {
        let mut project = Project::new("PathTest");
        project.path = PathBuf::from("/test/project.sikuli");

        assert_eq!(
            project.main_script_path(),
            PathBuf::from("/test/project.sikuli/main.py")
        );
        assert_eq!(
            project.images_dir(),
            PathBuf::from("/test/project.sikuli/images")
        );
    }
}
