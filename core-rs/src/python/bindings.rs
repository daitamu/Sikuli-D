//! PyO3 bindings for SikuliX Core
//! SikuliX Core の PyO3 バインディング
//!
//! This module provides Python bindings for the core Sikuli-D functionality.
//! このモジュールは、コア Sikuli-D 機能の Python バインディングを提供します。

#[cfg(feature = "python")]
use pyo3::prelude::*;
#[cfg(feature = "python")]
use pyo3::exceptions::{PyRuntimeError, PyValueError, PyIOError};

use crate::{
    Region, Match, Pattern, Screen, Mouse, Keyboard, Key, ImageMatcher,
    Result, SikulixError,
};

// ============================================================================
// Error Conversion / エラー変換
// ============================================================================

#[cfg(feature = "python")]
fn to_pyerr(err: SikulixError) -> PyErr {
    match err {
        SikulixError::ImageNotFound => {
            PyRuntimeError::new_err("Image not found on screen")
        }
        SikulixError::ImageLoadError(msg) => {
            PyValueError::new_err(format!("Failed to load image: {}", msg))
        }
        SikulixError::ScreenCaptureError(msg) => {
            PyRuntimeError::new_err(format!("Screen capture failed: {}", msg))
        }
        SikulixError::FindFailed { pattern_name, timeout_secs } => {
            PyRuntimeError::new_err(
                format!("FindFailed: {} not found within {}s", pattern_name, timeout_secs)
            )
        }
        SikulixError::MouseError(msg) => {
            PyRuntimeError::new_err(format!("Mouse operation failed: {}", msg))
        }
        SikulixError::KeyboardError(msg) => {
            PyRuntimeError::new_err(format!("Keyboard operation failed: {}", msg))
        }
        SikulixError::IoError(e) => {
            PyIOError::new_err(e.to_string())
        }
        _ => PyRuntimeError::new_err(err.to_string()),
    }
}

// ============================================================================
// PyScreen - Screen capture wrapper
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Screen")]
struct PyScreen {
    inner: Screen,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyScreen {
    #[new]
    fn new(index: Option<u32>) -> Self {
        Self {
            inner: Screen::new(index.unwrap_or(0)),
        }
    }

    /// Capture the entire screen
    /// 画面全体をキャプチャ
    fn capture(&self, py: Python) -> PyResult<Vec<u8>> {
        py.allow_threads(|| {
            self.inner
                .capture()
                .map_err(to_pyerr)?
                .into_bytes()
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to convert image: {}", e)))
        })
    }

    /// Get screen dimensions (width, height)
    /// 画面サイズを取得 (幅, 高さ)
    fn dimensions(&mut self) -> PyResult<(u32, u32)> {
        self.inner
            .dimensions()
            .map_err(to_pyerr)
    }

    /// Get screen region
    /// 画面領域を取得
    fn get_region(&mut self) -> PyResult<PyRegion> {
        let region = self.inner.get_region().map_err(to_pyerr)?;
        Ok(PyRegion { inner: region })
    }
}

// ============================================================================
// PyRegion - Region wrapper
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Region")]
#[derive(Clone)]
struct PyRegion {
    inner: Region,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyRegion {
    #[new]
    fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            inner: Region::new(x, y, width, height),
        }
    }

    /// Get region center coordinates
    /// 領域の中心座標を取得
    fn center(&self) -> (i32, i32) {
        self.inner.center()
    }

    /// Check if point is inside region
    /// 点が領域内にあるか確認
    fn contains(&self, x: i32, y: i32) -> bool {
        self.inner.contains(x, y)
    }

    /// Get region area
    /// 領域の面積を取得
    fn area(&self) -> u64 {
        self.inner.area()
    }

