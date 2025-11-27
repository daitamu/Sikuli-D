//! Tests for debug module
//! デバッグモジュールのテスト

use super::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_debugger_initial_state() {
    let debugger = Debugger::new();
    assert_eq!(debugger.get_state(), DebugState::NotStarted);
    assert_eq!(debugger.list_breakpoints().len(), 0);
}

#[test]
fn test_breakpoint_add_remove() {
    let debugger = Debugger::new();

    debugger.add_breakpoint("test.py", 10);
    assert!(debugger.has_breakpoint("test.py", 10));

    debugger.remove_breakpoint("test.py", 10);
    assert!(!debugger.has_breakpoint("test.py", 10));
}

#[test]
fn test_breakpoint_toggle() {
    let debugger = Debugger::new();

    debugger.toggle_breakpoint("test.py", 15);
    assert!(debugger.has_breakpoint("test.py", 15));

    debugger.toggle_breakpoint("test.py", 15);
    assert!(!debugger.has_breakpoint("test.py", 15));
}

#[test]
fn test_multiple_breakpoints() {
    let debugger = Debugger::new();

    debugger.add_breakpoint("main.py", 10);
    debugger.add_breakpoint("main.py", 20);
    debugger.add_breakpoint("utils.py", 5);

    let breakpoints = debugger.list_breakpoints();
    assert_eq!(breakpoints.len(), 3);
    assert!(breakpoints.contains(&("main.py".to_string(), 10)));
    assert!(breakpoints.contains(&("main.py".to_string(), 20)));
    assert!(breakpoints.contains(&("utils.py".to_string(), 5)));
}

#[test]
fn test_clear_all_breakpoints() {
    let debugger = Debugger::new();

    debugger.add_breakpoint("test1.py", 10);
    debugger.add_breakpoint("test2.py", 20);
    debugger.add_breakpoint("test3.py", 30);

    assert_eq!(debugger.list_breakpoints().len(), 3);

    debugger.clear_all_breakpoints();
    assert_eq!(debugger.list_breakpoints().len(), 0);
}

#[test]
fn test_state_transitions() {
    let debugger = Debugger::new();

    debugger.pause().unwrap();
    assert_eq!(debugger.get_state(), DebugState::Paused);

    debugger.resume().unwrap();
    assert_eq!(debugger.get_state(), DebugState::Running);

    debugger.step_over().unwrap();
    assert_eq!(debugger.get_state(), DebugState::StepOver);

    debugger.step_into().unwrap();
    assert_eq!(debugger.get_state(), DebugState::StepInto);

    debugger.step_out().unwrap();
    assert_eq!(debugger.get_state(), DebugState::StepOut);

    debugger.stop().unwrap();
    assert_eq!(debugger.get_state(), DebugState::Stopped);
}

#[test]
fn test_current_position() {
    let debugger = Debugger::new();

    let file = PathBuf::from("test.py");
    debugger.set_current_position(file.clone(), 42);

    let (pos_file, pos_line) = debugger.get_current_position();
    assert_eq!(pos_file, Some(file));
    assert_eq!(pos_line, Some(42));
}

#[test]
fn test_call_stack() {
    let debugger = Debugger::new();

    let frame1 = CallFrame::new(
        0,
        "main".to_string(),
        PathBuf::from("main.py"),
        10,
    );
    let frame2 = CallFrame::new(
        1,
        "helper_function".to_string(),
        PathBuf::from("utils.py"),
        25,
    );

    debugger.push_frame(frame1.clone());
    debugger.push_frame(frame2.clone());

    let stack = debugger.get_call_stack();
    assert_eq!(stack.len(), 2);
    assert_eq!(stack[0].function, "main");
    assert_eq!(stack[1].function, "helper_function");

    let popped = debugger.pop_frame();
    assert!(popped.is_some());
    assert_eq!(popped.unwrap().function, "helper_function");

    assert_eq!(debugger.get_call_stack().len(), 1);
}

#[test]
fn test_local_variables() {
    let debugger = Debugger::new();

    let mut frame = CallFrame::new(
        0,
        "test_func".to_string(),
        PathBuf::from("test.py"),
        10,
    );
    frame.add_local("x".to_string(), VariableValue::Int(42));
    frame.add_local("name".to_string(), VariableValue::String("Alice".to_string()));
    frame.add_local("flag".to_string(), VariableValue::Bool(true));

    debugger.push_frame(frame);

    let locals = debugger.get_variables(Scope::Local);
    assert_eq!(locals.len(), 3);
}

#[test]
fn test_global_variables() {
    let debugger = Debugger::new();

    debugger.update_global("PI".to_string(), VariableValue::Float(3.14159));
    debugger.update_global("DEBUG".to_string(), VariableValue::Bool(true));

    let globals = debugger.get_variables(Scope::Global);
    assert_eq!(globals.len(), 2);
}

#[test]
fn test_all_variables() {
    let debugger = Debugger::new();

    // Add local variables
    let mut frame = CallFrame::new(
        0,
        "main".to_string(),
        PathBuf::from("test.py"),
        5,
    );
    frame.add_local("local_var".to_string(), VariableValue::Int(100));
    debugger.push_frame(frame);

    // Add global variables
    debugger.update_global("global_var".to_string(), VariableValue::String("test".to_string()));

    let all_vars = debugger.get_variables(Scope::All);
    assert_eq!(all_vars.len(), 2);
}

