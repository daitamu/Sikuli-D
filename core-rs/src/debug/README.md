# Debug Infrastructure / デバッグ基盤

## Overview / 概要

This module provides comprehensive debugging capabilities for Sikuli-D script execution, including breakpoint management, execution control, variable inspection, and event notifications.

このモジュールは、ブレークポイント管理、実行制御、変数インスペクション、イベント通知を含む、Sikuli-Dスクリプト実行のための包括的なデバッグ機能を提供します。

## Architecture / アーキテクチャ

```
debug/
├── mod.rs              # Module exports / モジュールエクスポート
├── debugger.rs         # Core debugger implementation / コアデバッガ実装
├── highlight.rs        # Visual highlighting (existing) / ビジュアルハイライト（既存）
├── highlight_linux.rs  # Linux-specific highlight / Linux固有ハイライト
├── tests.rs            # Comprehensive tests / 包括的テスト
└── README.md           # This file / このファイル
```

## Core Components / コアコンポーネント

### Debugger

Thread-safe debugger implementation with:
スレッドセーフなデバッガ実装、以下を含む:

- **Breakpoint Management** / ブレークポイント管理
  - Add, remove, toggle breakpoints / ブレークポイントの追加、削除、切り替え
  - List all breakpoints / すべてのブレークポイントをリスト
  - Check if breakpoint exists / ブレークポイントが存在するかチェック
  - Clear all breakpoints / すべてのブレークポイントをクリア

- **Execution Control** / 実行制御
  - Pause execution / 実行を一時停止
  - Resume execution / 実行を再開
  - Step over (next line) / ステップオーバー（次の行）
  - Step into (enter function) / ステップイン（関数に入る）
  - Step out (exit function) / ステップアウト（関数から出る)
  - Stop execution / 実行を停止

- **State Inspection** / 状態検査
  - Get current position (file, line) / 現在の位置を取得（ファイル、行）
  - Get call stack / コールスタックを取得
  - Get variables by scope (local, global, all) / スコープ別に変数を取得
  - Evaluate expressions / 式を評価

- **Event Notification** / イベント通知
  - Register callbacks for debug events / デバッグイベントのコールバックを登録
  - Breakpoint hit notifications / ブレークポイントヒット通知
  - Pause/resume notifications / 一時停止/再開通知
  - Step completed notifications / ステップ完了通知
  - Variable changed notifications / 変数変更通知

## Types / 型

### DebugState

Represents the current execution state:
現在の実行状態を表します:

```rust
pub enum DebugState {
    NotStarted,  // Not started / 未開始
    Running,     // Running normally / 通常実行中
    Paused,      // Paused / 一時停止中
    StepOver,    // Stepping over / ステップオーバー中
    StepInto,    // Stepping into / ステップイン中
    StepOut,     // Stepping out / ステップアウト中
    Stopped,     // Stopped / 停止
    Error,       // Error occurred / エラー発生
}
```

### VariableValue

Represents variable values:
変数値を表します:

```rust
pub enum VariableValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
    List(Vec<VariableValue>),
    Dict(HashMap<String, VariableValue>),
    Object(String),       // Object with type name / 型名を持つオブジェクト
    Unknown(String),      // Unknown type / 不明な型
}
```

### CallFrame

Represents a call stack frame:
コールスタックフレームを表します:

```rust
pub struct CallFrame {
    pub depth: usize,              // Frame depth / フレーム深度
    pub function: String,          // Function name / 関数名
    pub file: PathBuf,             // File path / ファイルパス
    pub line: u32,                 // Line number / 行番号
    pub locals: HashMap<String, VariableValue>,  // Local variables / ローカル変数
}
```

### DebugEvent

Events emitted by the debugger:
デバッガが発行するイベント:

```rust
pub enum DebugEvent {
    BreakpointHit { file: PathBuf, line: u32, hit_count: u32 },
    Paused { file: PathBuf, line: u32 },
    Resumed,
    StepCompleted { file: PathBuf, line: u32 },
    Stopped,
    Error { message: String },
    VariableChanged { name: String, value: VariableValue },
}
```

## Usage Examples / 使用例

### Basic Breakpoint Management / 基本的なブレークポイント管理

