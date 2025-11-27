//! Debug panel integration for IDE
//! IDE用デバッグパネル統合
//!
//! Provides Tauri commands to control the core-rs debugger from the IDE frontend.
//! core-rsデバッガをIDEフロントエンドから制御するためのTauriコマンドを提供します。

use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use sikulix_core::debug::{CallFrame, DebugEvent, DebugState as CoreDebugState, Debugger, Scope, VariableInfo};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Window};

// ============================================================================
// State Management / 状態管理
// ============================================================================

/// Debug state for the IDE
/// IDE用デバッグ状態
#[derive(Default)]
pub struct DebugPanelState {
    /// Core debugger instance / コアデバッガインスタンス
    debugger: Arc<Mutex<Option<Debugger>>>,
    /// Currently debugging script path / 現在デバッグ中のスクリプトパス
    current_script: Mutex<Option<PathBuf>>,
    /// Debug session active flag / デバッグセッション有効フラグ
    is_active: Mutex<bool>,
}

impl DebugPanelState {
    pub fn new() -> Self {
        Self {
            debugger: Arc::new(Mutex::new(None)),
            current_script: Mutex::new(None),
            is_active: Mutex::new(false),
        }
    }

    /// Get or create debugger instance
    /// デバッガインスタンスを取得または作成
    fn get_debugger(&self) -> Arc<Mutex<Option<Debugger>>> {
        self.debugger.clone()
    }

    /// Initialize debugger for a new session
    /// 新しいセッションのためにデバッガを初期化
    fn init_debugger(&self, window: Window) {
        let mut debugger = self.debugger.lock().unwrap();
        let new_debugger = Debugger::new();

        // Register event callback to forward events to frontend
        // フロントエンドにイベントを転送するためのイベントコールバックを登録
        let window_clone = window.clone();
        new_debugger.register_callback(move |event| {
            let event_data = DebugEventData::from_core_event(event);
            if let Err(e) = window_clone.emit("debug-event", event_data) {
                error!("Failed to emit debug event: {}", e);
            }
        });

        *debugger = Some(new_debugger);
        *self.is_active.lock().unwrap() = true;
    }

    /// Clean up debugger session
    /// デバッガセッションをクリーンアップ
    fn cleanup_debugger(&self) {
        let mut debugger = self.debugger.lock().unwrap();
        if let Some(dbg) = debugger.as_ref() {
            dbg.reset();
        }
        *debugger = None;
        *self.current_script.lock().unwrap() = None;
        *self.is_active.lock().unwrap() = false;
    }
}

// ============================================================================
// Data Transfer Objects / データ転送オブジェクト
// ============================================================================

/// Debug state for frontend
/// フロントエンド用デバッグ状態
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DebugState {
    NotStarted,
    Running,
    Paused,
    StepOver,
    StepInto,
    StepOut,
    Stopped,
    Error,
}

impl From<CoreDebugState> for DebugState {
    fn from(state: CoreDebugState) -> Self {
        match state {
            CoreDebugState::NotStarted => DebugState::NotStarted,
            CoreDebugState::Running => DebugState::Running,
            CoreDebugState::Paused => DebugState::Paused,
            CoreDebugState::StepOver => DebugState::StepOver,
            CoreDebugState::StepInto => DebugState::StepInto,
            CoreDebugState::StepOut => DebugState::StepOut,
            CoreDebugState::Stopped => DebugState::Stopped,
            CoreDebugState::Error => DebugState::Error,
        }
    }
}

/// Variable information for frontend
/// フロントエンド用変数情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariableInfoData {
    pub name: String,
    pub value: String,
    pub type_name: String,
    pub scope: String,
}

impl From<VariableInfo> for VariableInfoData {
    fn from(info: VariableInfo) -> Self {
        let scope_str = match info.scope {
            Scope::Local => "local",
            Scope::Global => "global",
            Scope::All => "all",
        };

        Self {
            name: info.name,
            value: format!("{}", info.value),
            type_name: info.type_name,
            scope: scope_str.to_string(),
        }
    }
}

/// Call frame information for frontend
/// フロントエンド用コールフレーム情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFrameData {
    pub depth: usize,
    pub function: String,
    pub file: String,
    pub line: u32,
}

