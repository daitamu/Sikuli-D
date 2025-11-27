//! Python Environment Detection
//! Python 環境検出
//!
//! Detects Python installations on the system including:
//! システム上のPythonインストールを検出:
//! - System Python (python, python3, python2 on PATH)
//! - Windows py launcher
//! - Virtual environments (venv, virtualenv, conda)

use log::{debug, info, warn};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Python environment information
/// Python環境情報
#[derive(Debug, Clone)]
pub struct PythonEnvironment {
    /// Path to Python executable / Python実行ファイルへのパス
    pub path: PathBuf,
    /// Version tuple (major, minor, patch) / バージョンタプル
    pub version: (u8, u8, u8),
    /// Whether this is a virtual environment / 仮想環境かどうか
    pub is_venv: bool,
    /// Path to virtual environment root (if applicable) / 仮想環境ルートパス
    pub venv_path: Option<PathBuf>,
    /// Display name / 表示名
    pub display_name: String,
}

impl PythonEnvironment {
    /// Detect all available Python environments
    /// 利用可能なすべてのPython環境を検出
    pub fn detect_all() -> Vec<Self> {
        info!("Detecting all Python environments");
        let mut environments = Vec::new();

        // Try Windows py launcher first
        if cfg!(windows) {
            if let Some(envs) = Self::detect_py_launcher() {
                environments.extend(envs);
            }
        }

        // Try common Python commands
        for cmd in &["python3", "python", "python2"] {
            if let Some(env) = Self::detect_from_command(cmd) {
                // Avoid duplicates
                if !environments.iter().any(|e| e.path == env.path) {
                    environments.push(env);
                }
            }
        }

        // Sort by version (newest first)
        environments.sort_by(|a, b| b.version.cmp(&a.version));

        info!("Found {} Python environments", environments.len());
        for env in &environments {
            debug!(
                "  {} - Python {}.{}.{} at {}",
                env.display_name,
                env.version.0,
                env.version.1,
                env.version.2,
                env.path.display()
            );
        }

        environments
    }

    /// Detect the default system Python
    /// デフォルトのシステムPythonを検出
    pub fn detect_system() -> Option<Self> {
        info!("Detecting system Python");

        // Prefer python3 over python
        if let Some(env) = Self::detect_from_command("python3") {
            return Some(env);
        }

        if let Some(env) = Self::detect_from_command("python") {
            return Some(env);
        }

        // Try py launcher on Windows
        if cfg!(windows) {
            if let Some(envs) = Self::detect_py_launcher() {
                // Return the newest Python 3
                return envs.into_iter().find(|e| e.version.0 >= 3);
            }
        }

        warn!("No system Python found");
        None
    }

    /// Detect Python from a specific command
    /// 特定のコマンドからPythonを検出
    fn detect_from_command(cmd: &str) -> Option<Self> {
        debug!("Trying to detect Python from command: {}", cmd);

        // Get Python path using 'which' or 'where'
        let path = Self::get_python_path(cmd)?;

        // Get version
        let version = Self::get_python_version(&path)?;

        // Check if it's a venv
        let (is_venv, venv_path) = Self::check_venv(&path);

        let display_name = if is_venv {
            format!("Python {}.{}.{} (venv)", version.0, version.1, version.2)
        } else {
            format!("Python {}.{}.{}", version.0, version.1, version.2)
        };

        Some(PythonEnvironment {
            path,
            version,
            is_venv,
            venv_path,
            display_name,
        })
    }

    /// Detect Python installations via Windows py launcher
    /// Windows py ランチャー経由でPythonインストールを検出
    #[cfg(windows)]
    fn detect_py_launcher() -> Option<Vec<Self>> {
        debug!("Trying Windows py launcher");

        // Get list of installed Pythons
        let output = Command::new("py").arg("-0p").output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut environments = Vec::new();

        for line in stdout.lines() {
            // Format: " -V:3.11 *        C:\Python311\python.exe"
            // Or: " -V:3.11          C:\Python311\python.exe"
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Parse version and path
            if let Some((version_str, path_str)) = Self::parse_py_launcher_line(line) {
                if let Some(version) = Self::parse_version_string(&version_str) {
                    let path = PathBuf::from(path_str.trim());
                    if path.exists() {
                        let (is_venv, venv_path) = Self::check_venv(&path);
                        let display_name = format!(
                            "Python {}.{}.{} (py launcher)",
                            version.0, version.1, version.2
                        );
                        environments.push(PythonEnvironment {
                            path,
                            version,
                            is_venv,
                            venv_path,
                            display_name,
                        });
                    }
                }
            }
        }

        if environments.is_empty() {
            None
        } else {
            Some(environments)
        }
    }

    #[cfg(not(windows))]
    fn detect_py_launcher() -> Option<Vec<Self>> {
        None
    }

