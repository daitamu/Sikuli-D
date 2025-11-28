//! Python integration module
//! Python統合モジュール
//!
//! Supports automatic Python 2 to Python 3 conversion using lib2to3.
//! lib2to3を使用した自動Python 2→Python 3変換に対応。

use std::path::Path;
use std::time::Duration;
use anyhow::{Result, Context, bail};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use wait_timeout::ChildExt;

/// Execute a Python script with Sikuli-D API
/// Sikuli-D API付きでPythonスクリプトを実行
///
/// This function automatically detects Python 2 code and converts it to Python 3
/// using lib2to3 before execution.
/// Python 2コードを自動検出し、実行前にlib2to3でPython 3に変換します。
pub fn execute_script(
    script_path: &Path,
    args: &[String],
    workdir: Option<&Path>,
    timeout_secs: u64,
) -> Result<()> {
    log::info!("Executing Python script: {}", script_path.display());

    // Read script source for Python version detection
    let source = std::fs::read_to_string(script_path)
        .with_context(|| format!("Failed to read script: {}", script_path.display()))?;

    // Detect Python version
    let python_version = detect_python_version(&source);
    log::debug!("Detected Python version: {}", python_version);

    // Convert Python 2 code if needed
    let (script_to_run, temp_file) = if python_version == PythonVersionDetected::Python2 {
        log::info!("Python 2 code detected, converting to Python 3...");
        let converted = crate::converter::convert_python2_to_3(&source)
            .context("Failed to convert Python 2 code to Python 3")?;
        log::info!("Conversion successful");

        // Create temporary file for converted code
        let temp_path = script_path.with_extension("py3.tmp");
        std::fs::write(&temp_path, &converted)
            .with_context(|| format!("Failed to write converted script: {}", temp_path.display()))?;
        (temp_path.clone(), Some(temp_path))
    } else if python_version == PythonVersionDetected::Mixed {
        log::warn!("Warning: Script contains mixed Python 2/3 syntax");
        log::warn!("Attempting conversion anyway...");
        let converted = crate::converter::convert_python2_to_3(&source)
            .context("Failed to convert mixed Python 2/3 code")?;

        let temp_path = script_path.with_extension("py3.tmp");
        std::fs::write(&temp_path, &converted)
            .with_context(|| format!("Failed to write converted script: {}", temp_path.display()))?;
        (temp_path.clone(), Some(temp_path))
    } else {
        (script_path.to_path_buf(), None)
    };

    // Execute the script
    let result = execute_python_script(&script_to_run, args, workdir, timeout_secs);

    // Clean up temporary file
    if let Some(temp) = temp_file {
        if let Err(e) = std::fs::remove_file(&temp) {
            log::warn!("Failed to remove temporary file {}: {}", temp.display(), e);
        }
    }

    result
}

/// Python version detected from syntax
/// 構文から検出されたPythonバージョン
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonVersionDetected {
    /// Python 2 syntax detected / Python 2構文を検出
    Python2,
    /// Python 3 syntax detected / Python 3構文を検出
    Python3,
    /// Could be either version (compatible with both) / どちらのバージョンでも動作可能
    Unknown,
    /// Mixed Python 2/3 syntax / Python 2/3混在構文
    Mixed,
}

impl std::fmt::Display for PythonVersionDetected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonVersionDetected::Python2 => write!(f, "Python2 only"),
            PythonVersionDetected::Python3 => write!(f, "Python3 only"),
            PythonVersionDetected::Unknown => write!(f, "Python2/3 OK"),
            PythonVersionDetected::Mixed => write!(f, "Python2/3 Mixed"),
        }
    }
}

/// Detect Python version from source code
/// ソースコードからPythonバージョンを検出
pub fn detect_python_version(source: &str) -> PythonVersionDetected {
    let mut has_py2_syntax = false;
    let mut has_py3_syntax = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Python 2 specific patterns
        if is_python2_syntax(trimmed) {
            has_py2_syntax = true;
        }

        // Python 3 specific patterns
        if is_python3_syntax(trimmed) {
            has_py3_syntax = true;
        }
    }

    match (has_py2_syntax, has_py3_syntax) {
        (true, true) => PythonVersionDetected::Mixed,
        (true, false) => PythonVersionDetected::Python2,
        (false, true) => PythonVersionDetected::Python3,
        (false, false) => PythonVersionDetected::Unknown,
    }
}

