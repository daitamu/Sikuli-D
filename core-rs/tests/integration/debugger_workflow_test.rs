//! Debugger workflow integration tests
//! デバッガワークフロー統合テスト
//!
//! Tests the debugger functionality including breakpoints, stepping, and variable inspection.
//! ブレークポイント、ステップ実行、変数インスペクションなどのデバッガ機能をテストします。

use sikulix_core::debug::{
    Debugger, DebugState, DebugEvent, CallFrame, VariableInfo, VariableValue,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

#[test]
fn test_debugger_creation() {
    // Test basic debugger creation
    // 基本的なデバッガ作成をテスト
    let debugger = Debugger::new();

    assert_eq!(debugger.get_state(), DebugState::Idle);
    assert_eq!(debugger.breakpoint_count(), 0);
}

#[test]
fn test_breakpoint_management() {
    // Test adding and removing breakpoints
    // ブレークポイントの追加と削除をテスト
    let mut debugger = Debugger::new();

    // Add breakpoints
    // ブレークポイントを追加
    debugger.add_breakpoint("test.py", 10);
    debugger.add_breakpoint("test.py", 20);
    debugger.add_breakpoint("main.py", 5);

    assert_eq!(debugger.breakpoint_count(), 3);

    // Check if specific breakpoint exists
    // 特定のブレークポイントが存在するか確認
    assert!(debugger.has_breakpoint("test.py", 10));
    assert!(debugger.has_breakpoint("test.py", 20));
    assert!(debugger.has_breakpoint("main.py", 5));
    assert!(!debugger.has_breakpoint("test.py", 15));

    // Remove breakpoint
    // ブレークポイントを削除
    debugger.remove_breakpoint("test.py", 10);
    assert_eq!(debugger.breakpoint_count(), 2);
    assert!(!debugger.has_breakpoint("test.py", 10));
}

#[test]
fn test_conditional_breakpoints() {
    // Test conditional breakpoints
    // 条件付きブレークポイントをテスト
    let mut debugger = Debugger::new();

    // Add conditional breakpoint
    // 条件付きブレークポイントを追加
    debugger.add_conditional_breakpoint("test.py", 15, "x > 10");

    assert!(debugger.has_breakpoint("test.py", 15));

    // Get breakpoint condition
    // ブレークポイント条件を取得
    if let Some(condition) = debugger.get_breakpoint_condition("test.py", 15) {
        assert_eq!(condition, "x > 10");
    } else {
        panic!("Conditional breakpoint should exist");
    }
}

#[test]
fn test_debugger_state_transitions() {
    // Test debugger state transitions
    // デバッガ状態遷移をテスト
    let mut debugger = Debugger::new();

    assert_eq!(debugger.get_state(), DebugState::Idle);

    // Start debugging session
    // デバッグセッションを開始
    debugger.start_session("test.py").unwrap();
    assert_eq!(debugger.get_state(), DebugState::Running);

    // Pause execution
    // 実行を一時停止
    debugger.pause();
    assert_eq!(debugger.get_state(), DebugState::Paused);

    // Resume execution
    // 実行を再開
    debugger.resume();
    assert_eq!(debugger.get_state(), DebugState::Running);

    // Stop session
    // セッションを停止
    debugger.stop();
    assert_eq!(debugger.get_state(), DebugState::Idle);
}

#[test]
fn test_step_operations() {
    // Test step over, step into, step out
    // ステップオーバー、ステップイン、ステップアウトをテスト
    let mut debugger = Debugger::new();

    debugger.start_session("test.py").unwrap();
    debugger.pause();

    // Step over
    // ステップオーバー
    debugger.step_over();
    assert_eq!(debugger.get_state(), DebugState::Paused);

    // Step into
    // ステップイン
    debugger.step_into();
    assert_eq!(debugger.get_state(), DebugState::Paused);

    // Step out
    // ステップアウト
    debugger.step_out();
    assert_eq!(debugger.get_state(), DebugState::Paused);

    debugger.stop();
}

#[test]
fn test_call_stack_tracking() {
    // Test call stack tracking
    // コールスタック追跡をテスト
    let mut debugger = Debugger::new();

    debugger.start_session("test.py").unwrap();

    // Simulate entering functions
    // 関数への入場をシミュレート
    let frame1 = CallFrame {
        function_name: "main".to_string(),
        file_path: "test.py".to_string(),
        line_number: 10,
        scope_variables: HashMap::new(),
    };

    let frame2 = CallFrame {
        function_name: "process_data".to_string(),
        file_path: "test.py".to_string(),
        line_number: 25,
        scope_variables: HashMap::new(),
    };

    debugger.push_call_frame(frame1);
    debugger.push_call_frame(frame2);

    let stack = debugger.get_call_stack();
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0].function_name, "main");
    assert_eq!(stack[1].function_name, "process_data");

    // Pop frame
    // フレームをポップ
    debugger.pop_call_frame();
    assert_eq!(debugger.get_call_stack().len(), 1);

    debugger.stop();
}

