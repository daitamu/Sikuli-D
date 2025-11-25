//! Python integration module
//!
//! Provides Python 2/3 dual runtime support with automatic syntax detection.
//! Uses PyO3 for Python 3 embedding.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_python2_print() {
        let source = "print 'hello'";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
        );
    }

    #[test]
    fn test_detect_python3_fstring() {
        let source = "name = 'world'\nprint(f\"hello {name}\")";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python3
        );
    }

    #[test]
    fn test_detect_mixed_syntax() {
        let source = "print 'hello'\nprint(f\"world\")";
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
    fn test_detect_xrange() {
        let source = "for i in xrange(10): pass";
        assert_eq!(
            SyntaxAnalyzer::detect_version(source),
            PythonVersion::Python2
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
    fn test_validate_mixed_error() {
        let source = "print 'hello'\nasync def foo(): pass";
        assert!(SyntaxAnalyzer::validate(source).is_err());
    }
}
