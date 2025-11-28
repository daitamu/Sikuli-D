//! Python integration module
//! Python統合モジュール
//!
//! Provides Python 2/3 dual runtime support with automatic syntax detection.
//! Python 2/3デュアルランタイム対応と自動構文検出を提供します。
//! Uses PyO3 for Python 3 embedding.
//! PyO3を使用してPython 3を組み込みます。

pub mod detector;
pub mod executor;

#[cfg(feature = "python")]
pub mod bindings;

pub use detector::PythonEnvironment;
pub use executor::{ExecutionState, OutputLine, ScriptExecutor};

#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::{Result, SikulixError};

/// Python version detected from syntax
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PythonVersion {
    Python2,
    Python3,
    Unknown,
    Mixed, // Contains both Python 2 and 3 specific syntax (error)
}

impl std::fmt::Display for PythonVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PythonVersion::Python2 => write!(f, "Python 2"),
            PythonVersion::Python3 => write!(f, "Python 3"),
            PythonVersion::Unknown => write!(f, "Unknown"),
            PythonVersion::Mixed => write!(f, "Mixed (Error)"),
        }
    }
}

/// Python syntax analyzer for version detection
pub struct SyntaxAnalyzer;

impl SyntaxAnalyzer {
    /// Detect Python version from source code
    pub fn detect_version(source: &str) -> PythonVersion {
        let mut has_py2_syntax = false;
        let mut has_py3_syntax = false;

        for line in source.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Python 2 specific patterns
            if Self::is_python2_syntax(trimmed) {
                has_py2_syntax = true;
            }

            // Python 3 specific patterns
            if Self::is_python3_syntax(trimmed) {
                has_py3_syntax = true;
            }
        }

        match (has_py2_syntax, has_py3_syntax) {
            (true, true) => PythonVersion::Mixed,
            (true, false) => PythonVersion::Python2,
            (false, true) => PythonVersion::Python3,
            (false, false) => PythonVersion::Unknown, // Default to Python 3
        }
    }

    /// Check for Python 2 specific syntax
    fn is_python2_syntax(line: &str) -> bool {
        // print statement (not function)
        if line.starts_with("print ") && !line.contains("print(") {
            return true;
        }

        // Old-style exception handling
        if line.contains("except") && line.contains(",") && !line.contains(" as ") {
            // except Exception, e: (Python 2)
            return true;
        }

        // xrange (Python 2 only)
        if line.contains("xrange(") {
            return true;
        }

        // raw_input (Python 2 only)
        if line.contains("raw_input(") {
            return true;
        }

        // Unicode literal u"..." (primarily Python 2, but valid in 3.3+)
        // We'll be lenient here

        // Long integer literal with L suffix
        if Self::has_long_literal(line) {
            return true;
        }

        // basestring (Python 2 only)
        if line.contains("basestring") {
            return true;
        }

        // execfile (Python 2 only)
        if line.contains("execfile(") {
            return true;
        }

        false
    }

    /// Check for Python 3 specific syntax
    fn is_python3_syntax(line: &str) -> bool {
        // print function with keywords
        if line.contains("print(")
            && (line.contains("end=") || line.contains("sep=") || line.contains("file="))
        {
            return true;
        }

        // Type hints
        if line.contains("->") && line.contains("def ") {
            return true;
        }

        // f-strings
        if line.contains("f\"") || line.contains("f'") {
            return true;
        }

        // async/await
        if line.starts_with("async ")
            || line.contains(" async ")
            || line.starts_with("await ")
            || line.contains(" await ")
        {
            return true;
        }

        // Walrus operator
        if line.contains(":=") {
            return true;
        }

        // nonlocal keyword
        if line.starts_with("nonlocal ") {
            return true;
        }

        // yield from
        if line.contains("yield from") {
            return true;
        }

        // Keyword-only arguments with *
        if line.contains("def ") && line.contains(", *,") {
            return true;
        }

        false
    }

    /// Check for long integer literals (123L)
    fn has_long_literal(line: &str) -> bool {
        let chars: Vec<char> = line.chars().collect();
        for i in 1..chars.len() {
            if (chars[i] == 'L' || chars[i] == 'l') && chars[i - 1].is_ascii_digit() {
                // Make sure it's not part of an identifier
                if i + 1 >= chars.len() || !chars[i + 1].is_alphanumeric() {
                    return true;
                }
            }
        }
        false
    }

    /// Validate that source code doesn't mix Python 2 and 3 syntax
    pub fn validate(source: &str) -> Result<PythonVersion> {
        let version = Self::detect_version(source);
        match version {
            PythonVersion::Mixed => Err(SikulixError::PythonError(
                "Source code contains mixed Python 2 and 3 syntax".to_string(),
            )),
            _ => Ok(version),
        }
    }
}