#[test]
fn test_variable_inspection() {
    // Test variable inspection at breakpoint
    // ブレークポイントでの変数インスペクションをテスト
    let mut debugger = Debugger::new();

    debugger.start_session("test.py").unwrap();
    debugger.pause();

    // Set some variables
    // 変数を設定
    let mut variables = HashMap::new();
    variables.insert(
        "x".to_string(),
        VariableInfo {
            name: "x".to_string(),
            value: VariableValue::Integer(42),
            var_type: "int".to_string(),
        },
    );
    variables.insert(
        "name".to_string(),
        VariableInfo {
            name: "name".to_string(),
            value: VariableValue::String("Test".to_string()),
            var_type: "str".to_string(),
        },
    );

    debugger.set_local_variables(variables.clone());

    // Inspect variables
    // 変数をインスペクト
    let locals = debugger.get_local_variables();
    assert_eq!(locals.len(), 2);
    assert!(locals.contains_key("x"));
    assert!(locals.contains_key("name"));

    if let Some(var) = locals.get("x") {
        assert_eq!(var.var_type, "int");
        if let VariableValue::Integer(val) = var.value {
            assert_eq!(val, 42);
        }
    }

    debugger.stop();
}

#[test]
fn test_debug_event_notifications() {
    // Test debug event notification system
    // デバッグイベント通知システムをテスト
    let debugger = Debugger::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();

    // Register event handler
    // イベントハンドラを登録
    debugger.on_event(move |event| {
        events_clone.lock().unwrap().push(event);
    });

    // Trigger various events
    // 様々なイベントをトリガー
    debugger.trigger_event(DebugEvent::SessionStarted {
        script_path: "test.py".to_string(),
    });

    debugger.trigger_event(DebugEvent::BreakpointHit {
        file_path: "test.py".to_string(),
        line_number: 10,
    });

    debugger.trigger_event(DebugEvent::StepCompleted {
        file_path: "test.py".to_string(),
        line_number: 11,
    });

    // Verify events were received
    // イベントが受信されたことを確認
    let received_events = events.lock().unwrap();
    assert!(received_events.len() >= 3);
}

#[test]
fn test_breakpoint_hit_workflow() {
    // Test complete workflow when breakpoint is hit
    // ブレークポイントヒット時の完全なワークフローをテスト
    let mut debugger = Debugger::new();

    // Set up breakpoint
    // ブレークポイントを設定
    debugger.add_breakpoint("test.py", 15);

    // Start session
    // セッションを開始
    debugger.start_session("test.py").unwrap();
    assert_eq!(debugger.get_state(), DebugState::Running);

    // Simulate hitting breakpoint
    // ブレークポイントヒットをシミュレート
    if debugger.has_breakpoint("test.py", 15) {
        debugger.pause();
        debugger.set_current_location("test.py", 15);

        assert_eq!(debugger.get_state(), DebugState::Paused);

        let (file, line) = debugger.get_current_location();
        assert_eq!(file, "test.py");
        assert_eq!(line, 15);

        // Inspect variables at this point
        // この時点で変数をインスペクト
        let locals = debugger.get_local_variables();
        println!("Variables at breakpoint: {:?}", locals);

        // Continue execution
        // 実行を継続
        debugger.resume();
        assert_eq!(debugger.get_state(), DebugState::Running);
    }

    debugger.stop();
}