impl From<CallFrame> for CallFrameData {
    fn from(frame: CallFrame) -> Self {
        Self {
            depth: frame.depth,
            function: frame.function,
            file: frame.file.to_string_lossy().to_string(),
            line: frame.line,
        }
    }
}

/// Debug event data for frontend
/// フロントエンド用デバッグイベントデータ
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum DebugEventData {
    BreakpointHit {
        file: String,
        line: u32,
        hit_count: u32,
    },
    Paused {
        file: String,
        line: u32,
    },
    Resumed,
    StepCompleted {
        file: String,
        line: u32,
    },
    Stopped,
    Error {
        message: String,
    },
    VariableChanged {
        name: String,
        value: String,
    },
}

impl DebugEventData {
    fn from_core_event(event: DebugEvent) -> Self {
        match event {
            DebugEvent::BreakpointHit { file, line, hit_count } => {
                DebugEventData::BreakpointHit {
                    file: file.to_string_lossy().to_string(),
                    line,
                    hit_count,
                }
            }
            DebugEvent::Paused { file, line } => DebugEventData::Paused {
                file: file.to_string_lossy().to_string(),
                line,
            },
            DebugEvent::Resumed => DebugEventData::Resumed,
            DebugEvent::StepCompleted { file, line } => DebugEventData::StepCompleted {
                file: file.to_string_lossy().to_string(),
                line,
            },
            DebugEvent::Stopped => DebugEventData::Stopped,
            DebugEvent::Error { message } => DebugEventData::Error { message },
            DebugEvent::VariableChanged { name, value } => DebugEventData::VariableChanged {
                name,
                value: format!("{}", value),
            },
        }
    }
}

/// Breakpoint information
/// ブレークポイント情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakpointInfo {
    pub file: String,
    pub line: u32,
}

// ============================================================================
// Tauri Commands / Tauriコマンド
// ============================================================================

/// Initialize debug session
/// デバッグセッションを初期化
#[tauri::command]
pub fn debug_init_session(
    window: Window,
    script_path: String,
    state: State<DebugPanelState>,
) -> Result<(), String> {
    info!("Initializing debug session for: {}", script_path);

    state.init_debugger(window);
    *state.current_script.lock().unwrap() = Some(PathBuf::from(script_path));

    Ok(())
}

/// End debug session
/// デバッグセッションを終了
#[tauri::command]
pub fn debug_end_session(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Ending debug session");
    state.cleanup_debugger();
    Ok(())
}

