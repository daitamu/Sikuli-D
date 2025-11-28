//! Script Execution Engine / スクリプト実行エンジン
//!
//! Provides script execution functionality by calling sikulid CLI (runtime-rs) as a subprocess.
//! sikulid CLI (runtime-rs) をサブプロセスとして呼び出すことでスクリプト実行機能を提供します。
//!
//! Design principle: IDE does NOT use core-rs directly, it delegates to runtime-rs.
//! 設計原則：IDEはcore-rsを直接使用せず、runtime-rsに委譲します。

use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State, Window};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex as AsyncMutex;
use uuid::Uuid;

// ============================================================================
// Types and Structures / 型と構造体
// ============================================================================

/// Script execution options
/// スクリプト実行オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptRunOptions {
    /// Working directory / 作業ディレクトリ
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,

    /// Additional arguments / 追加引数
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables / 環境変数
    #[serde(default)]
    pub env_vars: HashMap<String, String>,

    /// Debug mode / デバッグモード
    #[serde(default)]
    pub debug: bool,

    /// Timeout in seconds / タイムアウト（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

impl Default for ScriptRunOptions {
    fn default() -> Self {
        Self {
            working_dir: None,
            args: Vec::new(),
            env_vars: HashMap::new(),
            debug: false,
            timeout_secs: None,
        }
    }
}

/// Result of script execution
/// スクリプト実行結果
#[derive(Debug, Clone, Serialize)]
pub struct ScriptRunResult {
    /// Exit code / 終了コード
    pub exit_code: i32,

    /// Standard output / 標準出力
    pub stdout: String,

    /// Standard error / 標準エラー
    pub stderr: String,

    /// Execution duration in milliseconds / 実行時間（ミリ秒）
    pub duration_ms: u64,

    /// Success flag / 成功フラグ
    pub success: bool,
}

/// Event payloads for streaming output
/// ストリーミング出力用のイベントペイロード
#[derive(Debug, Clone, Serialize)]
struct OutputEvent {
    process_id: String,
    line: String,
}

#[derive(Debug, Clone, Serialize)]
struct CompleteEvent {
    process_id: String,
    exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[allow(dead_code)]
struct ErrorEvent {
    process_id: String,
    error: String,
}

// ============================================================================
// Script Process State / スクリプトプロセス状態
// ============================================================================

/// Running process information
/// 実行中のプロセス情報
#[allow(dead_code)]
struct RunningProcess {
    child: Child,
    start_time: Instant,
    script_path: String,
}

/// State for managing running scripts
/// 実行中のスクリプトを管理する状態
pub struct ScriptProcessState {
    processes: Arc<AsyncMutex<HashMap<String, RunningProcess>>>,
}

impl Default for ScriptProcessState {
    fn default() -> Self {
        Self {
            processes: Arc::new(AsyncMutex::new(HashMap::new())),
        }
    }
}

impl ScriptProcessState {
    pub fn new() -> Self {
        Self::default()
    }
}

// ============================================================================
// Script Executor / スクリプト実行エンジン
// ============================================================================

/// Script executor that calls sikulid CLI
/// sikulid CLIを呼び出すスクリプト実行エンジン
pub struct ScriptExecutor {
    sikulid_path: PathBuf,
}

impl ScriptExecutor {
    /// Create new script executor
    /// 新しいスクリプト実行エンジンを作成
    pub fn new() -> Self {
        let sikulid_path = Self::find_sikulid_binary();
        debug!("ScriptExecutor initialized with path: {:?}", sikulid_path);
        Self { sikulid_path }
    }

    /// Find sikulid binary in various locations
    /// 様々な場所でsikulidバイナリを検索
    fn find_sikulid_binary() -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            "sikulid.exe"
        } else {
            "sikulid"
        };

        // 1. Check next to the IDE executable (production)
        // IDE実行ファイルの隣をチェック（本番環境）
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let sikulid_path = exe_dir.join(exe_name);
                debug!("Checking for sikulid at: {:?}", sikulid_path);
                if sikulid_path.exists() {
                    info!("Found sikulid at: {:?}", sikulid_path);
                    return sikulid_path;
                }

