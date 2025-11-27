//! Tests for REPL functionality
//! REPL機能のテスト

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::path::PathBuf;

    #[test]
    fn test_repl_config_default() {
        let config = ReplConfig::default();
        assert_eq!(config.python_path, None);
        assert_eq!(config.enable_history, true);
        assert_eq!(config.startup_script, None);
    }

    #[test]
    fn test_has_unclosed_brackets() {
        let repl = Repl::new(ReplConfig::default()).expect("Failed to create REPL");

        // Balanced
        assert!(!repl.has_unclosed_brackets("print('hello')"));
        assert!(!repl.has_unclosed_brackets("func(a, b, c)"));
        assert!(!repl.has_unclosed_brackets("x = [1, 2, 3]"));

        // Unclosed
        assert!(repl.has_unclosed_brackets("print('hello'"));
        assert!(repl.has_unclosed_brackets("func(a, b"));
        assert!(repl.has_unclosed_brackets("x = [1, 2"));
        assert!(repl.has_unclosed_brackets("def foo():"));

        // String handling
        assert!(!repl.has_unclosed_brackets(r#"print("()")"#));
        assert!(repl.has_unclosed_brackets(r#"print("hello"#));
    }

    #[test]
    fn test_is_incomplete() {
        let repl = Repl::new(ReplConfig::default()).expect("Failed to create REPL");

        // Complete lines
        assert!(!repl.is_incomplete("x = 5"));
        assert!(!repl.is_incomplete("print('hello')"));

        // Incomplete lines
        assert!(repl.is_incomplete("def foo():"));
        assert!(repl.is_incomplete("if x > 5:"));
        assert!(repl.is_incomplete("x = \\"));
        assert!(repl.is_incomplete("print('hello'"));
    }

    #[test]
    fn test_history_file_path() {
        let path = Repl::get_history_file();
        assert!(path.to_string_lossy().contains(".sikulix_history"));
    }

    #[test]
    fn test_startup_script() {
        let script = PythonRepl::create_startup_script();
        assert!(script.contains("from sikulix_api import *"));
        assert!(script.contains("def sikulix_help():"));
        assert!(script.contains("sys.ps1 = \"\""));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::super::*;

    // Note: These tests require Python to be installed
    // これらのテストにはPythonがインストールされている必要があります

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_find_python() {
        let result = crate::python::find_python();
        assert!(result.is_ok(), "Python should be found on system");
    }

    #[test]
    #[ignore]
    fn test_python_repl_start() {
        let python = crate::python::find_python()
            .expect("Python not found");

        let result = PythonRepl::start(&python);
        assert!(result.is_ok(), "Should start Python REPL");
    }
}
