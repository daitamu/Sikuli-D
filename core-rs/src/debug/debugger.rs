//! Debugger implementation for script debugging
//! スクリプトデバッグのためのデバッガ実装
//!
//! Provides comprehensive debugging capabilities including:
//! 以下を含む包括的なデバッグ機能を提供します:
//! - Breakpoint management with conditions / 条件付きブレークポイント管理
//! - Execution control (pause, resume, step) / 実行制御（一時停止、再開、ステップ）
//! - Variable inspection / 変数インスペクション
//! - Call stack tracking / コールスタック追跡
//! - Expression evaluation / 式評価

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::SikulixError;
use log::{debug, info, warn};

/// Debug state representing the current execution state
/// 現在の実行状態を表すデバッグ状態
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugState {
    /// Not started / 未開始
    NotStarted,
    /// Running normally / 通常実行中
    Running,
    /// Paused at breakpoint or by user / ブレークポイントまたはユーザーによって一時停止
    Paused,
    /// Stepping over current line / 現在の行をステップオーバー中
    StepOver,
    /// Stepping into function / 関数にステップイン中
    StepInto,
    /// Stepping out of function / 関数からステップアウト中
    StepOut,
    /// Stopped / 停止
    Stopped,
    /// Error occurred / エラー発生
    Error,
}

impl std::fmt::Display for DebugState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DebugState::NotStarted => write!(f, "Not Started"),
            DebugState::Running => write!(f, "Running"),
            DebugState::Paused => write!(f, "Paused"),
            DebugState::StepOver => write!(f, "Step Over"),
            DebugState::StepInto => write!(f, "Step Into"),
            DebugState::StepOut => write!(f, "Step Out"),
            DebugState::Stopped => write!(f, "Stopped"),
            DebugState::Error => write!(f, "Error"),
        }
    }
}

/// Variable scope for inspection
/// インスペクション用の変数スコープ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// Local variables in current frame / 現在のフレームのローカル変数
    Local,
    /// Global variables / グローバル変数
    Global,
    /// All variables (local + global) / すべての変数（ローカル + グローバル）
    All,
}

/// Variable value types
/// 変数値の型
#[derive(Debug, Clone, PartialEq)]
pub enum VariableValue {
    /// Integer / 整数
    Int(i64),
    /// Float / 浮動小数点数
    Float(f64),
    /// String / 文字列
    String(String),
    /// Boolean / ブール値
    Bool(bool),
    /// None / None
    None,
    /// List of values / 値のリスト
    List(Vec<VariableValue>),
    /// Dictionary / 辞書
    Dict(HashMap<String, VariableValue>),
    /// Object with type name / 型名を持つオブジェクト
    Object(String),
    /// Unknown type / 不明な型
    Unknown(String),
}

impl std::fmt::Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::Int(v) => write!(f, "{}", v),
            VariableValue::Float(v) => write!(f, "{}", v),
            VariableValue::String(v) => write!(f, "\"{}\"", v),
            VariableValue::Bool(v) => write!(f, "{}", v),
            VariableValue::None => write!(f, "None"),
            VariableValue::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            VariableValue::Dict(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", k, v)?;
                }
                write!(f, "}}")
            }
            VariableValue::Object(type_name) => write!(f, "<{} object>", type_name),
            VariableValue::Unknown(repr) => write!(f, "{}", repr),
        }
    }
}

/// Variable information for debugging
/// デバッグ用変数情報
#[derive(Debug, Clone)]
pub struct VariableInfo {
    /// Variable name / 変数名
    pub name: String,
    /// Variable value / 変数値
    pub value: VariableValue,
    /// Type name / 型名
    pub type_name: String,
    /// Scope / スコープ
    pub scope: Scope,
}

impl VariableInfo {
    /// Create a new variable info
    /// 新しい変数情報を作成
    pub fn new(name: String, value: VariableValue, type_name: String, scope: Scope) -> Self {
        Self {
            name,
            value,
            type_name,
            scope,
        }
    }
}