```rust
use sikulix_core::debug::{Debugger, DebugState};

// Create debugger / デバッガを作成
let debugger = Debugger::new();

// Add breakpoints / ブレークポイントを追加
debugger.add_breakpoint("script.py", 10);
debugger.add_breakpoint("script.py", 25);

// Check if breakpoint exists / ブレークポイントが存在するかチェック
assert!(debugger.has_breakpoint("script.py", 10));

// Toggle breakpoint / ブレークポイントを切り替え
debugger.toggle_breakpoint("script.py", 10);  // Removes / 削除
debugger.toggle_breakpoint("script.py", 10);  // Adds again / 再度追加

// List all breakpoints / すべてのブレークポイントをリスト
for (file, line) in debugger.list_breakpoints() {
    println!("Breakpoint at {}:{}", file, line);
}

// Clear all / すべてをクリア
debugger.clear_all_breakpoints();
```

### Execution Control / 実行制御

```rust
use sikulix_core::debug::{Debugger, DebugState};

let debugger = Debugger::new();

// Pause execution / 実行を一時停止
debugger.pause().unwrap();
assert_eq!(debugger.get_state(), DebugState::Paused);

// Resume / 再開
debugger.resume().unwrap();
assert_eq!(debugger.get_state(), DebugState::Running);

// Step operations / ステップ操作
debugger.step_over().unwrap();   // Next line / 次の行
debugger.step_into().unwrap();   // Into function / 関数に入る
debugger.step_out().unwrap();    // Out of function / 関数から出る

// Stop / 停止
debugger.stop().unwrap();
assert_eq!(debugger.get_state(), DebugState::Stopped);
```

### Variable Inspection / 変数インスペクション

```rust
use sikulix_core::debug::{Debugger, CallFrame, VariableValue, Scope};
use std::path::PathBuf;

let debugger = Debugger::new();

// Create a call frame with local variables / ローカル変数を持つコールフレームを作成
let mut frame = CallFrame::new(
    0,
    "main".to_string(),
    PathBuf::from("script.py"),
    10,
);
frame.add_local("x".to_string(), VariableValue::Int(42));
frame.add_local("name".to_string(), VariableValue::String("Alice".to_string()));

debugger.push_frame(frame);

// Add global variable / グローバル変数を追加
debugger.update_global("PI".to_string(), VariableValue::Float(3.14159));

// Get local variables / ローカル変数を取得
let locals = debugger.get_variables(Scope::Local);
for var in locals {
    println!("{}: {} ({})", var.name, var.value, var.type_name);
}

// Get global variables / グローバル変数を取得
let globals = debugger.get_variables(Scope::Global);

// Get all variables / すべての変数を取得
let all = debugger.get_variables(Scope::All);
```

### Event Callbacks / イベントコールバック

```rust
use sikulix_core::debug::{Debugger, DebugEvent};
use std::sync::Arc;

let debugger = Debugger::new();

// Register callback / コールバックを登録
debugger.register_callback(|event| {
    match event {
        DebugEvent::BreakpointHit { file, line, hit_count } => {
            println!("Hit breakpoint at {}:{} (count: {})",
                file.display(), line, hit_count);
        }
        DebugEvent::Paused { file, line } => {
            println!("Paused at {}:{}", file.display(), line);
        }
        DebugEvent::Resumed => {
            println!("Execution resumed");
        }
        DebugEvent::StepCompleted { file, line } => {
            println!("Step completed at {}:{}", file.display(), line);
        }
        DebugEvent::VariableChanged { name, value } => {
            println!("Variable changed: {} = {}", name, value);
        }
        _ => {}
    }
});

// Events will be triggered automatically / イベントは自動的にトリガーされます
debugger.set_current_position(PathBuf::from("test.py"), 10);
debugger.pause().unwrap();  // Triggers Paused event / Pausedイベントをトリガー
```

### Expression Evaluation / 式評価

```rust
use sikulix_core::debug::{Debugger, CallFrame, VariableValue};
use std::path::PathBuf;

let debugger = Debugger::new();

// Set up some variables / 変数を設定
let mut frame = CallFrame::new(
    0,
    "test".to_string(),
    PathBuf::from("test.py"),
    5,
);
frame.add_local("x".to_string(), VariableValue::Int(42));
debugger.push_frame(frame);

debugger.update_global("PI".to_string(), VariableValue::Float(3.14));

// Evaluate expressions / 式を評価
match debugger.evaluate_expression("x") {
    Ok(VariableValue::Int(value)) => println!("x = {}", value),
    Err(e) => eprintln!("Error: {}", e),
    _ => {}
}

match debugger.evaluate_expression("PI") {
    Ok(value) => println!("PI = {}", value),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration with Python Executor / Python実行エンジンとの統合

The debugger is designed to integrate with the Python execution engine:
デバッガはPython実行エンジンとの統合を想定しています:

```rust
// Example integration pseudocode / 統合の疑似コード例