                // 2. Check in workspace target/release (development)
                // ワークスペースのtarget/releaseをチェック（開発環境）
                let mut current = exe_dir.to_path_buf();
                for _ in 0..10 {
                    // Check target/release
                    let release_path = current.join("target").join("release").join(exe_name);
                    debug!("Checking for sikulid at: {:?}", release_path);
                    if release_path.exists() {
                        info!("Found sikulid at: {:?}", release_path);
                        return release_path;
                    }

                    // Check target/debug
                    let debug_path = current.join("target").join("debug").join(exe_name);
                    debug!("Checking for sikulid at: {:?}", debug_path);
                    if debug_path.exists() {
                        info!("Found sikulid at: {:?}", debug_path);
                        return debug_path;
                    }

                    if let Some(parent) = current.parent() {
                        current = parent.to_path_buf();
                    } else {
                        break;
                    }
                }
            }
        }

        // 3. Fallback to PATH
        // PATHにフォールバック
        warn!("sikulid not found in known locations, falling back to PATH");
        PathBuf::from(exe_name)
    }

    /// Execute script and wait for completion
    /// スクリプトを実行して完了を待つ
    pub async fn execute(
        &self,
        script_path: &str,
        options: &ScriptRunOptions,
    ) -> Result<ScriptRunResult, String> {
        let start = Instant::now();
        info!("Executing script: {}", script_path);

        // Build command
        let mut cmd = self.build_command(script_path, options)?;

        // Execute with optional timeout
        let output = match options.timeout_secs {
            Some(timeout) => {
                tokio::time::timeout(Duration::from_secs(timeout), cmd.output())
                    .await
                    .map_err(|_| {
                        error!("Script execution timed out after {}s", timeout);
                        format!("Script execution timed out after {} seconds", timeout)
                    })?
                    .map_err(|e| {
                        error!("Failed to execute script: {}", e);
                        format!("Failed to execute script: {}", e)
                    })?
            }
            None => cmd.output().await.map_err(|e| {
                error!("Failed to execute script: {}", e);
                format!("Failed to execute script: {}", e)
            })?,
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        info!(
            "Script completed: exit_code={}, duration={}ms, success={}",
            exit_code, duration_ms, success
        );

        if !success {
            warn!("Script failed with stderr: {}", stderr);
        }

        Ok(ScriptRunResult {
            exit_code,
            stdout,
            stderr,
            duration_ms,
            success,
        })
    }

    /// Execute script with real-time output streaming
    /// リアルタイム出力ストリーミング付きでスクリプトを実行
    pub async fn execute_streaming(
        &self,
        script_path: &str,
        options: &ScriptRunOptions,
        window: &Window,
        state: &State<'_, ScriptProcessState>,
    ) -> Result<String, String> {
        let process_id = Uuid::new_v4().to_string();
        info!(
            "Starting streaming execution: script={}, process_id={}",
            script_path, process_id
        );

        // Build command
        let mut cmd = self.build_command(script_path, options)?;

        // Set up pipes for streaming
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // Spawn process
        let mut child = cmd.spawn().map_err(|e| {
            error!("Failed to spawn process: {}", e);
            format!("Failed to spawn process: {}", e)
        })?;

        // Take stdout and stderr handles
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        // Clone window for async tasks
        let window_clone = window.clone();
        let process_id_clone = process_id.clone();

        // Spawn task to stream stdout
        if let Some(stdout) = stdout {
            let window_stdout = window_clone.clone();
            let pid_stdout = process_id_clone.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("[{}] stdout: {}", pid_stdout, line);
                    let _ = window_stdout.emit(
                        "script-stdout",
                        OutputEvent {
                            process_id: pid_stdout.clone(),
                            line,
                        },
                    );
                }
                debug!("[{}] stdout stream ended", pid_stdout);
            });
        }

        // Spawn task to stream stderr
        if let Some(stderr) = stderr {
            let window_stderr = window_clone.clone();
            let pid_stderr = process_id_clone.clone();

            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    debug!("[{}] stderr: {}", pid_stderr, line);
                    let _ = window_stderr.emit(
                        "script-stderr",
                        OutputEvent {
                            process_id: pid_stderr.clone(),
                            line,
                        },
                    );
                }
                debug!("[{}] stderr stream ended", pid_stderr);
            });
        }

        // Store running process
        let running_process = RunningProcess {
            child,
            start_time: Instant::now(),
            script_path: script_path.to_string(),
        };

        {
            let mut processes = state.processes.lock().await;
            processes.insert(process_id.clone(), running_process);
            debug!(
                "Process stored: {} (total: {})",
                process_id,
                processes.len()
            );
        }

        // Spawn task to wait for completion
        let window_complete = window_clone.clone();
        let pid_complete = process_id_clone.clone();
        let processes_complete = state.inner().processes.clone();

        tokio::spawn(async move {
            // Wait for process to complete
            // Note: We DON'T remove from HashMap here - we need the process handle for stop_script
            // プロセスをHashMapから削除しない - stop_scriptでハンドルが必要
            let exit_code = loop {
                // Check if process still exists (might be killed by stop_script)
                let mut processes = processes_complete.lock().await;
                if let Some(running) = processes.get_mut(&pid_complete) {
                    // Try to check if process has exited
                    match running.child.try_wait() {
                        Ok(Some(status)) => {
                            // Process has exited - now remove from HashMap
                            let duration = running.start_time.elapsed();
                            let code = status.code();
                            info!(
                                "[{}] Process completed: exit_code={:?}, duration={:?}",
                                pid_complete, code, duration
                            );
                            processes.remove(&pid_complete);
                            break code;
                        }
                        Ok(None) => {
                            // Process still running - drop lock and wait a bit
                            drop(processes);
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                            continue;
                        }
                        Err(e) => {
                            error!("[{}] Failed to check process status: {}", pid_complete, e);
                            processes.remove(&pid_complete);
                            break None;
                        }
                    }
                } else {
                    // Process was removed by stop_script
                    debug!("[{}] Process was stopped externally", pid_complete);
                    break None;
                }
            };

            // Emit completion event
            let _ = window_complete.emit(
                "script-complete",
                CompleteEvent {
                    process_id: pid_complete.clone(),
                    exit_code,
                },
            );

            debug!("[{}] Completion event emitted", pid_complete);
        });

        Ok(process_id)
    }

    /// Build Command for sikulix CLI
    /// sikulix CLI用のCommandを構築
    fn build_command(
        &self,
        script_path: &str,
        options: &ScriptRunOptions,
    ) -> Result<Command, String> {
        debug!("Building command for script: {}", script_path);

        let mut cmd = Command::new(&self.sikulid_path);

        // Add 'run' subcommand
        cmd.arg("run").arg(script_path);

        // Add debug flag if enabled
        if options.debug {
            cmd.arg("--debug");
            debug!("Debug mode enabled");
        }

        // Set working directory
        if let Some(ref dir) = options.working_dir {
            cmd.current_dir(dir);
            debug!("Working directory: {}", dir);
        }

        // Add script arguments (after --)
        if !options.args.is_empty() {
            cmd.arg("--").args(&options.args);
            debug!("Script arguments: {:?}", options.args);
        }

        // Set environment variables
        for (key, value) in &options.env_vars {
            cmd.env(key, value);
            debug!("Environment variable: {}={}", key, value);
        }

        Ok(cmd)
    }
}