/// Call stack frame
/// コールスタックフレーム
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Frame depth (0 = current) / フレーム深度（0 = 現在）
    pub depth: usize,
    /// Function or method name / 関数またはメソッド名
    pub function: String,
    /// File path / ファイルパス
    pub file: PathBuf,
    /// Line number / 行番号
    pub line: u32,
    /// Local variables / ローカル変数
    pub locals: HashMap<String, VariableValue>,
}

impl CallFrame {
    /// Create a new call frame
    /// 新しいコールフレームを作成
    pub fn new(depth: usize, function: String, file: PathBuf, line: u32) -> Self {
        Self {
            depth,
            function,
            file,
            line,
            locals: HashMap::new(),
        }
    }

    /// Add a local variable
    /// ローカル変数を追加
    pub fn add_local(&mut self, name: String, value: VariableValue) {
        self.locals.insert(name, value);
    }

    /// Get local variable
    /// ローカル変数を取得
    pub fn get_local(&self, name: &str) -> Option<&VariableValue> {
        self.locals.get(name)
    }
}

/// Debug event for notifications
/// 通知用デバッグイベント
#[derive(Debug, Clone)]
pub enum DebugEvent {
    /// Breakpoint was hit / ブレークポイントがヒット
    BreakpointHit {
        file: PathBuf,
        line: u32,
        hit_count: u32,
    },
    /// Execution paused / 実行が一時停止
    Paused { file: PathBuf, line: u32 },
    /// Execution resumed / 実行が再開
    Resumed,
    /// Step completed / ステップ完了
    StepCompleted { file: PathBuf, line: u32 },
    /// Execution stopped / 実行が停止
    Stopped,
    /// Error occurred / エラー発生
    Error { message: String },
    /// Variable changed / 変数が変更された
    VariableChanged { name: String, value: VariableValue },
}

/// Event callback type
/// イベントコールバック型
pub type EventCallback = Arc<dyn Fn(DebugEvent) + Send + Sync>;

/// Debugger for script debugging
/// スクリプトデバッグ用デバッガ
pub struct Debugger {
    /// Current debug state / 現在のデバッグ状態
    state: Arc<Mutex<DebugState>>,

    /// Breakpoints by file / ファイルごとのブレークポイント
    breakpoints: Arc<Mutex<HashMap<String, Vec<u32>>>>,

    /// Current execution location / 現在の実行位置
    current_file: Arc<Mutex<Option<PathBuf>>>,
    current_line: Arc<Mutex<Option<u32>>>,

    /// Call stack / コールスタック
    call_stack: Arc<Mutex<Vec<CallFrame>>>,

    /// Global variables / グローバル変数
    global_variables: Arc<Mutex<HashMap<String, VariableValue>>>,

    /// Event callbacks / イベントコールバック
    event_callbacks: Arc<Mutex<Vec<EventCallback>>>,
}

impl Default for Debugger {
    fn default() -> Self {
        Self::new()
    }
}

