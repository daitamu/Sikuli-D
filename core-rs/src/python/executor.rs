//! Script Executor
//! スクリプト実行エンジン
//!
//! Executes Python scripts with output streaming and process management.
//! 出力ストリーミングとプロセス管理を備えたPythonスクリプト実行。

use log::{debug, info, warn};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread;

use super::detector::PythonEnvironment;
use crate::{Result, SikulixError};

/// Script execution state
/// スクリプト実行状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionState {
    /// Not started / 未開始
    Idle,
    /// Running / 実行中
    Running,
    /// Completed successfully / 正常完了
    Completed,
    /// Completed with error / エラー終了
    Failed,
    /// Stopped by user / ユーザーによる停止
    Stopped,
}

impl std::fmt::Display for ExecutionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionState::Idle => write!(f, "Idle"),
            ExecutionState::Running => write!(f, "Running"),
            ExecutionState::Completed => write!(f, "Completed"),
            ExecutionState::Failed => write!(f, "Failed"),
            ExecutionState::Stopped => write!(f, "Stopped"),
        }
    }
}

/// Output type from script execution
/// スクリプト実行からの出力タイプ
#[derive(Debug, Clone)]
pub enum OutputLine {
    /// Standard output / 標準出力
    Stdout(String),
    /// Standard error / 標準エラー出力
    Stderr(String),
    /// Execution completed with exit code / 終了コード付き実行完了
    Exit(i32),
    /// Error message / エラーメッセージ
    Error(String),
}

/// Script executor for running Python scripts
/// Pythonスクリプト実行用エグゼキューター
pub struct ScriptExecutor {
    process: Option<Child>,
    state: Arc<Mutex<ExecutionState>>,
    output_receiver: Option<Receiver<OutputLine>>,
    python_env: Option<PythonEnvironment>,
    working_dir: Option<PathBuf>,
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptExecutor {
    /// Create a new script executor
    /// 新しいスクリプトエグゼキューターを作成
    pub fn new() -> Self {
        Self {
            process: None,
            state: Arc::new(Mutex::new(ExecutionState::Idle)),
            output_receiver: None,
            python_env: None,
            working_dir: None,
        }
    }

    /// Set Python environment to use
    /// 使用するPython環境を設定
    pub fn with_python(mut self, env: PythonEnvironment) -> Self {
        self.python_env = Some(env);
        self
    }

    /// Set working directory
    /// 作業ディレクトリを設定
    pub fn with_working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    /// Get current execution state
    /// 現在の実行状態を取得
    pub fn state(&self) -> ExecutionState {
        *self.state.lock().unwrap()
    }

    /// Check if script is running
    /// スクリプトが実行中か確認
    pub fn is_running(&self) -> bool {
        self.state() == ExecutionState::Running
    }

    /// Run a Python script file
    /// Pythonスクリプトファイルを実行
    pub fn run_file(&mut self, script_path: &Path) -> Result<Receiver<OutputLine>> {
        if self.is_running() {
            return Err(SikulixError::PythonError(
                "A script is already running".to_string(),
            ));
        }

        if !script_path.exists() {
            return Err(SikulixError::PythonError(format!(
                "Script not found: {}",
                script_path.display()
            )));
        }

        info!("Running script: {}", script_path.display());

        // Get Python environment
        let python_env = self
            .python_env
            .clone()
            .or_else(PythonEnvironment::detect_system);
        let python_env = python_env
            .ok_or_else(|| SikulixError::PythonError("No Python environment found".to_string()))?;

        debug!("Using Python: {}", python_env.path.display());

        // Build command
        let mut cmd = Command::new(&python_env.path);
        cmd.arg("-u"); // Unbuffered output
        cmd.arg(script_path);

        // Set working directory
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        } else if let Some(parent) = script_path.parent() {
            cmd.current_dir(parent);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        // Set environment variables
        cmd.env("PYTHONIOENCODING", "utf-8");

        // Spawn process
        let mut child = cmd
            .spawn()
            .map_err(|e| SikulixError::PythonError(format!("Failed to start Python: {}", e)))?;

        // Create output channel
        let (tx, rx) = channel();
        self.output_receiver = Some(rx);

        // Set state to running
        *self.state.lock().unwrap() = ExecutionState::Running;

        // Spawn output reader threads
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Stdout reader thread
        if let Some(stdout) = stdout {
            let tx_stdout = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let _ = tx_stdout.send(OutputLine::Stdout(line));
                        }
                        Err(e) => {
                            warn!("Error reading stdout: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Stderr reader thread
        if let Some(stderr) = stderr {
            let tx_stderr = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    match line {
                        Ok(line) => {
                            let _ = tx_stderr.send(OutputLine::Stderr(line));
                        }
                        Err(e) => {
                            warn!("Error reading stderr: {}", e);
                            break;
                        }
                    }
                }
            });
        }

        // Store process
        self.process = Some(child);

        // Spawn completion watcher
        let tx_completion = tx;
        let state_completion = Arc::clone(&self.state);
        thread::spawn(move || {
            // Wait a bit for process to be set
            thread::sleep(std::time::Duration::from_millis(100));

            // Monitor state changes
            loop {
                let current_state = *state_completion.lock().unwrap();
                if current_state != ExecutionState::Running {
                    break;
                }
                thread::sleep(std::time::Duration::from_millis(100));
            }

            // Send exit notification
            let final_state = *state_completion.lock().unwrap();
            let exit_code = match final_state {
                ExecutionState::Completed => 0,
                ExecutionState::Failed => 1,
                ExecutionState::Stopped => -1,
                _ => -2,
            };
            let _ = tx_completion.send(OutputLine::Exit(exit_code));
        });

        // Return receiver for output streaming
        Ok(self.output_receiver.take().unwrap())
    }

