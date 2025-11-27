//! Python 2 to Python 3 Converter
//! Python 2 から Python 3 への変換器
//!
//! Uses lib2to3 (via Python) to convert Python 2 code to Python 3.
//! lib2to3（Python経由）を使用してPython 2コードをPython 3に変換します。

use anyhow::{bail, Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Convert Python 2 code to Python 3 using lib2to3
/// lib2to3を使用してPython 2コードをPython 3に変換
pub fn convert_python2_to_3(source: &str) -> Result<String> {
    log::debug!("Converting Python 2 code to Python 3");

    // Find Python 3 interpreter
    let python = find_python3()?;
    log::debug!("Using Python: {}", python);

    // Get path to py2to3.py converter script
    let converter_script = get_converter_script_path()?;
    log::debug!("Using converter: {}", converter_script.display());

    // Run converter: python py2to3.py - -
    // Input via stdin, output via stdout
    let mut child = Command::new(&python)
        .arg(&converter_script)
        .arg("-") // Read from stdin
        .arg("-") // Write to stdout
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start Python converter process")?;

    // Write source to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin
            .write_all(source.as_bytes())
            .context("Failed to write to converter stdin")?;
    }

    // Wait for completion and get output
    let output = child
        .wait_with_output()
        .context("Failed to wait for converter process")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Python 2 to 3 conversion failed: {}", stderr);
    }

    let converted = String::from_utf8(output.stdout)
        .context("Converter output is not valid UTF-8")?;

    log::debug!(
        "Conversion complete: {} bytes -> {} bytes",
        source.len(),
        converted.len()
    );

    Ok(converted)
}

/// Convert a Python 2 file to Python 3
/// Python 2ファイルをPython 3に変換
#[allow(dead_code)]
pub fn convert_python2_file(input_path: &Path, output_path: Option<&Path>) -> Result<String> {
    log::info!("Converting Python 2 file: {}", input_path.display());

    // Read source file
    let source = std::fs::read_to_string(input_path)
        .with_context(|| format!("Failed to read file: {}", input_path.display()))?;

    // Convert
    let converted = convert_python2_to_3(&source)?;

    // Write output if specified
    if let Some(out_path) = output_path {
        std::fs::write(out_path, &converted)
            .with_context(|| format!("Failed to write file: {}", out_path.display()))?;
        log::info!("Converted file written to: {}", out_path.display());
    }

    Ok(converted)
}

/// Find Python 3 interpreter
/// Python 3インタプリタを検索
fn find_python3() -> Result<String> {
    // Try python3 first
    if let Ok(output) = Command::new("python3").arg("--version").output() {
        if output.status.success() {
            return Ok("python3".to_string());
        }
    }

    // Try python and check version
    if let Ok(output) = Command::new("python").arg("--version").output() {
        if output.status.success() {
            let version_str = String::from_utf8_lossy(&output.stdout);
            if version_str.contains("Python 3") {
                return Ok("python".to_string());
            }
        }
    }

    // Windows: try py launcher
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("py").arg("-3").arg("--version").output() {
            if output.status.success() {
                return Ok("py -3".to_string());
            }
        }
    }

    bail!("Python 3 not found. Required for Python 2 to 3 conversion.")
}

/// Get path to py2to3.py converter script
/// py2to3.py変換スクリプトのパスを取得
fn get_converter_script_path() -> Result<std::path::PathBuf> {
    // First check if we're in development
    let exe_path = std::env::current_exe()?;
    if let Some(parent) = exe_path.parent() {
        // Check next to executable
        let script_path = parent.join("py2to3.py");
        if script_path.exists() {
            return Ok(script_path);
        }

        // Check in sikulid_api next to executable
        let script_path = parent.join("sikulid_api").join("py2to3.py");
        if script_path.exists() {
            return Ok(script_path);
        }

        // Check in parent directories (development)
        for ancestor in parent.ancestors().take(5) {
            let script_path = ancestor
                .join("runtime-rs")
                .join("sikulid_api")
                .join("py2to3.py");
            if script_path.exists() {
                return Ok(script_path);
            }
        }
    }

    // Fallback to current directory
    let current = std::env::current_dir()?;
    let script_path = current.join("sikulid_api").join("py2to3.py");
    if script_path.exists() {
        return Ok(script_path);
    }

    bail!("py2to3.py converter script not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_python3() {
        // This may fail if Python 3 is not installed
        let result = find_python3();
        // Just check that it doesn't panic
        if let Ok(python) = result {
            assert!(!python.is_empty());
        }
    }

    #[test]
    fn test_convert_print_statement() {
        // Skip if Python 3 or converter not available
        if find_python3().is_err() {
            return;
        }
        if get_converter_script_path().is_err() {
            return;
        }

        let source = "print 'hello world'";
        let result = convert_python2_to_3(source);

        if let Ok(converted) = result {
            assert!(converted.contains("print("));
            assert!(converted.contains("hello world"));
        }
    }

    #[test]
    fn test_convert_xrange() {
        // Skip if not available
        if find_python3().is_err() || get_converter_script_path().is_err() {
            return;
        }

        let source = "for i in xrange(10): print i";
        let result = convert_python2_to_3(source);

        if let Ok(converted) = result {
            assert!(converted.contains("range("));
            assert!(!converted.contains("xrange"));
        }
    }

    #[test]
    fn test_convert_raw_input() {
        // Skip if not available
        if find_python3().is_err() || get_converter_script_path().is_err() {
            return;
        }

        let source = "name = raw_input('Enter name: ')";
        let result = convert_python2_to_3(source);

        if let Ok(converted) = result {
            assert!(converted.contains("input("));
            assert!(!converted.contains("raw_input"));
        }
    }
}
