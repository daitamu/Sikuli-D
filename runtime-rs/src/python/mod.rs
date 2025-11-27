//! Python integration module
//! Python統合モジュール

use std::path::Path;
use anyhow::{Result, Context, bail};
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

/// Execute a Python script with SikuliX API
/// SikuliX API付きでPythonスクリプトを実行
pub fn execute_script(
    script_path: &Path,
    args: &[String],
    workdir: Option<&Path>,
    timeout_secs: u64,
) -> Result<()> {
    log::info!("Executing Python script: {}", script_path.display());

    // Find Python interpreter
    let python = find_python()?;
    log::debug!("Using Python: {}", python);

    // Build command
    let mut cmd = Command::new(&python);
    cmd.arg("-u"); // Unbuffered output

    // Add SikuliX API to Python path
    // This will be replaced with PyO3 module import
    let sikulix_api_path = get_sikulix_api_path()?;
    cmd.env("PYTHONPATH", &sikulix_api_path);

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
        // TODO: Implement timeout
        child.wait().context("Failed to wait for process")?
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

/// Find Python interpreter
/// Pythonインタプリタを検索
pub fn find_python() -> Result<String> {
    // Try python3 first
    if Command::new("python3").arg("--version").output().is_ok() {
        return Ok("python3".to_string());
    }

    // Try python
    if Command::new("python").arg("--version").output().is_ok() {
        return Ok("python".to_string());
    }

    // Windows: try py launcher
    #[cfg(windows)]
    if Command::new("py").arg("-3").arg("--version").output().is_ok() {
        return Ok("py -3".to_string());
    }

    bail!("Python not found. Please install Python 3.")
}

/// Get path to sikulix_api Python package
/// sikulix_api Pythonパッケージのパスを取得
pub fn get_sikulix_api_path() -> Result<String> {
    // First check if we're in development
    let exe_path = std::env::current_exe()?;
    if let Some(parent) = exe_path.parent() {
        // Check for sikulix_api next to executable
        let api_path = parent.join("sikulix_api");
        if api_path.exists() {
            return Ok(api_path.to_string_lossy().to_string());
        }

        // Check in parent directories (development)
        for ancestor in parent.ancestors().take(5) {
            let api_path = ancestor.join("runtime-rs").join("sikulix_api");
            if api_path.exists() {
                return Ok(api_path.to_string_lossy().to_string());
            }
        }
    }

    // Fallback to current directory
    let current = std::env::current_dir()?;
    let api_path = current.join("sikulix_api");
    if api_path.exists() {
        return Ok(api_path.to_string_lossy().to_string());
    }

    bail!("sikulix_api not found")
}