/// Add a breakpoint
/// ブレークポイントを追加
#[tauri::command]
pub fn debug_add_breakpoint(
    file: String,
    line: u32,
    state: State<DebugPanelState>,
) -> Result<(), String> {
    debug!("Adding breakpoint at {}:{}", file, line);

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.add_breakpoint(&file, line);
        Ok(())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Remove a breakpoint
/// ブレークポイントを削除
#[tauri::command]
pub fn debug_remove_breakpoint(
    file: String,
    line: u32,
    state: State<DebugPanelState>,
) -> Result<(), String> {
    debug!("Removing breakpoint at {}:{}", file, line);

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.remove_breakpoint(&file, line);
        Ok(())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Toggle breakpoint
/// ブレークポイントを切り替え
#[tauri::command]
pub fn debug_toggle_breakpoint(
    file: String,
    line: u32,
    state: State<DebugPanelState>,
) -> Result<bool, String> {
    debug!("Toggling breakpoint at {}:{}", file, line);

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.toggle_breakpoint(&file, line);
        let is_set = dbg.has_breakpoint(&file, line);
        Ok(is_set)
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// List all breakpoints
/// すべてのブレークポイントをリスト
#[tauri::command]
pub fn debug_list_breakpoints(
    state: State<DebugPanelState>,
) -> Result<Vec<BreakpointInfo>, String> {
    debug!("Listing all breakpoints");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        let breakpoints = dbg
            .list_breakpoints()
            .into_iter()
            .map(|(file, line)| BreakpointInfo { file, line })
            .collect();
        Ok(breakpoints)
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Clear all breakpoints
/// すべてのブレークポイントをクリア
#[tauri::command]
pub fn debug_clear_breakpoints(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Clearing all breakpoints");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.clear_all_breakpoints();
        Ok(())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Pause execution
/// 実行を一時停止
#[tauri::command]
pub fn debug_pause(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Pausing execution");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.pause().map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Resume execution
/// 実行を再開
#[tauri::command]
pub fn debug_resume(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Resuming execution");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.resume().map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Step over current line
/// 現在の行をステップオーバー
#[tauri::command]
pub fn debug_step_over(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Step over");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.step_over().map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Step into function
/// 関数にステップイン
#[tauri::command]
pub fn debug_step_into(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Step into");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.step_into().map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Step out of current function
/// 現在の関数からステップアウト
#[tauri::command]
pub fn debug_step_out(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Step out");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.step_out().map_err(|e| e.to_string())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Stop execution
/// 実行を停止
#[tauri::command]
pub fn debug_stop(state: State<DebugPanelState>) -> Result<(), String> {
    info!("Stopping execution");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        dbg.stop().map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Get current debug state
/// 現在のデバッグ状態を取得
#[tauri::command]
pub fn debug_get_state(state: State<DebugPanelState>) -> Result<DebugState, String> {
    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        Ok(DebugState::from(dbg.get_state()))
    } else {
        Ok(DebugState::NotStarted)
    }
}

/// Get variables in the specified scope
/// 指定されたスコープの変数を取得
#[tauri::command]
pub fn debug_get_variables(
    scope: Option<String>,
    state: State<DebugPanelState>,
) -> Result<Vec<VariableInfoData>, String> {
    debug!("Getting variables with scope: {:?}", scope);

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        let scope_enum = match scope.as_deref() {
            Some("local") => Scope::Local,
            Some("global") => Scope::Global,
            _ => Scope::All,
        };

        let variables = dbg
            .get_variables(scope_enum)
            .into_iter()
            .map(VariableInfoData::from)
            .collect();

        Ok(variables)
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Get call stack
/// コールスタックを取得
#[tauri::command]
pub fn debug_get_call_stack(
    state: State<DebugPanelState>,
) -> Result<Vec<CallFrameData>, String> {
    debug!("Getting call stack");

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        let call_stack = dbg
            .get_call_stack()
            .into_iter()
            .map(CallFrameData::from)
            .collect();

        Ok(call_stack)
    } else {
        Err("Debugger not initialized".to_string())
    }
}

/// Get current execution position
/// 現在の実行位置を取得
#[tauri::command]
pub fn debug_get_current_position(
    state: State<DebugPanelState>,
) -> Result<Option<(String, u32)>, String> {
    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        let (file, line) = dbg.get_current_position();
        match (file, line) {
            (Some(f), Some(l)) => Ok(Some((f.to_string_lossy().to_string(), l))),
            _ => Ok(None),
        }
    } else {
        Ok(None)
    }
}

/// Evaluate expression in current context
/// 現在のコンテキストで式を評価
#[tauri::command]
pub fn debug_evaluate_expression(
    expr: String,
    state: State<DebugPanelState>,
) -> Result<String, String> {
    debug!("Evaluating expression: {}", expr);

    let debugger = state.get_debugger();
    let debugger = debugger.lock().unwrap();

    if let Some(dbg) = debugger.as_ref() {
        match dbg.evaluate_expression(&expr) {
            Ok(value) => Ok(format!("{}", value)),
            Err(e) => Err(e.to_string()),
        }
    } else {
        Err("Debugger not initialized".to_string())
    }
}

// ============================================================================
// Tests / テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_state_conversion() {
        assert!(matches!(
            DebugState::from(CoreDebugState::NotStarted),
            DebugState::NotStarted
        ));
        assert!(matches!(
            DebugState::from(CoreDebugState::Running),
            DebugState::Running
        ));
        assert!(matches!(
            DebugState::from(CoreDebugState::Paused),
            DebugState::Paused
        ));
    }

    #[test]
    fn test_debug_panel_state_creation() {
        let state = DebugPanelState::new();
        assert!(state.debugger.lock().unwrap().is_none());
        assert!(state.current_script.lock().unwrap().is_none());
        assert!(!*state.is_active.lock().unwrap());
    }

    #[test]
    fn test_breakpoint_info_serialization() {
        let info = BreakpointInfo {
            file: "test.py".to_string(),
            line: 42,
        };

        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test.py"));
        assert!(json.contains("42"));
    }
}