/// Check for Python 2 specific syntax
/// Python 2特有の構文をチェック
fn is_python2_syntax(line: &str) -> bool {
    // print statement (not function)
    // print文（関数ではない）
    if line.starts_with("print ") && !line.contains("print(") {
        return true;
    }

    // Old-style exception handling: except Exception, e:
    // 旧式の例外処理
    if line.contains("except") && line.contains(",") && !line.contains(" as ") {
        return true;
    }

    // xrange (Python 2 only)
    if line.contains("xrange(") {
        return true;
    }

    // raw_input (Python 2 only)
    if line.contains("raw_input(") {
        return true;
    }

    // Long integer literal with L suffix (e.g., 123L)
    // L接尾辞付き長整数リテラル
    if has_long_literal(line) {
        return true;
    }

    // basestring (Python 2 only)
    if line.contains("basestring") {
        return true;
    }

    // execfile (Python 2 only)
    if line.contains("execfile(") {
        return true;
    }

    // has_key method (Python 2 dict method)
    if line.contains(".has_key(") {
        return true;
    }

    // Old division operator with __future__ import check
    // (We can't easily detect this without more context)

    false
}

/// Check for Python 3 specific syntax
/// Python 3特有の構文をチェック
fn is_python3_syntax(line: &str) -> bool {
    // print function with keywords
    // キーワード付きprint関数
    if line.contains("print(")
        && (line.contains("end=") || line.contains("sep=") || line.contains("file="))
    {
        return true;
    }

    // Type hints (def foo(x: int) -> str:)
    // 型ヒント
    if line.contains("->") && line.contains("def ") {
        return true;
    }

    // f-strings
    if line.contains("f\"") || line.contains("f'") {
        return true;
    }

    // async/await
    if line.starts_with("async ")
        || line.contains(" async ")
        || line.starts_with("await ")
        || line.contains(" await ")
    {
        return true;
    }

    // Walrus operator (:=)
    if line.contains(":=") {
        return true;
    }

    // nonlocal keyword
    if line.starts_with("nonlocal ") {
        return true;
    }

    // yield from
    if line.contains("yield from") {
        return true;
    }

    // Keyword-only arguments with *
    if line.contains("def ") && line.contains(", *,") {
        return true;
    }

    false
}

/// Check for long integer literals (123L)
/// 長整数リテラル(123L)をチェック
fn has_long_literal(line: &str) -> bool {
    let chars: Vec<char> = line.chars().collect();
    for i in 1..chars.len() {
        if (chars[i] == 'L' || chars[i] == 'l') && chars[i - 1].is_ascii_digit() {
            if i + 1 >= chars.len() || !chars[i + 1].is_alphanumeric() {
                return true;
            }
        }
    }
    false
}

/// Execute a Python script (internal implementation)
/// Pythonスクリプトを実行（内部実装）
fn execute_python_script(
    script_path: &Path,
    args: &[String],
    workdir: Option<&Path>,
    timeout_secs: u64,
) -> Result<()> {
    // Find Python interpreter
    let python_cmd = find_python()?;
    log::debug!("Using Python: {} {:?}", python_cmd.program, python_cmd.extra_args);

    // Build command
    let mut cmd = Command::new(&python_cmd.program);
    for arg in &python_cmd.extra_args {
        cmd.arg(arg);
    }
    cmd.arg("-u"); // Unbuffered output

    // Add Sikuli-D API to Python path
    let sikulid_api_path = get_sikulid_api_path()?;
    cmd.env("PYTHONPATH", &sikulid_api_path);

    cmd.arg(script_path);
    cmd.args(args);

    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    } else if let Some(script_dir) = script_path.parent() {
        cmd.current_dir(script_dir);
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    log::debug!("Command: {:?}", cmd);

    // Start process
    let mut child = cmd.spawn()
        .context("Failed to start Python process")?;

    // Handle output streaming
    let stdout = child.stdout.take().expect("stdout");
    let stderr = child.stderr.take().expect("stderr");

    // Spawn threads for output handling
    let stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
            }
        }
    });

    let stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                eprintln!("{}", line);
            }
        }
    });

    // Wait for completion (with optional timeout)
    let status = if timeout_secs > 0 {
        let timeout_duration = Duration::from_secs(timeout_secs);
        match child.wait_timeout(timeout_duration).context("Failed to wait for process")? {
            Some(status) => status,
            None => {
                // Timeout occurred - kill the process
                log::warn!("Script execution timed out after {}s, killing process...", timeout_secs);
                let _ = child.kill();
                let _ = child.wait(); // Reap the zombie process
                bail!("Script execution timed out after {}s / スクリプト実行がタイムアウトしました ({}秒)", timeout_secs, timeout_secs);
            }
        }
    } else {
        child.wait().context("Failed to wait for process")?
    };

    // Wait for output threads
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        bail!("Script exited with code: {}", code);
    }

    log::info!("Script completed successfully");
    Ok(())
}

/// Python command configuration
/// Pythonコマンド設定
pub struct PythonCommand {
    pub program: String,
    pub extra_args: Vec<String>,
}