    /// Offset region by (dx, dy)
    /// 領域を (dx, dy) だけオフセット
    fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            inner: self.inner.offset(dx, dy),
        }
    }

    /// Expand region by amount
    /// 領域を量だけ拡大
    fn expand(&self, amount: i32) -> Self {
        Self {
            inner: self.inner.expand(amount),
        }
    }

    // Getters for properties
    #[getter]
    fn x(&self) -> i32 { self.inner.x }

    #[getter]
    fn y(&self) -> i32 { self.inner.y }

    #[getter]
    fn width(&self) -> u32 { self.inner.width }

    #[getter]
    fn height(&self) -> u32 { self.inner.height }

    fn __repr__(&self) -> String {
        format!("Region({}, {}, {}, {})",
            self.inner.x, self.inner.y, self.inner.width, self.inner.height)
    }
}

// ============================================================================
// PyMatch - Match result wrapper
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Match")]
#[derive(Clone)]
struct PyMatch {
    inner: Match,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyMatch {
    /// Get match center coordinates
    /// マッチの中心座標を取得
    fn center(&self) -> (i32, i32) {
        self.inner.center()
    }

    /// Get target click point (alias for center)
    /// ターゲットクリックポイントを取得 (center のエイリアス)
    fn target(&self) -> (i32, i32) {
        self.inner.target()
    }

    /// Get match score (0.0-1.0)
    /// マッチスコアを取得 (0.0-1.0)
    #[getter]
    fn score(&self) -> f64 {
        self.inner.score
    }

    /// Get match region
    /// マッチ領域を取得
    #[getter]
    fn region(&self) -> PyRegion {
        PyRegion { inner: self.inner.region }
    }

    /// Click at match center
    /// マッチの中心をクリック
    fn click(&self) -> PyResult<()> {
        let (x, y) = self.center();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
        Mouse::click().map_err(to_pyerr)
    }

    /// Double click at match center
    /// マッチの中心をダブルクリック
    fn double_click(&self) -> PyResult<()> {
        let (x, y) = self.center();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
        Mouse::double_click().map_err(to_pyerr)
    }

    /// Right click at match center
    /// マッチの中心を右クリック
    fn right_click(&self) -> PyResult<()> {
        let (x, y) = self.center();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
        Mouse::right_click().map_err(to_pyerr)
    }

    /// Highlight the match region
    /// マッチ領域をハイライト
    fn highlight(&self, seconds: Option<f64>) -> PyResult<()> {
        if let Some(s) = seconds {
            self.inner.highlight_with_duration(s)
        } else {
            self.inner.highlight()
        }.map_err(to_pyerr)
    }

    fn __repr__(&self) -> String {
        format!("Match(score={:.2}, region={})",
            self.inner.score,
            format!("Region({}, {}, {}, {})",
                self.inner.region.x, self.inner.region.y,
                self.inner.region.width, self.inner.region.height))
    }
}

// ============================================================================
// PyPattern - Pattern wrapper
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Pattern")]
struct PyPattern {
    inner: Pattern,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPattern {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        let pattern = Pattern::from_file(&path).map_err(to_pyerr)?;
        Ok(Self { inner: pattern })
    }

    /// Set similarity threshold (0.0-1.0)
    /// 類似度閾値を設定 (0.0-1.0)
    fn similar(mut slf: PyRefMut<Self>, similarity: f64) -> PyRefMut<Self> {
        slf.inner = slf.inner.clone().similar(similarity);
        slf
    }

    /// Set target offset (x, y)
    /// ターゲットオフセットを設定 (x, y)
    fn target_offset(mut slf: PyRefMut<Self>, x: i32, y: i32) -> PyRefMut<Self> {
        slf.inner = slf.inner.clone().target_offset(x, y);
        slf
    }

    #[getter]
    fn similarity(&self) -> f64 {
        self.inner.similarity
    }

