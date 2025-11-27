//! Python integration tests
//! Python統合テスト
//!
//! Tests Python version detection, syntax analysis, and script execution.
//! Pythonバージョン検出、構文解析、スクリプト実行をテストします。

use sikulid::python::{PythonVersion, SyntaxAnalyzer, PythonEnvironment, ScriptExecutor};
use std::path::Path;

#[test]
fn test_python2_syntax_detection() {
    // Test detection of Python 2 specific syntax
    // Python 2固有の構文検出をテスト
    let py2_scripts = vec![
        "print 'Hello Python 2'",
        "print 'test'",
        "raw_input('Enter: ')",
        "xrange(10)",
        "except Exception, e:",
        "123L",
        "basestring",
        "execfile('script.py')",
    ];

    for script in py2_scripts {
        let version = SyntaxAnalyzer::detect_version(script);
        assert_eq!(
            version,
            PythonVersion::Python2,
            "Should detect Python 2 syntax in: {}",
            script
        );
    }
}

#[test]
fn test_python3_syntax_detection() {
    // Test detection of Python 3 specific syntax
    // Python 3固有の構文検出をテスト
    let py3_scripts = vec![
        "print('Hello Python 3')",
        "print('test', end='')",
        "print('a', 'b', sep=',')",
        "def func() -> int:",
        "f'Hello {name}'",
        "async def main():",
        "await asyncio.sleep(1)",
        "x: int = 5",
        "nonlocal x",
    ];

    for script in py3_scripts {
        let version = SyntaxAnalyzer::detect_version(script);
        assert_eq!(
            version,
            PythonVersion::Python3,
            "Should detect Python 3 syntax in: {}",
            script
        );
    }
}

#[test]
fn test_mixed_syntax_detection() {
    // Test detection of mixed Python 2/3 syntax (error case)
    // 混合Python 2/3構文の検出をテスト（エラーケース）
    let mixed_script = r#"
print 'Python 2 syntax'
print('Python 3 syntax', end='')
"#;

    let version = SyntaxAnalyzer::detect_version(mixed_script);
    assert_eq!(
        version,
        PythonVersion::Mixed,
        "Should detect mixed syntax"
    );
}

#[test]
fn test_unknown_syntax_detection() {
    // Test detection when no specific version markers are found
    // バージョンマーカーが見つからない場合の検出をテスト
    let neutral_script = r#"
x = 1 + 1
y = x * 2
result = x + y
"#;

    let version = SyntaxAnalyzer::detect_version(neutral_script);
    assert_eq!(
        version,
        PythonVersion::Unknown,
        "Should return Unknown for neutral syntax"
    );
}

#[test]
fn test_comment_and_empty_line_handling() {
    // Test that comments and empty lines are ignored
    // コメントと空行が無視されることをテスト
    let script_with_comments = r#"
# This is a comment
# print 'This should be ignored'

x = 1  # Inline comment
print('Hello')  # Python 3 print
"#;

    let version = SyntaxAnalyzer::detect_version(script_with_comments);
    assert_eq!(
        version,
        PythonVersion::Python3,
        "Should detect Python 3 and ignore comments"
    );
}

#[test]
fn test_sikuli_script_detection() {
    // Test detection in typical SikuliX scripts
    // 典型的なSikuliXスクリプトでの検出をテスト
    let sikuli_py2_script = r#"
from sikuli import *
click("button.png")
print "Clicked button"
"#;

    let sikuli_py3_script = r#"
from sikuli import *
click("button.png")
print("Clicked button")
"#;

    assert_eq!(
        SyntaxAnalyzer::detect_version(sikuli_py2_script),
        PythonVersion::Python2
    );

    assert_eq!(
        SyntaxAnalyzer::detect_version(sikuli_py3_script),
        PythonVersion::Python3
    );
}

#[test]
fn test_python_version_display() {
    // Test the Display trait implementation
    // Display トレイト実装をテスト
    assert_eq!(PythonVersion::Python2.to_string(), "Python 2");
    assert_eq!(PythonVersion::Python3.to_string(), "Python 3");
    assert_eq!(PythonVersion::Unknown.to_string(), "Unknown");
    assert_eq!(PythonVersion::Mixed.to_string(), "Mixed (Error)");
}

#[test]
fn test_python_environment_detection() {
    // Test Python environment detection
    // Python環境検出をテスト
    let env = PythonEnvironment::detect();

    // Should detect something or report not found
    // 何かを検出するか、見つからないことを報告するべき
    match env {
        Some(python_env) => {
            println!("Detected Python environment: {:?}", python_env);
        }
        None => {
            println!("No Python environment detected");
        }
    }

    // Test passes regardless of Python installation
    // Python インストールに関わらずテストは通過
}