#[test]
fn test_multiple_sessions() {
    // Test that debugger can handle multiple debug sessions
    // デバッガが複数のデバッグセッションを処理できることをテスト
    let mut debugger = Debugger::new();

    // First session
    // 最初のセッション
    debugger.start_session("test1.py").unwrap();
    debugger.add_breakpoint("test1.py", 10);
    debugger.stop();

    assert_eq!(debugger.get_state(), DebugState::Idle);

    // Second session
    // 2番目のセッション
    debugger.start_session("test2.py").unwrap();
    debugger.add_breakpoint("test2.py", 20);
    debugger.stop();

    assert_eq!(debugger.get_state(), DebugState::Idle);
}

#[test]
fn test_breakpoint_enable_disable() {
    // Test enabling and disabling breakpoints
    // ブレークポイントの有効化と無効化をテスト
    let mut debugger = Debugger::new();

    debugger.add_breakpoint("test.py", 10);
    assert!(debugger.is_breakpoint_enabled("test.py", 10));

    // Disable breakpoint
    // ブレークポイントを無効化
    debugger.disable_breakpoint("test.py", 10);
    assert!(!debugger.is_breakpoint_enabled("test.py", 10));

    // Enable breakpoint
    // ブレークポイントを有効化
    debugger.enable_breakpoint("test.py", 10);
    assert!(debugger.is_breakpoint_enabled("test.py", 10));
}

#[test]
fn test_clear_all_breakpoints() {
    // Test clearing all breakpoints
    // すべてのブレークポイントをクリアするテスト
    let mut debugger = Debugger::new();

    debugger.add_breakpoint("test.py", 10);
    debugger.add_breakpoint("test.py", 20);
    debugger.add_breakpoint("main.py", 5);

    assert_eq!(debugger.breakpoint_count(), 3);

    debugger.clear_all_breakpoints();

    assert_eq!(debugger.breakpoint_count(), 0);
}

#[test]
fn test_variable_value_types() {
    // Test different variable value types
    // 異なる変数値タイプをテスト
    let values = vec![
        VariableValue::Integer(42),
        VariableValue::Float(3.14),
        VariableValue::String("test".to_string()),
        VariableValue::Boolean(true),
        VariableValue::None,
        VariableValue::List(vec![
            VariableValue::Integer(1),
            VariableValue::Integer(2),
        ]),
        VariableValue::Dict(HashMap::new()),
    ];

    for value in values {
        let var_info = VariableInfo {
            name: "test_var".to_string(),
            value: value.clone(),
            var_type: format!("{:?}", value),
        };

        assert_eq!(var_info.name, "test_var");
    }
}

#[test]
fn test_debugger_error_handling() {
    // Test error handling in debugger
    // デバッガのエラーハンドリングをテスト
    let mut debugger = Debugger::new();

    // Try to pause when not running
    // 実行していない時に一時停止を試行
    debugger.pause();
    // Should handle gracefully
    // 正常に処理されるべき

    // Try to resume when not paused
    // 一時停止していない時に再開を試行
    debugger.resume();
    // Should handle gracefully
    // 正常に処理されるべき

    // Try to step when not in debug mode
    // デバッグモードでない時にステップを試行
    debugger.step_over();
    // Should handle gracefully
    // 正常に処理されるべき
}

#[test]
#[ignore = "Requires actual Python script execution"]
fn test_real_debugging_session() -> sikulix_core::Result<()> {
    // Test real debugging session with Python script
    // Pythonスクリプトで実際のデバッグセッションをテスト
    let mut debugger = Debugger::new();

    let script = r#"
def add(a, b):
    result = a + b
    return result

x = 5
y = 10
z = add(x, y)
print(z)
"#;

    // Set breakpoint on line 3 (inside add function)
    // 3行目（add関数内）にブレークポイントを設定
    debugger.add_breakpoint("test.py", 3);

    // Start debugging
    // デバッグを開始
    debugger.start_session("test.py")?;

    // Execute script until breakpoint
    // ブレークポイントまでスクリプトを実行
    // (This would require Python integration)
    // (これにはPython統合が必要)

    debugger.stop();

    Ok(())
}