    /// Parse a line from py launcher output
    /// py ランチャー出力の行をパース
    #[cfg(windows)]
    fn parse_py_launcher_line(line: &str) -> Option<(String, String)> {
        // Format: " -V:3.11 *        C:\Python311\python.exe"
        let line = line.trim_start_matches([' ', '-', 'V', ':'].as_ref());

        // Find the path (starts with a drive letter or backslash)
        let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace()).collect();
        if !parts.is_empty() {
            let version_part = parts[0].trim_end_matches('*').trim();

            // Find path in the rest
            if parts.len() >= 2 {
                let rest = parts[1].trim();
                // Find path starting with drive letter
                if let Some(path_start) = rest.find(|c: char| c.is_ascii_alphabetic()) {
                    let path = &rest[path_start..];
                    if path.len() > 2 && &path[1..2] == ":" {
                        return Some((version_part.to_string(), path.trim().to_string()));
                    }
                }
            }
        }
        None
    }

    /// Get Python executable path
    /// Python実行ファイルパスを取得
    fn get_python_path(cmd: &str) -> Option<PathBuf> {
        #[cfg(windows)]
        let which_cmd = "where";
        #[cfg(not(windows))]
        let which_cmd = "which";

        let output = Command::new(which_cmd).arg(cmd).output().ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let path_str = stdout.lines().next()?.trim();

        if path_str.is_empty() {
            return None;
        }

        let path = PathBuf::from(path_str);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    /// Get Python version from executable
    /// 実行ファイルからPythonバージョンを取得
    fn get_python_version(python_path: &PathBuf) -> Option<(u8, u8, u8)> {
        let output = Command::new(python_path).arg("--version").output().ok()?;

        // Python version can be in stdout or stderr depending on version
        let version_str = if !output.stdout.is_empty() {
            String::from_utf8_lossy(&output.stdout)
        } else {
            String::from_utf8_lossy(&output.stderr)
        };

        Self::parse_version_output(&version_str)
    }

    /// Parse version from "Python X.Y.Z" output
    /// "Python X.Y.Z" 出力からバージョンをパース
    fn parse_version_output(output: &str) -> Option<(u8, u8, u8)> {
        // Format: "Python 3.11.5" or "Python 2.7.18"
        let output = output.trim();
        let version_part = output.strip_prefix("Python ")?.trim();
        Self::parse_version_string(version_part)
    }

    /// Parse version string "X.Y.Z" or "X.Y"
    /// バージョン文字列 "X.Y.Z" または "X.Y" をパース
    fn parse_version_string(version: &str) -> Option<(u8, u8, u8)> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() >= 2 {
            let major = parts[0].parse().ok()?;
            let minor = parts[1].parse().ok()?;
            let patch = parts.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
            Some((major, minor, patch))
        } else {
            None
        }
    }

    /// Check if Python is running in a virtual environment
    /// Pythonが仮想環境で実行されているか確認
    fn check_venv(python_path: &Path) -> (bool, Option<PathBuf>) {
        // Check for VIRTUAL_ENV environment variable indicator
        // Check if pyvenv.cfg exists in parent directories

        let mut current = python_path.parent();
        while let Some(dir) = current {
            let pyvenv_cfg = dir.join("pyvenv.cfg");
            if pyvenv_cfg.exists() {
                return (true, Some(dir.to_path_buf()));
            }

            // Also check parent (for Scripts/python.exe on Windows)
            let parent_pyvenv = dir.parent().map(|p| p.join("pyvenv.cfg"));
            if let Some(cfg) = parent_pyvenv {
                if cfg.exists() {
                    return (true, dir.parent().map(|p| p.to_path_buf()));
                }
            }

            current = dir.parent();

            // Don't go too far up
            if current.map(|p| p.components().count()).unwrap_or(0) < 2 {
                break;
            }
        }

        (false, None)
    }

    /// Check if this Python version supports a feature
    /// このPythonバージョンが機能をサポートしているか確認
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "async" => self.version.0 >= 3 && self.version.1 >= 5,
            "fstring" => self.version.0 >= 3 && self.version.1 >= 6,
            "walrus" => self.version.0 >= 3 && self.version.1 >= 8,
            "match" => self.version.0 >= 3 && self.version.1 >= 10,
            _ => true,
        }
    }

    /// Get a short version string
    /// 短いバージョン文字列を取得
    pub fn version_string(&self) -> String {
        format!("{}.{}.{}", self.version.0, self.version.1, self.version.2)
    }

    /// Check if this is Python 2
    /// Python 2かどうか
    pub fn is_python2(&self) -> bool {
        self.version.0 == 2
    }

    /// Check if this is Python 3
    /// Python 3かどうか
    pub fn is_python3(&self) -> bool {
        self.version.0 == 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_output() {
        assert_eq!(
            PythonEnvironment::parse_version_output("Python 3.11.5"),
            Some((3, 11, 5))
        );
        assert_eq!(
            PythonEnvironment::parse_version_output("Python 2.7.18"),
            Some((2, 7, 18))
        );
        assert_eq!(
            PythonEnvironment::parse_version_output("Python 3.9"),
            Some((3, 9, 0))
        );
    }

    #[test]
    fn test_parse_version_string() {
        assert_eq!(
            PythonEnvironment::parse_version_string("3.11.5"),
            Some((3, 11, 5))
        );
        assert_eq!(
            PythonEnvironment::parse_version_string("3.11"),
            Some((3, 11, 0))
        );
    }

    #[test]
    fn test_supports_feature() {
        let env = PythonEnvironment {
            path: PathBuf::new(),
            version: (3, 10, 0),
            is_venv: false,
            venv_path: None,
            display_name: "Test".to_string(),
        };

        assert!(env.supports_feature("async"));
        assert!(env.supports_feature("fstring"));
        assert!(env.supports_feature("walrus"));
        assert!(env.supports_feature("match"));
    }

    #[test]
    fn test_is_python_version() {
        let py2 = PythonEnvironment {
            path: PathBuf::new(),
            version: (2, 7, 18),
            is_venv: false,
            venv_path: None,
            display_name: "Test".to_string(),
        };

        let py3 = PythonEnvironment {
            path: PathBuf::new(),
            version: (3, 11, 0),
            is_venv: false,
            venv_path: None,
            display_name: "Test".to_string(),
        };

        assert!(py2.is_python2());
        assert!(!py2.is_python3());
        assert!(!py3.is_python2());
        assert!(py3.is_python3());
    }
}