#[test]
fn test_variable_value_display() {
    assert_eq!(format!("{}", VariableValue::Int(42)), "42");
    assert_eq!(format!("{}", VariableValue::Float(3.14)), "3.14");
    assert_eq!(
        format!("{}", VariableValue::String("hello".to_string())),
        "\"hello\""
    );
    assert_eq!(format!("{}", VariableValue::Bool(true)), "true");
    assert_eq!(format!("{}", VariableValue::None), "None");
}

#[test]
fn test_variable_value_list() {
    let list = VariableValue::List(vec![
        VariableValue::Int(1),
        VariableValue::Int(2),
        VariableValue::Int(3),
    ]);
    assert_eq!(format!("{}", list), "[1, 2, 3]");
}

#[test]
fn test_variable_value_dict() {
    use std::collections::HashMap;
    let mut map = HashMap::new();
    map.insert("key1".to_string(), VariableValue::Int(10));
    map.insert("key2".to_string(), VariableValue::String("value".to_string()));
    let dict = VariableValue::Dict(map);

    let s = format!("{}", dict);
    assert!(s.contains("key1"));
    assert!(s.contains("10"));
}

#[test]
fn test_event_callback() {
    let debugger = Debugger::new();
    let call_count = Arc::new(AtomicUsize::new(0));
    let call_count_clone = call_count.clone();

    debugger.register_callback(move |event| {
        if matches!(event, DebugEvent::Paused { .. }) {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    debugger.set_current_position(PathBuf::from("test.py"), 10);
    debugger.pause().unwrap();

    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[test]
fn test_multiple_event_callbacks() {
    let debugger = Debugger::new();
    let count1 = Arc::new(AtomicUsize::new(0));
    let count2 = Arc::new(AtomicUsize::new(0));

    let count1_clone = count1.clone();
    debugger.register_callback(move |_event| {
        count1_clone.fetch_add(1, Ordering::SeqCst);
    });

    let count2_clone = count2.clone();
    debugger.register_callback(move |_event| {
        count2_clone.fetch_add(1, Ordering::SeqCst);
    });

    debugger.set_current_position(PathBuf::from("test.py"), 10);
    debugger.pause().unwrap();

    assert_eq!(count1.load(Ordering::SeqCst), 1);
    assert_eq!(count2.load(Ordering::SeqCst), 1);
}

#[test]
fn test_breakpoint_hit_notification() {
    let debugger = Debugger::new();
    let hit = Arc::new(AtomicUsize::new(0));
    let hit_clone = hit.clone();

    debugger.register_callback(move |event| {
        if matches!(event, DebugEvent::BreakpointHit { .. }) {
            hit_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    debugger.notify_breakpoint_hit(PathBuf::from("test.py"), 10, 1);

    assert_eq!(hit.load(Ordering::SeqCst), 1);
    assert_eq!(debugger.get_state(), DebugState::Paused);
}

#[test]
fn test_reset() {
    let debugger = Debugger::new();

    // Set up some state
    debugger.add_breakpoint("test.py", 10);
    debugger.set_current_position(PathBuf::from("test.py"), 10);
    let frame = CallFrame::new(
        0,
        "main".to_string(),
        PathBuf::from("test.py"),
        10,
    );
    debugger.push_frame(frame);
    debugger.update_global("var".to_string(), VariableValue::Int(42));
    debugger.pause().unwrap();

    // Reset
    debugger.reset();

    // Verify reset state
    assert_eq!(debugger.get_state(), DebugState::NotStarted);
    assert_eq!(debugger.get_call_stack().len(), 0);
    assert_eq!(debugger.get_variables(Scope::Global).len(), 0);
    let (file, line) = debugger.get_current_position();
    assert!(file.is_none());
    assert!(line.is_none());

    // Breakpoints should remain after reset
    assert!(debugger.has_breakpoint("test.py", 10));
}

#[test]
fn test_debug_state_display() {
    assert_eq!(format!("{}", DebugState::NotStarted), "Not Started");
    assert_eq!(format!("{}", DebugState::Running), "Running");
    assert_eq!(format!("{}", DebugState::Paused), "Paused");
    assert_eq!(format!("{}", DebugState::StepOver), "Step Over");
    assert_eq!(format!("{}", DebugState::StepInto), "Step Into");
    assert_eq!(format!("{}", DebugState::StepOut), "Step Out");
    assert_eq!(format!("{}", DebugState::Stopped), "Stopped");
    assert_eq!(format!("{}", DebugState::Error), "Error");
}

#[test]
fn test_evaluate_expression_simple_variable() {
    let debugger = Debugger::new();

    let mut frame = CallFrame::new(
        0,
        "main".to_string(),
        PathBuf::from("test.py"),
        5,
    );
    frame.add_local("x".to_string(), VariableValue::Int(42));
    debugger.push_frame(frame);

    let result = debugger.evaluate_expression("x");
    assert!(result.is_ok());
    assert!(matches!(result.unwrap(), VariableValue::Int(42)));
}

#[test]
fn test_evaluate_expression_global_variable() {
    let debugger = Debugger::new();

    debugger.update_global("PI".to_string(), VariableValue::Float(3.14));

    let result = debugger.evaluate_expression("PI");
    assert!(result.is_ok());
}

#[test]
fn test_evaluate_expression_not_found() {
    let debugger = Debugger::new();

    let result = debugger.evaluate_expression("nonexistent_var");
    assert!(result.is_err());
}