impl Debugger {
    /// Create a new debugger
    /// 新しいデバッガを作成
    pub fn new() -> Self {
        info!("Creating new Debugger instance");
        Self {
            state: Arc::new(Mutex::new(DebugState::NotStarted)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            current_file: Arc::new(Mutex::new(None)),
            current_line: Arc::new(Mutex::new(None)),
            call_stack: Arc::new(Mutex::new(Vec::new())),
            global_variables: Arc::new(Mutex::new(HashMap::new())),
            event_callbacks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    // ========================================================================
    // State Management / 状態管理
    // ========================================================================

    /// Get current debug state
    /// 現在のデバッグ状態を取得
    pub fn get_state(&self) -> DebugState {
        *self.state.lock().unwrap()
    }

    /// Set debug state
    /// デバッグ状態を設定
    fn set_state(&self, new_state: DebugState) {
        let old_state = *self.state.lock().unwrap();
        if old_state != new_state {
            debug!("Debug state changed: {} -> {}", old_state, new_state);
            *self.state.lock().unwrap() = new_state;
        }
    }

    /// Get current position (file, line)
    /// 現在の位置を取得（ファイル、行）
    pub fn get_current_position(&self) -> (Option<PathBuf>, Option<u32>) {
        (
            self.current_file.lock().unwrap().clone(),
            *self.current_line.lock().unwrap(),
        )
    }

    /// Set current position
    /// 現在の位置を設定
    pub fn set_current_position(&self, file: PathBuf, line: u32) {
        *self.current_file.lock().unwrap() = Some(file);
        *self.current_line.lock().unwrap() = Some(line);
    }

    // ========================================================================
    // Breakpoint Management / ブレークポイント管理
    // ========================================================================

    /// Add a breakpoint at the specified file and line
    /// 指定されたファイルと行にブレークポイントを追加
    pub fn add_breakpoint(&self, file: &str, line: u32) {
        info!("Adding breakpoint at {}:{}", file, line);
        let mut breakpoints = self.breakpoints.lock().unwrap();
        breakpoints
            .entry(file.to_string())
            .or_insert_with(Vec::new)
            .push(line);
    }

    /// Remove a breakpoint from the specified file and line
    /// 指定されたファイルと行からブレークポイントを削除
    pub fn remove_breakpoint(&self, file: &str, line: u32) {
        info!("Removing breakpoint at {}:{}", file, line);
        let mut breakpoints = self.breakpoints.lock().unwrap();
        if let Some(lines) = breakpoints.get_mut(file) {
            lines.retain(|&l| l != line);
            if lines.is_empty() {
                breakpoints.remove(file);
            }
        }
    }

    /// Toggle breakpoint (add if not exists, remove if exists)
    /// ブレークポイントを切り替え（存在しない場合は追加、存在する場合は削除）
    pub fn toggle_breakpoint(&self, file: &str, line: u32) {
        let has_bp = {
            let breakpoints = self.breakpoints.lock().unwrap();
            breakpoints
                .get(file)
                .map(|lines| lines.contains(&line))
                .unwrap_or(false)
        };

        if has_bp {
            self.remove_breakpoint(file, line);
        } else {
            self.add_breakpoint(file, line);
        }
    }

    /// List all breakpoints
    /// すべてのブレークポイントをリスト
    pub fn list_breakpoints(&self) -> Vec<(String, u32)> {
        let breakpoints = self.breakpoints.lock().unwrap();
        let mut result = Vec::new();
        for (file, lines) in breakpoints.iter() {
            for &line in lines {
                result.push((file.clone(), line));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));
        result
    }

    /// Clear all breakpoints
    /// すべてのブレークポイントをクリア
    pub fn clear_all_breakpoints(&self) {
        info!("Clearing all breakpoints");
        self.breakpoints.lock().unwrap().clear();
    }

    /// Check if there's a breakpoint at the given location
    /// 指定された場所にブレークポイントがあるかチェック
    pub fn has_breakpoint(&self, file: &str, line: u32) -> bool {
        let breakpoints = self.breakpoints.lock().unwrap();
        breakpoints
            .get(file)
            .map(|lines| lines.contains(&line))
            .unwrap_or(false)
    }

    // ========================================================================
    // Execution Control / 実行制御
    // ========================================================================

    /// Pause execution
    /// 実行を一時停止
    pub fn pause(&self) -> Result<(), SikulixError> {
        info!("Pausing execution");
        self.set_state(DebugState::Paused);

        if let (Some(file), Some(line)) = self.get_current_position() {
            self.notify_event(DebugEvent::Paused { file, line });
        }

        Ok(())
    }

    /// Resume execution
    /// 実行を再開
    pub fn resume(&self) -> Result<(), SikulixError> {
        info!("Resuming execution");
        self.set_state(DebugState::Running);
        self.notify_event(DebugEvent::Resumed);
        Ok(())
    }

    /// Step over the current line
    /// 現在の行をステップオーバー
    pub fn step_over(&self) -> Result<(), SikulixError> {
        info!("Step over");
        self.set_state(DebugState::StepOver);

        if let (Some(file), Some(line)) = self.get_current_position() {
            self.notify_event(DebugEvent::StepCompleted { file, line });
        }

        Ok(())
    }

    /// Step into a function
    /// 関数にステップイン
    pub fn step_into(&self) -> Result<(), SikulixError> {
        info!("Step into");
        self.set_state(DebugState::StepInto);

        if let (Some(file), Some(line)) = self.get_current_position() {
            self.notify_event(DebugEvent::StepCompleted { file, line });
        }

        Ok(())
    }

    /// Step out of the current function
    /// 現在の関数からステップアウト
    pub fn step_out(&self) -> Result<(), SikulixError> {
        info!("Step out");
        self.set_state(DebugState::StepOut);

        if let (Some(file), Some(line)) = self.get_current_position() {
            self.notify_event(DebugEvent::StepCompleted { file, line });
        }

        Ok(())
    }

    /// Stop execution
    /// 実行を停止
    pub fn stop(&self) -> Result<(), SikulixError> {
        info!("Stopping execution");
        self.set_state(DebugState::Stopped);
        self.notify_event(DebugEvent::Stopped);
        self.call_stack.lock().unwrap().clear();
        Ok(())
    }

    // ========================================================================
    // State Inspection / 状態検査
    // ========================================================================

    /// Get call stack
    /// コールスタックを取得
    pub fn get_call_stack(&self) -> Vec<CallFrame> {
        self.call_stack.lock().unwrap().clone()
    }

    /// Get variables in the specified scope
    /// 指定されたスコープの変数を取得
    pub fn get_variables(&self, scope: Scope) -> Vec<VariableInfo> {
        let mut result = Vec::new();

        match scope {
            Scope::Local => {
                // Get local variables from current frame
                // 現在のフレームからローカル変数を取得
                if let Some(frame) = self.call_stack.lock().unwrap().first() {
                    for (name, value) in &frame.locals {
                        result.push(VariableInfo::new(
                            name.clone(),
                            value.clone(),
                            Self::get_type_name(value),
                            Scope::Local,
                        ));
                    }
                }
            }
            Scope::Global => {
                // Get global variables
                // グローバル変数を取得
                let globals = self.global_variables.lock().unwrap();
                for (name, value) in globals.iter() {
                    result.push(VariableInfo::new(
                        name.clone(),
                        value.clone(),
                        Self::get_type_name(value),
                        Scope::Global,
                    ));
                }
            }
            Scope::All => {
                // Get both local and global
                // ローカルとグローバル両方を取得
                result.extend(self.get_variables(Scope::Local));
                result.extend(self.get_variables(Scope::Global));
            }
        }

        result
    }

    /// Evaluate an expression in the current context
    /// 現在のコンテキストで式を評価
    pub fn evaluate_expression(&self, expr: &str) -> Result<VariableValue, SikulixError> {
        debug!("Evaluating expression: {}", expr);

        // Check if it's a variable name
        // 変数名かどうかチェック
        if let Some(frame) = self.call_stack.lock().unwrap().first() {
            if let Some(value) = frame.locals.get(expr) {
                return Ok(value.clone());
            }
        }

        let globals = self.global_variables.lock().unwrap();
        if let Some(value) = globals.get(expr) {
            return Ok(value.clone());
        }

        warn!("Expression evaluation not fully implemented: {}", expr);
        Err(SikulixError::PythonError(format!(
            "Cannot evaluate expression: {}",
            expr
        )))
    }

    /// Get type name from variable value
    /// 変数値から型名を取得
    fn get_type_name(value: &VariableValue) -> String {
        match value {
            VariableValue::Int(_) => "int".to_string(),
            VariableValue::Float(_) => "float".to_string(),
            VariableValue::String(_) => "str".to_string(),
            VariableValue::Bool(_) => "bool".to_string(),
            VariableValue::None => "NoneType".to_string(),
            VariableValue::List(_) => "list".to_string(),
            VariableValue::Dict(_) => "dict".to_string(),
            VariableValue::Object(type_name) => type_name.clone(),
            VariableValue::Unknown(_) => "unknown".to_string(),
        }
    }

    // ========================================================================
    // Call Stack Management / コールスタック管理
    // ========================================================================

    /// Push a call frame onto the stack
    /// コールフレームをスタックにプッシュ
    pub fn push_frame(&self, frame: CallFrame) {
        debug!("Pushing frame: {}:{}", frame.function, frame.line);
        self.call_stack.lock().unwrap().push(frame);
    }

    /// Pop a call frame from the stack
    /// コールフレームをスタックからポップ
    pub fn pop_frame(&self) -> Option<CallFrame> {
        let frame = self.call_stack.lock().unwrap().pop();
        if let Some(ref f) = frame {
            debug!("Popping frame: {}:{}", f.function, f.line);
        }
        frame
    }

    /// Update local variable in current frame
    /// 現在のフレームのローカル変数を更新
    pub fn update_local(&self, name: String, value: VariableValue) {
        if let Some(frame) = self.call_stack.lock().unwrap().first_mut() {
            frame.add_local(name.clone(), value.clone());
            self.notify_event(DebugEvent::VariableChanged { name, value });
        }
    }

    /// Update global variable
    /// グローバル変数を更新
    pub fn update_global(&self, name: String, value: VariableValue) {
        self.global_variables
            .lock()
            .unwrap()
            .insert(name.clone(), value.clone());
        self.notify_event(DebugEvent::VariableChanged { name, value });
    }

    // ========================================================================
    // Event Notification / イベント通知
    // ========================================================================

    /// Register an event callback
    /// イベントコールバックを登録
    pub fn register_callback<F>(&self, callback: F)
    where
        F: Fn(DebugEvent) + Send + Sync + 'static,
    {
        self.event_callbacks
            .lock()
            .unwrap()
            .push(Arc::new(callback));
    }

    /// Notify all registered callbacks of an event
    /// 登録されたすべてのコールバックにイベントを通知
    fn notify_event(&self, event: DebugEvent) {
        let callbacks = self.event_callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(event.clone());
        }
    }

    /// Notify breakpoint hit
    /// ブレークポイントヒットを通知
    pub fn notify_breakpoint_hit(&self, file: PathBuf, line: u32, hit_count: u32) {
        info!("Breakpoint hit at {}:{} (count: {})", file.display(), line, hit_count);
        self.set_state(DebugState::Paused);
        self.notify_event(DebugEvent::BreakpointHit {
            file,
            line,
            hit_count,
        });
    }

    // ========================================================================
    // Utility / ユーティリティ
    // ========================================================================

    /// Reset debugger to initial state
    /// デバッガを初期状態にリセット
    pub fn reset(&self) {
        info!("Resetting debugger");
        self.set_state(DebugState::NotStarted);
        self.call_stack.lock().unwrap().clear();
        self.global_variables.lock().unwrap().clear();
        *self.current_file.lock().unwrap() = None;
        *self.current_line.lock().unwrap() = None;
    }
}

// Thread-safe implementation
// スレッドセーフな実装
unsafe impl Send for Debugger {}
unsafe impl Sync for Debugger {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_creation() {
        let dbg = Debugger::new();
        assert_eq!(dbg.get_state(), DebugState::NotStarted);
    }

    #[test]
    fn test_breakpoint_management() {
        let dbg = Debugger::new();

        dbg.add_breakpoint("test.py", 10);
        assert!(dbg.has_breakpoint("test.py", 10));
        assert!(!dbg.has_breakpoint("test.py", 11));

        dbg.remove_breakpoint("test.py", 10);
        assert!(!dbg.has_breakpoint("test.py", 10));
    }

    #[test]
    fn test_toggle_breakpoint() {
        let dbg = Debugger::new();

        dbg.toggle_breakpoint("test.py", 10);
        assert!(dbg.has_breakpoint("test.py", 10));

        dbg.toggle_breakpoint("test.py", 10);
        assert!(!dbg.has_breakpoint("test.py", 10));
    }

    #[test]
    fn test_list_breakpoints() {
        let dbg = Debugger::new();

        dbg.add_breakpoint("test.py", 10);
        dbg.add_breakpoint("test.py", 20);
        dbg.add_breakpoint("main.py", 5);

        let bps = dbg.list_breakpoints();
        assert_eq!(bps.len(), 3);
        assert!(bps.contains(&("test.py".to_string(), 10)));
        assert!(bps.contains(&("test.py".to_string(), 20)));
        assert!(bps.contains(&("main.py".to_string(), 5)));
    }

    #[test]
    fn test_clear_all_breakpoints() {
        let dbg = Debugger::new();

        dbg.add_breakpoint("test.py", 10);
        dbg.add_breakpoint("main.py", 5);

        dbg.clear_all_breakpoints();
        assert_eq!(dbg.list_breakpoints().len(), 0);
    }

    #[test]
    fn test_execution_control() {
        let dbg = Debugger::new();

        dbg.pause().unwrap();
        assert_eq!(dbg.get_state(), DebugState::Paused);

        dbg.resume().unwrap();
        assert_eq!(dbg.get_state(), DebugState::Running);

        dbg.step_over().unwrap();
        assert_eq!(dbg.get_state(), DebugState::StepOver);

        dbg.step_into().unwrap();
        assert_eq!(dbg.get_state(), DebugState::StepInto);

        dbg.step_out().unwrap();
        assert_eq!(dbg.get_state(), DebugState::StepOut);

        dbg.stop().unwrap();
        assert_eq!(dbg.get_state(), DebugState::Stopped);
    }

    #[test]
    fn test_call_stack() {
        let dbg = Debugger::new();

        let frame1 = CallFrame::new(
            0,
            "main".to_string(),
            PathBuf::from("test.py"),
            10,
        );
        let frame2 = CallFrame::new(
            1,
            "helper".to_string(),
            PathBuf::from("test.py"),
            20,
        );

        dbg.push_frame(frame1);
        dbg.push_frame(frame2);

        let stack = dbg.get_call_stack();
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].function, "main");
        assert_eq!(stack[1].function, "helper");

        dbg.pop_frame();
        assert_eq!(dbg.get_call_stack().len(), 1);
    }