impl Default for ScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tauri Commands / Tauriコマンド
// ============================================================================

/// Execute a script and wait for completion
/// スクリプトを実行して完了を待つ
#[tauri::command]
pub async fn run_script(
    script_path: String,
    options: ScriptRunOptions,
) -> Result<ScriptRunResult, String> {
    info!("Command: run_script - path={}", script_path);

    let executor = ScriptExecutor::new();
    executor.execute(&script_path, &options).await
}

/// Execute a script with real-time output streaming
/// リアルタイム出力ストリーミング付きでスクリプトを実行
#[tauri::command]
pub async fn run_script_streaming(
    _app: AppHandle,
    window: Window,
    script_path: String,
    options: ScriptRunOptions,
    state: State<'_, ScriptProcessState>,
) -> Result<String, String> {
    info!("Command: run_script_streaming - path={}", script_path);

    let executor = ScriptExecutor::new();
    executor
        .execute_streaming(&script_path, &options, &window, &state)
        .await
}

/// Stop a running script by process ID
/// プロセスIDによって実行中のスクリプトを停止
#[tauri::command]
pub async fn stop_script(
    process_id: String,
    state: State<'_, ScriptProcessState>,
) -> Result<(), String> {
    info!("Command: stop_script - process_id={}", process_id);

    let mut processes = state.processes.lock().await;

    if let Some(mut running) = processes.remove(&process_id) {
        match running.child.kill().await {
            Ok(_) => {
                info!("Process {} killed successfully", process_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to kill process {}: {}", process_id, e);
                Err(format!("Failed to kill process: {}", e))
            }
        }
    } else {
        warn!("Process {} not found", process_id);
        Err(format!("Process {} not found", process_id))
    }
}

/// Get list of running processes
/// 実行中のプロセスのリストを取得
#[tauri::command]
pub async fn get_running_processes(
    state: State<'_, ScriptProcessState>,
) -> Result<Vec<String>, String> {
    let processes = state.processes.lock().await;
    let process_ids: Vec<String> = processes.keys().cloned().collect();
    debug!("Running processes: {:?}", process_ids);
    Ok(process_ids)
}

/// Stop all running scripts
/// すべての実行中のスクリプトを停止
#[tauri::command]
pub async fn stop_all_scripts(state: State<'_, ScriptProcessState>) -> Result<(), String> {
    info!("Command: stop_all_scripts");

    let mut processes = state.processes.lock().await;
    let count = processes.len();

    for (process_id, mut running) in processes.drain() {
        match running.child.kill().await {
            Ok(_) => debug!("Killed process: {}", process_id),
            Err(e) => warn!("Failed to kill process {}: {}", process_id, e),
        }
    }

    info!("Stopped {} processes", count);
    Ok(())
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_run_options_default() {
        let options = ScriptRunOptions::default();
        assert!(options.working_dir.is_none());
        assert!(options.args.is_empty());
        assert!(options.env_vars.is_empty());
        assert!(!options.debug);
        assert!(options.timeout_secs.is_none());
    }

    #[test]
    fn test_script_run_options_serialization() {
        let mut options = ScriptRunOptions::default();
        options.debug = true;
        options.args = vec!["--arg1".to_string(), "value1".to_string()];

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("\"debug\":true"));
        assert!(json.contains("--arg1"));
    }

    #[test]
    fn test_script_executor_creation() {
        let executor = ScriptExecutor::new();
        assert!(executor.sikulid_path.to_string_lossy().contains("sikulid"));
    }

    #[test]
    fn test_script_process_state_default() {
        let state = ScriptProcessState::default();
        // Just ensure it can be created without panic
        assert!(state.processes.try_lock().is_ok());
    }

    #[tokio::test]
    async fn test_build_command_basic() {
        let executor = ScriptExecutor::new();
        let options = ScriptRunOptions::default();

        let cmd = executor.build_command("test.py", &options);
        assert!(cmd.is_ok());
    }

    #[tokio::test]
    async fn test_build_command_with_debug() {
        let executor = ScriptExecutor::new();
        let mut options = ScriptRunOptions::default();
        options.debug = true;

        let cmd = executor.build_command("test.py", &options);
        assert!(cmd.is_ok());
    }

    #[tokio::test]
    async fn test_build_command_with_args() {
        let executor = ScriptExecutor::new();
        let mut options = ScriptRunOptions::default();
        options.args = vec!["arg1".to_string(), "arg2".to_string()];

        let cmd = executor.build_command("test.py", &options);
        assert!(cmd.is_ok());
    }
}