    fn __repr__(&self) -> String {
        format!("Pattern(similarity={:.2})", self.inner.similarity)
    }
}

// ============================================================================
// Module-Level Functions / モジュールレベル関数
// ============================================================================

/// Find pattern on screen (best match)
/// 画面上でパターンを検索 (最良マッチ)
#[cfg(feature = "python")]
#[pyfunction]
fn find(py: Python, pattern: &str, similarity: Option<f64>) -> PyResult<Option<PyMatch>> {
    py.allow_threads(|| {
        let screen = Screen::primary();
        let pat = Pattern::from_file(pattern)
            .map_err(to_pyerr)?
            .similar(similarity.unwrap_or(0.7));

        let screen_img = screen.capture().map_err(to_pyerr)?;
        let matcher = ImageMatcher::new();

        let result = matcher.find(&screen_img, &pat).map_err(to_pyerr)?;
        Ok(result.map(|m| PyMatch { inner: m }))
    })
}

/// Find all occurrences of pattern on screen
/// 画面上でパターンの全ての出現箇所を検索
#[cfg(feature = "python")]
#[pyfunction]
fn find_all(py: Python, pattern: &str, similarity: Option<f64>) -> PyResult<Vec<PyMatch>> {
    py.allow_threads(|| {
        let screen = Screen::primary();
        let pat = Pattern::from_file(pattern)
            .map_err(to_pyerr)?
            .similar(similarity.unwrap_or(0.7));

        let screen_img = screen.capture().map_err(to_pyerr)?;
        let matcher = ImageMatcher::new();

        let results = matcher.find_all(&screen_img, &pat).map_err(to_pyerr)?;
        Ok(results.into_iter().map(|m| PyMatch { inner: m }).collect())
    })
}

/// Wait for pattern to appear
/// パターンが表示されるのを待機
#[cfg(feature = "python")]
#[pyfunction]
fn wait(py: Python, pattern: &str, timeout: Option<f64>) -> PyResult<PyMatch> {
    py.allow_threads(|| {
        let screen = Screen::primary();
        let pat = Pattern::from_file(pattern).map_err(to_pyerr)?;
        let matcher = ImageMatcher::new();

        let result = matcher.wait(&screen, &pat, timeout.unwrap_or(3.0))
            .map_err(to_pyerr)?;

        Ok(PyMatch { inner: result })
    })
}

/// Check if pattern exists (non-throwing)
/// パターンが存在するか確認 (例外を投げない)
#[cfg(feature = "python")]
#[pyfunction]
fn exists(py: Python, pattern: &str, timeout: Option<f64>) -> PyResult<Option<PyMatch>> {
    py.allow_threads(|| {
        let screen = Screen::primary();
        let pat = Pattern::from_file(pattern).map_err(to_pyerr)?;
        let matcher = ImageMatcher::new();

        let result = matcher.exists(&screen, &pat, timeout.unwrap_or(0.0))
            .map_err(to_pyerr)?;

        Ok(result.map(|m| PyMatch { inner: m }))
    })
}

// ============================================================================
// Mouse/Keyboard Input Functions / マウス・キーボード入力関数
// ============================================================================

/// Click at coordinates or current position
/// 座標または現在位置でクリック
#[cfg(feature = "python")]
#[pyfunction]
fn click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::click().map_err(to_pyerr)
}

/// Double click at coordinates or current position
/// 座標または現在位置でダブルクリック
#[cfg(feature = "python")]
#[pyfunction]
fn double_click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::double_click().map_err(to_pyerr)
}

/// Right click at coordinates or current position
/// 座標または現在位置で右クリック
#[cfg(feature = "python")]
#[pyfunction]
fn right_click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::right_click().map_err(to_pyerr)
}

/// Type text
/// テキストを入力
#[cfg(feature = "python")]
#[pyfunction]
fn type_text(text: &str) -> PyResult<()> {
    Keyboard::type_text(text).map_err(to_pyerr)
}

/// Paste text via clipboard
/// クリップボード経由でテキストをペースト
#[cfg(feature = "python")]
#[pyfunction]
fn paste(text: &str) -> PyResult<()> {
    Keyboard::paste_text(text).map_err(to_pyerr)
}

/// Press hotkey combination
/// ホットキーの組み合わせを押下
#[cfg(feature = "python")]
#[pyfunction]
fn hotkey(keys: Vec<String>) -> PyResult<()> {
    let key_enum: Vec<Key> = keys.iter()
        .map(|s| parse_key(s))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| PyValueError::new_err(e))?;

    Keyboard::hotkey(&key_enum).map_err(to_pyerr)
}