    /// Run Python code directly (not from file)
    /// Pythonコードを直接実行（ファイルからではなく）
    pub fn run_code(&mut self, code: &str) -> Result<Receiver<OutputLine>> {
        if self.is_running() {
            return Err(SikulixError::PythonError(
                "A script is already running".to_string(),
            ));
        }

        info!("Running inline code ({} chars)", code.len());

        // Get Python environment
        let python_env = self
            .python_env
            .clone()
            .or_else(PythonEnvironment::detect_system);
        let python_env = python_env
            .ok_or_else(|| SikulixError::PythonError("No Python environment found".to_string()))?;

        // Build command
        let mut cmd = Command::new(&python_env.path);
        cmd.arg("-u");
        cmd.arg("-c");
        cmd.arg(code);

        // Set working directory
        if let Some(ref dir) = self.working_dir {
            cmd.current_dir(dir);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());

        // Set environment variables
        cmd.env("PYTHONIOENCODING", "utf-8");

        // Spawn process
        let mut child = cmd
            .spawn()
            .map_err(|e| SikulixError::PythonError(format!("Failed to start Python: {}", e)))?;

        // Create output channel
        let (tx, rx) = channel();

        // Set state to running
        *self.state.lock().unwrap() = ExecutionState::Running;

        // Spawn output reader threads (same as run_file)
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            let tx_stdout = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines().map_while(|l| l.ok()) {
                    let _ = tx_stdout.send(OutputLine::Stdout(line));
                }
            });
        }

        if let Some(stderr) = stderr {
            let tx_stderr = tx.clone();
            thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines().map_while(|l| l.ok()) {
                    let _ = tx_stderr.send(OutputLine::Stderr(line));
                }
            });
        }

        // Store process and spawn completion watcher
        self.process = Some(child);

        let state_watcher = Arc::clone(&self.state);
        let tx_completion = tx;
        thread::spawn(move || loop {
            let state = *state_watcher.lock().unwrap();
            if state != ExecutionState::Running {
                let exit_code = match state {
                    ExecutionState::Completed => 0,
                    ExecutionState::Failed => 1,
                    ExecutionState::Stopped => -1,
                    _ => -2,
                };
                let _ = tx_completion.send(OutputLine::Exit(exit_code));
                break;
            }
            thread::sleep(std::time::Duration::from_millis(100));
        });

        Ok(rx)
    }

    /// Stop the running script
    /// 実行中のスクリプトを停止
    pub fn stop(&mut self) -> Result<()> {
        if !self.is_running() {
            return Ok(());
        }

        info!("Stopping script execution");

        if let Some(ref mut child) = self.process {
            // Try graceful termination first
            #[cfg(unix)]
            {
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }

            #[cfg(windows)]
            {
                // On Windows, just kill the process
                let _ = child.kill();
            }

            // Wait a bit then force kill if needed
            thread::sleep(std::time::Duration::from_millis(500));

            if let Ok(None) = child.try_wait() {
                warn!("Process didn't terminate, forcing kill");
                let _ = child.kill();
            }

            let _ = child.wait();
        }

        *self.state.lock().unwrap() = ExecutionState::Stopped;
        self.process = None;

        info!("Script stopped");
        Ok(())
    }

    /// Wait for script completion and return exit code
    /// スクリプト完了を待機して終了コードを返す
    pub fn wait(&mut self) -> Result<i32> {
        if let Some(ref mut child) = self.process {
            let status = child.wait().map_err(|e| {
                SikulixError::PythonError(format!("Failed to wait for process: {}", e))
            })?;

            let exit_code = status.code().unwrap_or(-1);

            *self.state.lock().unwrap() = if exit_code == 0 {
                ExecutionState::Completed
            } else {
                ExecutionState::Failed
            };

            self.process = None;
            Ok(exit_code)
        } else {
            Ok(0)
        }
    }
}

impl Drop for ScriptExecutor {
    fn drop(&mut self) {
        // Ensure process is terminated when executor is dropped
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state_display() {
        assert_eq!(format!("{}", ExecutionState::Idle), "Idle");
        assert_eq!(format!("{}", ExecutionState::Running), "Running");
        assert_eq!(format!("{}", ExecutionState::Completed), "Completed");
    }

    #[test]
    fn test_executor_creation() {
        let executor = ScriptExecutor::new();
        assert_eq!(executor.state(), ExecutionState::Idle);
        assert!(!executor.is_running());
    }
}