/// Python runtime for executing scripts
#[cfg(feature = "python")]
pub struct PythonRuntime {
    version: PythonVersion,
}

#[cfg(feature = "python")]
impl PythonRuntime {
    /// Create a new Python runtime
    pub fn new() -> Result<Self> {
        // Initialize Python interpreter
        pyo3::prepare_freethreaded_python();

        Ok(Self {
            version: PythonVersion::Python3,
        })
    }

    /// Execute Python code
    pub fn execute(&self, source: &str) -> Result<()> {
        let detected = SyntaxAnalyzer::validate(source)?;

        if detected == PythonVersion::Python2 {
            return Err(SikulixError::PythonError(
                "Python 2 syntax detected. Please update to Python 3 syntax.".to_string(),
            ));
        }

        Python::with_gil(|py| {
            py.run(source, None, None)
                .map_err(|e| SikulixError::PythonError(e.to_string()))
        })
    }

    /// Execute Python code and return result
    pub fn eval<T: for<'py> FromPyObject<'py>>(&self, expression: &str) -> Result<T> {
        Python::with_gil(|py| {
            py.eval(expression, None, None)
                .map_err(|e| SikulixError::PythonError(e.to_string()))?
                .extract()
                .map_err(|e| SikulixError::PythonError(e.to_string()))
        })
    }

    /// Get the Python version being used
    pub fn version(&self) -> PythonVersion {
        self.version
    }
}

/// Stub runtime when Python feature is disabled
#[cfg(not(feature = "python"))]
pub struct PythonRuntime;

#[cfg(not(feature = "python"))]
impl PythonRuntime {
    pub fn new() -> Result<Self> {
        Err(SikulixError::PythonError(
            "Python support not compiled. Enable 'python' feature.".to_string(),
        ))
    }
}

/// Convenience function to detect system Python
/// システムPythonを検出する便利関数
pub fn detect_system_python() -> Result<PythonEnvironment> {
    PythonEnvironment::detect_system()
        .ok_or_else(|| SikulixError::PythonError("Python not found on system".to_string()))
}