/// Parse key string to Key enum
/// キー文字列をKey列挙型に変換
#[cfg(feature = "python")]
fn parse_key(s: &str) -> std::result::Result<Key, String> {
    match s.to_lowercase().as_str() {
        "ctrl" => Ok(Key::Ctrl),
        "shift" => Ok(Key::Shift),
        "alt" => Ok(Key::Alt),
        "meta" | "win" | "cmd" | "command" => Ok(Key::Meta),
        "enter" | "return" => Ok(Key::Enter),
        "tab" => Ok(Key::Tab),
        "space" => Ok(Key::Space),
        "backspace" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "escape" | "esc" => Ok(Key::Escape),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "pgup" => Ok(Key::PageUp),
        "pagedown" | "pgdn" => Ok(Key::PageDown),
        "up" => Ok(Key::Up),
        "down" => Ok(Key::Down),
        "left" => Ok(Key::Left),
        "right" => Ok(Key::Right),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        // Single letter keys
        s if s.len() == 1 => {
            let c = s.chars().next().unwrap().to_ascii_uppercase();
            if c.is_ascii_alphabetic() {
                match c {
                    'A' => Ok(Key::A), 'B' => Ok(Key::B), 'C' => Ok(Key::C), 'D' => Ok(Key::D),
                    'E' => Ok(Key::E), 'F' => Ok(Key::F), 'G' => Ok(Key::G), 'H' => Ok(Key::H),
                    'I' => Ok(Key::I), 'J' => Ok(Key::J), 'K' => Ok(Key::K), 'L' => Ok(Key::L),
                    'M' => Ok(Key::M), 'N' => Ok(Key::N), 'O' => Ok(Key::O), 'P' => Ok(Key::P),
                    'Q' => Ok(Key::Q), 'R' => Ok(Key::R), 'S' => Ok(Key::S), 'T' => Ok(Key::T),
                    'U' => Ok(Key::U), 'V' => Ok(Key::V), 'W' => Ok(Key::W), 'X' => Ok(Key::X),
                    'Y' => Ok(Key::Y), 'Z' => Ok(Key::Z),
                    _ => Err(format!("Unknown key: {}", s)),
                }
            } else if c.is_ascii_digit() {
                match c {
                    '0' => Ok(Key::Num0), '1' => Ok(Key::Num1), '2' => Ok(Key::Num2),
                    '3' => Ok(Key::Num3), '4' => Ok(Key::Num4), '5' => Ok(Key::Num5),
                    '6' => Ok(Key::Num6), '7' => Ok(Key::Num7), '8' => Ok(Key::Num8),
                    '9' => Ok(Key::Num9),
                    _ => Err(format!("Unknown key: {}", s)),
                }
            } else {
                Err(format!("Unknown key: {}", s))
            }
        }
        _ => Err(format!("Unknown key: {}", s)),
    }
}

// ============================================================================
// PyO3 Module Entry Point / PyO3 モジュールエントリーポイント
// ============================================================================

/// Creates the Python module
/// Python モジュールを作成
#[cfg(feature = "python")]
#[pymodule]
fn sikulix_py(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add classes
    m.add_class::<PyScreen>()?;
    m.add_class::<PyRegion>()?;
    m.add_class::<PyMatch>()?;
    m.add_class::<PyPattern>()?;

    // Add image finding functions
    m.add_function(wrap_pyfunction!(find, m)?)?;
    m.add_function(wrap_pyfunction!(find_all, m)?)?;
    m.add_function(wrap_pyfunction!(wait, m)?)?;
    m.add_function(wrap_pyfunction!(exists, m)?)?;

    // Add input functions
    m.add_function(wrap_pyfunction!(click, m)?)?;
    m.add_function(wrap_pyfunction!(double_click, m)?)?;
    m.add_function(wrap_pyfunction!(right_click, m)?)?;
    m.add_function(wrap_pyfunction!(type_text, m)?)?;
    m.add_function(wrap_pyfunction!(paste, m)?)?;
    m.add_function(wrap_pyfunction!(hotkey, m)?)?;

    Ok(())
}