    #[test]
    fn test_variables() {
        let dbg = Debugger::new();

        // Add local variable
        let mut frame = CallFrame::new(
            0,
            "main".to_string(),
            PathBuf::from("test.py"),
            10,
        );
        frame.add_local("x".to_string(), VariableValue::Int(42));
        frame.add_local("y".to_string(), VariableValue::String("hello".to_string()));

        dbg.push_frame(frame);

        // Add global variable
        dbg.update_global("global_var".to_string(), VariableValue::Float(3.14));

        // Get local variables
        let locals = dbg.get_variables(Scope::Local);
        assert_eq!(locals.len(), 2);

        // Get global variables
        let globals = dbg.get_variables(Scope::Global);
        assert_eq!(globals.len(), 1);

        // Get all variables
        let all = dbg.get_variables(Scope::All);
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_variable_value_display() {
        assert_eq!(format!("{}", VariableValue::Int(42)), "42");
        assert_eq!(format!("{}", VariableValue::Float(3.14)), "3.14");
        assert_eq!(format!("{}", VariableValue::String("hello".to_string())), "\"hello\"");
        assert_eq!(format!("{}", VariableValue::Bool(true)), "true");
        assert_eq!(format!("{}", VariableValue::None), "None");
    }

    #[test]
    fn test_reset() {
        let dbg = Debugger::new();

        dbg.add_breakpoint("test.py", 10);
        dbg.set_current_position(PathBuf::from("test.py"), 10);
        dbg.push_frame(CallFrame::new(
            0,
            "main".to_string(),
            PathBuf::from("test.py"),
            10,
        ));

        dbg.reset();

        assert_eq!(dbg.get_state(), DebugState::NotStarted);
        assert_eq!(dbg.get_call_stack().len(), 0);
        assert_eq!(dbg.get_variables(Scope::Global).len(), 0);
    }

    #[test]
    fn test_event_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let dbg = Debugger::new();
        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        dbg.register_callback(move |event| {
            if matches!(event, DebugEvent::Paused { .. }) {
                called_clone.store(true, Ordering::SeqCst);
            }
        });

        dbg.set_current_position(PathBuf::from("test.py"), 10);
        dbg.pause().unwrap();

        assert!(called.load(Ordering::SeqCst));
    }
}