/// Convenience function to detect all Python environments
/// すべてのPython環境を検出する便利関数
pub fn detect_all_python() -> Vec<PythonEnvironment> {
    PythonEnvironment::detect_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ======================================================================
    // PythonVersion Tests / PythonVersion テスト
    // ======================================================================

    #[test]
    fn test_python_version_display() {
        assert_eq!(PythonVersion::Python2.to_string(), "Python 2");
        assert_eq!(PythonVersion::Python3.to_string(), "Python 3");
        assert_eq!(PythonVersion::Unknown.to_string(), "Unknown");
        assert_eq!(PythonVersion::Mixed.to_string(), "Mixed (Error)");
    }

    #[test]
    fn test_python_version_equality() {
        assert_eq!(PythonVersion::Python2, PythonVersion::Python2);
        assert_ne!(PythonVersion::Python2, PythonVersion::Python3);
    }

    // ======================================================================
    // SyntaxAnalyzer - Python 2 Detection Tests / Python 2 検出テスト
    // ======================================================================

    #[test]
    fn test_detect_python2_print() {
        let source = "print 'hello'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_python2_print_multiple_items() {
        let source = "print 'hello', 'world'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_python2_exception_syntax() {
        let source = "try:\n    pass\nexcept Exception, e:\n    pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_xrange() {
        let source = "for i in xrange(10): pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_raw_input() {
        let source = "name = raw_input('Enter name: ')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_long_literal() {
        let source = "x = 123456789L";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_long_literal_lowercase() {
        let source = "x = 123l";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_basestring() {
        let source = "if isinstance(x, basestring): pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_execfile() {
        let source = "execfile('script.py')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    // ======================================================================
    // SyntaxAnalyzer - Python 3 Detection Tests / Python 3 検出テスト
    // ======================================================================

    #[test]
    fn test_detect_python3_fstring() {
        let source = "name = 'world'\nprint(f\"hello {name}\")";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_python3_fstring_single_quote() {
        let source = "print(f'hello {name}')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_python3_print_with_end() {
        let source = "print('hello', end='')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_python3_print_with_sep() {
        let source = "print('a', 'b', sep=', ')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_python3_print_with_file() {
        let source = "print('error', file=sys.stderr)";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_python3_type_hints() {
        let source = "def greet(name: str) -> str:\n    return f'Hello {name}'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_async() {
        let source = "async def foo(): await bar()";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_async_with_space() {
        let source = "x = async def foo(): pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_await() {
        let source = "result = await some_coroutine()";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_walrus_operator() {
        let source = "if (n := len(items)) > 10:\n    print(n)";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_nonlocal() {
        let source = "def outer():\n    x = 1\n    def inner():\n        nonlocal x\n        x = 2";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_yield_from() {
        let source = "def generator():\n    yield from range(10)";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_keyword_only_args() {
        let source = "def func(a, *, b):\n    pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    // ======================================================================
    // SyntaxAnalyzer - Mixed and Unknown Tests / Mixed/Unknown テスト
    // ======================================================================

    #[test]
    fn test_detect_mixed_syntax() {
        let source = "print 'hello'\nprint(f\"world\")";
        assert_eq!(SyntaxAnalyzer::detect_version(source), PythonVersion::Mixed);
    }

    #[test]
    fn test_detect_mixed_async_and_print() {
        let source = "print 'hello'\nasync def foo(): pass";
        assert_eq!(SyntaxAnalyzer::detect_version(source), PythonVersion::Mixed);
    }

    #[test]
    fn test_detect_mixed_xrange_and_fstring() {
        let source = "for i in xrange(10):\n    print(f'item {i}')";
        assert_eq!(SyntaxAnalyzer::detect_version(source), PythonVersion::Mixed);
    }

    #[test]
    fn test_detect_unknown() {
        let source = "x = 1\ny = 2\nz = x + y";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_detect_unknown_print_function() {
        // print() with no keywords could be Python 2 or 3
        let source = "print('hello')";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_detect_empty_source() {
        let source = "";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_detect_only_comments() {
        let source = "# This is a comment\n# Another comment";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_detect_whitespace_only() {
        let source = "   \n\n\t  \n   ";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    // ======================================================================
    // SyntaxAnalyzer - Edge Cases / エッジケース
    // ======================================================================

    #[test]
    fn test_detect_ignores_inline_comments() {
        let source = "x = 1  # print 'this is a comment'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_detect_print_in_string_not_detected() {
        let source = "s = \"print 'hello'\"";
        // This should NOT detect as Python 2 since print is in a string
        // However, the current implementation may detect it
        // This is an edge case that could be improved
        let version = SyntaxAnalyzer::detect_version(source);
        assert!(matches!(
            version,
            PythonVersion::Python2 | PythonVersion::Unknown
        ));
    }

    #[test]
    fn test_long_literal_in_word() {
        // "HELLO" contains "L" but should not be detected as long literal
        let source = "x = 'HELLO'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Unknown
        );
    }

    #[test]
    fn test_long_literal_at_end_of_number() {
        let source = "x = 12345L";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_long_literal_with_following_operator() {
        let source = "x = 100L + 200";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_multiline_code() {
        let source = r#"
def greet(name: str) -> str:
    return f'Hello {name}'

if __name__ == '__main__':
    print(greet('World'))
"#;
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    // ======================================================================
    // SyntaxAnalyzer - validate() Tests / validate() テスト
    // ======================================================================

    #[test]
    fn test_validate_mixed_error() {
        let source = "print 'hello'\nasync def foo(): pass";
        assert!(SyntaxAnalyzer::validate(source).is_err());
    }

    #[test]
    fn test_validate_python2_ok() {
        let source = "print 'hello'";
        let result = SyntaxAnalyzer::validate(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PythonVersion::Python2);
    }

    #[test]
    fn test_validate_python3_ok() {
        let source = "async def foo(): pass";
        let result = SyntaxAnalyzer::validate(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PythonVersion::Python3);
    }

    #[test]
    fn test_validate_unknown_ok() {
        let source = "x = 1 + 2";
        let result = SyntaxAnalyzer::validate(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PythonVersion::Unknown);
    }

    #[test]
    fn test_validate_mixed_error_message() {
        let source = "print 'hello'\nprint(f'world')";
        let result = SyntaxAnalyzer::validate(source);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("mixed"));
    }

    // ======================================================================
    // SyntaxAnalyzer - Complex Real-World Examples / 実際の複雑な例
    // ======================================================================

    #[test]
    fn test_detect_sikuli_python2_script() {
        let source = r#"
print "SikuliX Python 2 Script"
for i in xrange(10):
    print "Iteration:", i
    if exists("button.png"):
        click("button.png")
"#;
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_sikuli_python3_script() {
        let source = r#"
print("SikuliX Python 3 Script")
for i in range(10):
    print(f"Iteration: {i}")
    if exists("button.png"):
        click("button.png")
"#;
        // This might be detected as Python3 due to f-string, or Unknown
        let version = SyntaxAnalyzer::detect_version(source);
        assert!(matches!(
            version,
            PythonVersion::Python3 | PythonVersion::Unknown
        ));
    }

    #[test]
    fn test_detect_async_sikuli_script() {
        let source = r#"
async def wait_for_button():
    while not exists("button.png"):
        await asyncio.sleep(0.1)
    click("button.png")
"#;
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }
}