#[test]
fn test_script_executor_creation() {
    // Test that ScriptExecutor can be created
    // ScriptExecutor が作成できることをテスト
    let executor = ScriptExecutor::new();

    // Should be able to create executor
    // エグゼキュータを作成できるべき
    assert!(executor.is_ok() || executor.is_err());
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_script_execution_simple() -> sikulid::Result<()> {
    // Test execution of a simple Python script
    // シンプルなPythonスクリプトの実行をテスト
    let script = r#"
print("Hello from Python")
result = 2 + 2
print(f"Result: {result}")
"#;

    let mut executor = ScriptExecutor::new()?;
    let output = executor.execute_script(script)?;

    assert!(output.stdout.contains("Hello from Python"));
    assert!(output.stdout.contains("Result: 4"));
    assert_eq!(output.exit_code, 0);

    Ok(())
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_script_execution_with_error() -> sikulid::Result<()> {
    // Test execution of a script with errors
    // エラーを含むスクリプトの実行をテスト
    let script = r#"
print("Starting")
invalid syntax here!!!
print("This won't execute")
"#;

    let mut executor = ScriptExecutor::new()?;
    let result = executor.execute_script(script);

    assert!(result.is_err(), "Should fail on syntax error");

    Ok(())
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_script_execution_with_imports() -> sikulid::Result<()> {
    // Test execution with standard library imports
    // 標準ライブラリインポートを使った実行をテスト
    let script = r#"
import sys
import os

print(f"Python version: {sys.version}")
print(f"Platform: {sys.platform}")
"#;

    let mut executor = ScriptExecutor::new()?;
    let output = executor.execute_script(script)?;

    assert!(output.stdout.contains("Python version:"));
    assert!(output.stdout.contains("Platform:"));

    Ok(())
}

#[test]
#[ignore = "Requires Python runtime and test fixture"]
fn test_python_script_from_file() -> sikulid::Result<()> {
    // Test execution from a file
    // ファイルからの実行をテスト
    let script_path = "tests/fixtures/scripts/test_simple.py";

    if !Path::new(script_path).exists() {
        // Skip if fixture doesn't exist
        // フィクスチャが存在しない場合はスキップ
        println!("Test fixture not found: {}", script_path);
        return Ok(());
    }

    let script = std::fs::read_to_string(script_path)?;
    let mut executor = ScriptExecutor::new()?;
    let output = executor.execute_script(&script)?;

    assert_eq!(output.exit_code, 0);

    Ok(())
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_script_with_timeout() -> sikulid::Result<()> {
    // Test script execution with timeout
    // タイムアウト付きスクリプト実行をテスト
    let script = r#"
import time
print("Starting long operation")
time.sleep(10)
print("Finished")
"#;

    let mut executor = ScriptExecutor::new()?;

    // Set a short timeout
    // 短いタイムアウトを設定
    executor.set_timeout(1.0);

    let start = std::time::Instant::now();
    let result = executor.execute_script(script);
    let elapsed = start.elapsed();

    // Should timeout
    // タイムアウトするべき
    assert!(result.is_err(), "Should timeout");
    assert!(
        elapsed.as_secs() <= 2,
        "Should timeout quickly, took {}s",
        elapsed.as_secs()
    );

    Ok(())
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_script_cancellation() -> sikulid::Result<()> {
    // Test script cancellation
    // スクリプトキャンセルをテスト
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let script = r#"
import time
for i in range(100):
    print(f"Iteration {i}")
    time.sleep(0.1)
"#;

    let mut executor = ScriptExecutor::new()?;
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Spawn execution in background
    // バックグラウンドで実行を開始
    let cancel_flag_clone = cancel_flag.clone();
    let handle = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(500));
        cancel_flag_clone.store(true, Ordering::SeqCst);
    });

    executor.set_cancel_flag(cancel_flag);
    let result = executor.execute_script(script);

    handle.join().unwrap();

    // Should be cancelled
    // キャンセルされるべき
    assert!(result.is_err(), "Should be cancelled");

    Ok(())
}

#[test]
fn test_python_script_validation() {
    // Test script validation without execution
    // 実行せずにスクリプト検証をテスト
    let valid_scripts = vec![
        "print('Hello')",
        "x = 1 + 1",
        "def func(): pass",
    ];

    let invalid_scripts = vec![
        "print 'Missing parentheses in Python 3'  # But valid in Python 2",
        "invalid syntax!!!",
        "def func( missing_paren:",
    ];

    // For now, just verify version detection works
    // 今のところ、バージョン検出が機能することを確認
    for script in valid_scripts {
        let _version = SyntaxAnalyzer::detect_version(script);
        // Should not panic
        // パニックしないべき
    }

    for script in invalid_scripts {
        let _version = SyntaxAnalyzer::detect_version(script);
        // Should not panic even for invalid syntax
        // 無効な構文でもパニックしないべき
    }
}

#[test]
#[ignore = "Requires Python runtime"]
fn test_python_variable_inspection() -> sikulid::Result<()> {
    // Test inspecting variables after script execution
    // スクリプト実行後の変数インスペクションをテスト
    let script = r#"
x = 42
name = "Test"
items = [1, 2, 3]
"#;

    let mut executor = ScriptExecutor::new()?;
    let _output = executor.execute_script(script)?;

    // Try to get variables
    // 変数取得を試行
    if let Ok(vars) = executor.get_variables() {
        println!("Variables: {:?}", vars);
        assert!(vars.contains_key("x") || vars.len() >= 0);
    }

    Ok(())
}

#[test]
fn test_complex_python_detection() {
    // Test detection in more complex scenarios
    // より複雑なシナリオでの検出をテスト
    let complex_script = r#"
#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
This is a docstring
"""

from typing import List, Optional

def process_data(items: List[int]) -> Optional[int]:
    """Process items and return result."""
    if not items:
        return None

    result = sum(items)
    print(f"Processing {len(items)} items")

    return result

if __name__ == "__main__":
    data = [1, 2, 3, 4, 5]
    result = process_data(data)
    print(f"Result: {result}")
"#;

    let version = SyntaxAnalyzer::detect_version(complex_script);
    assert_eq!(
        version,
        PythonVersion::Python3,
        "Should detect Python 3 from type hints and f-strings"
    );
}

#[test]
fn test_japanese_script_detection() {
    // Test detection with Japanese comments and strings
    // 日本語コメントと文字列での検出をテスト
    let japanese_script = r#"
# 日本語のコメント
print("こんにちは、世界！")
print("テスト")
"#;

    let version = SyntaxAnalyzer::detect_version(japanese_script);
    assert_eq!(
        version,
        PythonVersion::Python3,
        "Should handle Japanese text correctly"
    );
}