impl PythonCommand {
    pub fn new(program: &str) -> Self {
        Self {
            program: program.to_string(),
            extra_args: Vec::new(),
        }
    }

    pub fn with_arg(mut self, arg: &str) -> Self {
        self.extra_args.push(arg.to_string());
        self
    }
}

/// Find Python interpreter
/// Pythonインタプリタを検索
pub fn find_python() -> Result<PythonCommand> {
    // Try python3 first
    if Command::new("python3").arg("--version").output().is_ok() {
        return Ok(PythonCommand::new("python3"));
    }

    // Try python
    if Command::new("python").arg("--version").output().is_ok() {
        return Ok(PythonCommand::new("python"));
    }

    // Windows: try py launcher
    #[cfg(windows)]
    if Command::new("py").arg("-3").arg("--version").output().is_ok() {
        return Ok(PythonCommand::new("py").with_arg("-3"));
    }

    bail!("Python not found. Please install Python 3.")
}

/// Get path to directory containing sikulid_api Python package
/// sikulid_api Pythonパッケージを含むディレクトリのパスを取得
///
/// Returns the parent directory of sikulid_api so that both `sikulid_api`
/// and `sikuli` (compatibility module) can be imported.
/// sikulid_apiの親ディレクトリを返し、`sikulid_api`と`sikuli`（互換モジュール）
/// の両方をインポートできるようにします。
pub fn get_sikulid_api_path() -> Result<String> {
    log::debug!("Searching for sikulid_api...");

    // First check if we're in development
    let exe_path = std::env::current_exe()?;
    log::debug!("Executable path: {}", exe_path.display());

    if let Some(parent) = exe_path.parent() {
        // Check for sikulid_api next to executable
        let api_path = parent.join("sikulid_api");
        log::debug!("Checking: {}", api_path.display());
        if api_path.exists() {
            log::debug!("Found sikulid_api at: {}", parent.display());
            return Ok(parent.to_string_lossy().to_string());
        }

        // Check in parent directories (development)
        // Walk up the directory tree looking for runtime-rs/sikulid_api
        let mut current = parent.to_path_buf();
        for _ in 0..10 {
            let api_path = current.join("runtime-rs").join("sikulid_api");
            log::debug!("Checking: {}", api_path.display());
            if api_path.exists() {
                let result = current.join("runtime-rs").to_string_lossy().to_string();
                log::debug!("Found sikulid_api at: {}", result);
                return Ok(result);
            }

            // Also check for sikulid_api directly (installed location)
            let direct_path = current.join("sikulid_api");
            if direct_path.exists() {
                let result = current.to_string_lossy().to_string();
                log::debug!("Found sikulid_api at: {}", result);
                return Ok(result);
            }

            if let Some(p) = current.parent() {
                current = p.to_path_buf();
            } else {
                break;
            }
        }
    }

    // Fallback to current directory
    let current = std::env::current_dir()?;
    log::debug!("Checking current directory: {}", current.display());
    let api_path = current.join("sikulid_api");
    if api_path.exists() {
        return Ok(current.to_string_lossy().to_string());
    }

    // Also check for runtime-rs/sikulid_api from current directory
    let api_path = current.join("runtime-rs").join("sikulid_api");
    if api_path.exists() {
        return Ok(current.join("runtime-rs").to_string_lossy().to_string());
    }

    bail!("sikulid_api not found. Please ensure runtime-rs/sikulid_api exists.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_python2_print() {
        let source = "print 'hello'";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python2);
    }

    #[test]
    fn test_detect_python2_xrange() {
        let source = "for i in xrange(10): pass";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python2);
    }

    #[test]
    fn test_detect_python2_raw_input() {
        let source = "name = raw_input('Name: ')";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python2);
    }

    #[test]
    fn test_detect_python3_fstring() {
        let source = "print(f'hello {name}')";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python3);
    }

    #[test]
    fn test_detect_python3_async() {
        let source = "async def foo(): pass";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python3);
    }

    #[test]
    fn test_detect_python3_type_hint() {
        let source = "def greet(name: str) -> str:\n    return name";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Python3);
    }

    #[test]
    fn test_detect_mixed() {
        let source = "print 'hello'\nprint(f'world')";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Mixed);
    }

    #[test]
    fn test_detect_unknown() {
        let source = "x = 1\ny = 2";
        assert_eq!(detect_python_version(source), PythonVersionDetected::Unknown);
    }

    #[test]
    fn test_has_long_literal() {
        assert!(has_long_literal("x = 123L"));
        assert!(has_long_literal("x = 100l + 200"));
        assert!(!has_long_literal("x = 'HELLO'"));
        assert!(!has_long_literal("x = Label"));
    }
}