// In Python executor / Python実行エンジン内で
impl PythonExecutor {
    fn execute_with_debug(&self, script: &str, debugger: Arc<Debugger>) -> Result<()> {
        debugger.set_state(DebugState::Running);

        // Parse script and execute line by line
        // スクリプトを解析し行ごとに実行
        for line_num in 1..script.lines().count() {
            // Check for breakpoint / ブレークポイントをチェック
            if debugger.has_breakpoint(&script_path, line_num) {
                debugger.notify_breakpoint_hit(
                    script_path.clone(),
                    line_num,
                    hit_count,
                );

                // Wait until resumed / 再開されるまで待機
                while debugger.get_state() == DebugState::Paused {
                    std::thread::sleep(Duration::from_millis(100));
                }
            }

            // Execute line / 行を実行
            execute_line(line)?;

            // Update current position / 現在の位置を更新
            debugger.set_current_position(script_path.clone(), line_num);

            // Handle step operations / ステップ操作を処理
            match debugger.get_state() {
                DebugState::StepOver => {
                    debugger.pause()?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
```

## Integration with Tauri IDE / Tauri IDEとの統合

The debugger can be exposed to the Tauri frontend through commands:
デバッガはコマンドを通じてTauriフロントエンドに公開できます:

```rust
// In ide-rs-tauri / ide-rs-tauri内で

use sikulix_core::debug::Debugger;
use std::sync::Arc;

struct DebuggerState {
    debugger: Arc<Debugger>,
}

#[tauri::command]
fn debug_add_breakpoint(
    state: State<DebuggerState>,
    file: String,
    line: u32
) -> Result<(), String> {
    state.debugger.add_breakpoint(&file, line);
    Ok(())
}

#[tauri::command]
fn debug_pause(state: State<DebuggerState>) -> Result<(), String> {
    state.debugger.pause()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn debug_get_variables(
    state: State<DebuggerState>,
    scope: String
) -> Result<Vec<VariableInfo>, String> {
    let scope = match scope.as_str() {
        "local" => Scope::Local,
        "global" => Scope::Global,
        _ => Scope::All,
    };
    Ok(state.debugger.get_variables(scope))
}
```

## Thread Safety / スレッドセーフ

The debugger is fully thread-safe and can be shared across threads:
デバッガは完全にスレッドセーフで、スレッド間で共有できます:

```rust
use std::sync::Arc;
use std::thread;

let debugger = Arc::new(Debugger::new());

// Share across threads / スレッド間で共有
let dbg1 = debugger.clone();
let handle1 = thread::spawn(move || {
    dbg1.add_breakpoint("test.py", 10);
});

let dbg2 = debugger.clone();
let handle2 = thread::spawn(move || {
    dbg2.pause().unwrap();
});

handle1.join().unwrap();
handle2.join().unwrap();
```

## Testing / テスト

Comprehensive test suite in `tests.rs`:
`tests.rs`に包括的なテストスイート:

```bash
# Run all debug tests / すべてのデバッグテストを実行
cargo test --lib debug

# Run specific test / 特定のテストを実行
cargo test --lib debug::tests::test_breakpoint_management

# Run with output / 出力付きで実行
cargo test --lib debug -- --nocapture
```

Test coverage includes:
テストカバレッジには以下が含まれます:

- ✅ Breakpoint management (add, remove, toggle, list, clear)
- ✅ Execution state transitions
- ✅ Call stack operations
- ✅ Variable inspection (local, global, all scopes)
- ✅ Event callbacks
- ✅ Expression evaluation
- ✅ Thread safety
- ✅ Reset functionality

## Future Enhancements / 今後の拡張

- [ ] Conditional breakpoints with full expression evaluation
     条件式の完全評価を持つ条件付きブレークポイント
- [ ] Watch expressions
     監視式
- [ ] Step count and time tracking
     ステップカウントと時間追跡
- [ ] Call stack unwinding with variable inspection at each frame
     各フレームでの変数インスペクション付きコールスタックアンワインド
- [ ] Logpoints (non-breaking breakpoints that log)
     ログポイント（ログを記録する非中断ブレークポイント）
- [ ] Remote debugging support
     リモートデバッグサポート

## License / ライセンス

Same as Sikuli-D project (MIT License)
Sikuli-Dプロジェクトと同じ（MITライセンス）
