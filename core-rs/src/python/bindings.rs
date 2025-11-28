//! PyO3 bindings for SikuliX Core
//! SikuliX Core の PyO3 バインディング
//!
//! This module provides Python bindings for the core Sikuli-D functionality.
//! このモジュールは、コア Sikuli-D 機能の Python バインディングを提供します。

#[cfg(feature = "python")]
use pyo3::exceptions::{PyIOError, PyRuntimeError, PyValueError};
#[cfg(feature = "python")]
use pyo3::prelude::*;

use crate::{
    ImageMatcher, Key, Keyboard, Match, Mouse, Observer, Pattern, Region, Result, Screen,
    SikulixError,
};

// ============================================================================
// Error Conversion / エラー変換
// ============================================================================

#[cfg(feature = "python")]
fn to_pyerr(err: SikulixError) -> PyErr {
    match err {
        SikulixError::ImageNotFound => PyRuntimeError::new_err("Image not found on screen"),
        SikulixError::ImageLoadError(msg) => {
            PyValueError::new_err(format!("Failed to load image: {}", msg))
        }
        SikulixError::ScreenCaptureError(msg) => {
            PyRuntimeError::new_err(format!("Screen capture failed: {}", msg))
        }
        SikulixError::FindFailed {
            pattern_name,
            timeout_secs,
        } => PyRuntimeError::new_err(format!(
            "FindFailed: {} not found within {}s",
            pattern_name, timeout_secs
        )),
        SikulixError::MouseError(msg) => {
            PyRuntimeError::new_err(format!("Mouse operation failed: {}", msg))
        }
        SikulixError::KeyboardError(msg) => {
            PyRuntimeError::new_err(format!("Keyboard operation failed: {}", msg))
        }
        SikulixError::IoError(e) => PyIOError::new_err(e.to_string()),
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
            let img = self.inner.capture().map_err(to_pyerr)?;
            Ok(img.into_bytes())
        })
    }

    /// Get screen dimensions (width, height)
    /// 画面サイズを取得 (幅, 高さ)
    fn dimensions(&mut self) -> PyResult<(u32, u32)> {
        self.inner.dimensions().map_err(to_pyerr)
    }

    /// Get screen region
    /// 画面領域を取得
    fn get_region(&mut self) -> PyResult<PyRegion> {
        let region = self.inner.get_region().map_err(to_pyerr)?;
        Ok(PyRegion { inner: region })
    }

    /// Get screen width
    /// 画面幅を取得
    #[pyo3(name = "getW")]
    fn get_w(&mut self) -> PyResult<u32> {
        let (w, _) = self.inner.dimensions().map_err(to_pyerr)?;
        Ok(w)
    }

    /// Get screen height
    /// 画面高さを取得
    #[pyo3(name = "getH")]
    fn get_h(&mut self) -> PyResult<u32> {
        let (_, h) = self.inner.dimensions().map_err(to_pyerr)?;
        Ok(h)
    }

    /// Get screen bounds (x, y, w, h)
    /// 画面境界を取得 (x, y, w, h)
    #[pyo3(name = "getBounds")]
    fn get_bounds(&mut self) -> PyResult<(i32, i32, u32, u32)> {
        let region = self.inner.get_region().map_err(to_pyerr)?;
        Ok((region.x, region.y, region.width, region.height))
    }

    /// Get the number of connected screens/monitors
    /// 接続されている画面/モニターの数を取得
    #[staticmethod]
    fn get_number_screens() -> u32 {
        Screen::get_number_screens()
    }

    /// Get the number of connected screens/monitors (alias)
    /// 接続されている画面/モニターの数を取得（エイリアス）
    #[staticmethod]
    #[pyo3(name = "getNumberScreens")]
    fn get_number_screens_alias() -> u32 {
        Screen::get_number_screens()
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
    fn x(&self) -> i32 {
        self.inner.x
    }

    #[getter]
    fn y(&self) -> i32 {
        self.inner.y
    }

    #[getter]
    fn width(&self) -> u32 {
        self.inner.width
    }

    #[getter]
    fn height(&self) -> u32 {
        self.inner.height
    }

    fn __repr__(&self) -> String {
        format!(
            "Region({}, {}, {}, {})",
            self.inner.x, self.inner.y, self.inner.width, self.inner.height
        )
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
        PyRegion {
            inner: self.inner.region,
        }
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
        }
        .map_err(to_pyerr)
    }

    fn __repr__(&self) -> String {
        format!(
            "Match(score={:.2}, region={})",
            self.inner.score,
            format!(
                "Region({}, {}, {}, {})",
                self.inner.region.x,
                self.inner.region.y,
                self.inner.region.width,
                self.inner.region.height
            )
        )
    }
}

// ============================================================================
// PyPattern - Pattern wrapper (lazy loading)
// PyPattern - パターンラッパー（遅延読み込み）
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Pattern")]
#[derive(Clone)]
struct PyPattern {
    /// Image path / 画像パス
    path: String,
    /// Similarity threshold (0.0-1.0) / 類似度閾値 (0.0-1.0)
    similarity_value: f64,
    /// Target offset / ターゲットオフセット
    offset: (i32, i32),
}

#[cfg(feature = "python")]
impl PyPattern {
    /// Convert to Pattern (loads file) / Patternに変換（ファイルを読み込む）
    fn to_pattern(&self) -> std::result::Result<Pattern, SikulixError> {
        let mut pattern = Pattern::from_file(&self.path)?;
        pattern.similarity = self.similarity_value;
        pattern.target_offset = self.offset;
        Ok(pattern)
    }
}

#[cfg(feature = "python")]
#[pymethods]
impl PyPattern {
    /// Create pattern from image path with optional similarity
    /// 画像パスからパターンを作成（オプションで類似度を指定）
    #[new]
    #[pyo3(signature = (path, similarity=None))]
    fn new(path: String, similarity: Option<f64>) -> Self {
        Self {
            path,
            similarity_value: similarity.unwrap_or(0.7),
            offset: (0, 0),
        }
    }

    /// Create new Pattern with different similarity threshold (0.0-1.0)
    /// 異なる類似度閾値で新しいPatternを作成 (0.0-1.0)
    fn similar(&self, similarity: f64) -> Self {
        Self {
            path: self.path.clone(),
            similarity_value: similarity.clamp(0.0, 1.0),
            offset: self.offset,
        }
    }

    /// Create new Pattern with target offset (x, y)
    /// ターゲットオフセット付きの新しいPatternを作成 (x, y)
    #[pyo3(name = "targetOffset")]
    fn target_offset(&self, x: i32, y: i32) -> Self {
        Self {
            path: self.path.clone(),
            similarity_value: self.similarity_value,
            offset: (x, y),
        }
    }

    /// Get similarity threshold
    /// 類似度閾値を取得
    #[getter]
    fn similarity(&self) -> f64 {
        self.similarity_value
    }

    /// Get image path
    /// 画像パスを取得
    #[getter]
    fn get_path(&self) -> &str {
        &self.path
    }

    /// Get filename (without directory)
    /// ファイル名を取得（ディレクトリなし）
    #[pyo3(name = "getFilename")]
    fn get_filename(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }

    fn __repr__(&self) -> String {
        format!("Pattern('{}', {:.2})", self.path, self.similarity_value)
    }
}

// ============================================================================
// PyLocation - Location wrapper
// ============================================================================

#[cfg(feature = "python")]
#[pyclass(name = "Location")]
#[derive(Clone)]
struct PyLocation {
    x: i32,
    y: i32,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyLocation {
    #[new]
    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Get X coordinate
    /// X座標を取得
    #[pyo3(name = "getX")]
    fn get_x(&self) -> i32 {
        self.x
    }

    /// Get Y coordinate
    /// Y座標を取得
    #[pyo3(name = "getY")]
    fn get_y(&self) -> i32 {
        self.y
    }

    /// Set location
    /// 位置を設定
    #[pyo3(name = "setLocation")]
    fn set_location(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    /// Create offset location
    /// オフセット位置を作成
    fn offset(&self, dx: i32, dy: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// Get location above
    /// 上の位置を取得
    fn above(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y - dy,
        }
    }

    /// Get location below
    /// 下の位置を取得
    fn below(&self, dy: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + dy,
        }
    }

    /// Get location to left
    /// 左の位置を取得
    fn left(&self, dx: i32) -> Self {
        Self {
            x: self.x - dx,
            y: self.y,
        }
    }

    /// Get location to right
    /// 右の位置を取得
    fn right(&self, dx: i32) -> Self {
        Self {
            x: self.x + dx,
            y: self.y,
        }
    }

    #[getter(x)]
    fn get_x_prop(&self) -> i32 {
        self.x
    }

    #[getter(y)]
    fn get_y_prop(&self) -> i32 {
        self.y
    }

    fn __repr__(&self) -> String {
        format!("Location({}, {})", self.x, self.y)
    }
}

// ============================================================================
// PySettings - Global Settings / グローバル設定
// ============================================================================

#[cfg(feature = "python")]
use std::sync::RwLock;

/// Global settings storage / グローバル設定ストレージ
#[cfg(feature = "python")]
static SETTINGS: RwLock<GlobalSettings> = RwLock::new(GlobalSettings::new());

/// Internal settings structure / 内部設定構造体
#[cfg(feature = "python")]
struct GlobalSettings {
    /// Minimum similarity threshold (0.0-1.0) / 最小類似度閾値
    min_similarity: f64,
    /// Auto wait timeout in seconds / 自動待機タイムアウト（秒）
    auto_wait_timeout: f64,
    /// Delay before mouse move in seconds / マウス移動前の遅延（秒）
    move_mouse_delay: f64,
    /// Delay after click in seconds / クリック後の遅延（秒）
    click_delay: f64,
    /// Delay between typed characters in seconds / 文字入力間の遅延（秒）
    type_delay: f64,
    /// Observe scan rate (scans per second) / 監視スキャンレート
    observe_scan_rate: f64,
    /// Wait scan rate (scans per second) / 待機スキャンレート
    wait_scan_rate: f64,
    /// Enable highlight / ハイライト有効化
    highlight: bool,
    /// Default highlight duration in seconds / デフォルトハイライト時間（秒）
    default_highlight_time: f64,
}

#[cfg(feature = "python")]
impl GlobalSettings {
    const fn new() -> Self {
        Self {
            min_similarity: 0.7,
            auto_wait_timeout: 3.0,
            move_mouse_delay: 0.3,
            click_delay: 0.0,
            type_delay: 0.0,
            observe_scan_rate: 3.0,
            wait_scan_rate: 3.0,
            highlight: false,
            default_highlight_time: 2.0,
        }
    }
}

/// Python Settings class / Python設定クラス
#[cfg(feature = "python")]
#[pyclass(name = "Settings")]
struct PySettings;

#[cfg(feature = "python")]
#[pymethods]
impl PySettings {
    #[new]
    fn new() -> Self {
        Self
    }

    // ---- MinSimilarity ----
    #[getter(MinSimilarity)]
    fn get_min_similarity(&self) -> f64 {
        SETTINGS.read().unwrap().min_similarity
    }

    #[setter(MinSimilarity)]
    fn set_min_similarity(&self, value: f64) {
        SETTINGS.write().unwrap().min_similarity = value.clamp(0.0, 1.0);
    }

    // ---- AutoWaitTimeout ----
    #[getter(AutoWaitTimeout)]
    fn get_auto_wait_timeout(&self) -> f64 {
        SETTINGS.read().unwrap().auto_wait_timeout
    }

    #[setter(AutoWaitTimeout)]
    fn set_auto_wait_timeout(&self, value: f64) {
        SETTINGS.write().unwrap().auto_wait_timeout = value.max(0.0);
    }

    // ---- MoveMouseDelay ----
    #[getter(MoveMouseDelay)]
    fn get_move_mouse_delay(&self) -> f64 {
        SETTINGS.read().unwrap().move_mouse_delay
    }

    #[setter(MoveMouseDelay)]
    fn set_move_mouse_delay(&self, value: f64) {
        SETTINGS.write().unwrap().move_mouse_delay = value.max(0.0);
    }

    // ---- ClickDelay ----
    #[getter(ClickDelay)]
    fn get_click_delay(&self) -> f64 {
        SETTINGS.read().unwrap().click_delay
    }

    #[setter(ClickDelay)]
    fn set_click_delay(&self, value: f64) {
        SETTINGS.write().unwrap().click_delay = value.max(0.0);
    }

    // ---- TypeDelay ----
    #[getter(TypeDelay)]
    fn get_type_delay(&self) -> f64 {
        SETTINGS.read().unwrap().type_delay
    }

    #[setter(TypeDelay)]
    fn set_type_delay(&self, value: f64) {
        SETTINGS.write().unwrap().type_delay = value.max(0.0);
    }

    // ---- ObserveScanRate ----
    #[getter(ObserveScanRate)]
    fn get_observe_scan_rate(&self) -> f64 {
        SETTINGS.read().unwrap().observe_scan_rate
    }

    #[setter(ObserveScanRate)]
    fn set_observe_scan_rate(&self, value: f64) {
        SETTINGS.write().unwrap().observe_scan_rate = value.max(0.1);
    }

    // ---- WaitScanRate ----
    #[getter(WaitScanRate)]
    fn get_wait_scan_rate(&self) -> f64 {
        SETTINGS.read().unwrap().wait_scan_rate
    }

    #[setter(WaitScanRate)]
    fn set_wait_scan_rate(&self, value: f64) {
        SETTINGS.write().unwrap().wait_scan_rate = value.max(0.1);
    }

    // ---- Highlight ----
    #[getter(Highlight)]
    fn get_highlight(&self) -> bool {
        SETTINGS.read().unwrap().highlight
    }

    #[setter(Highlight)]
    fn set_highlight(&self, value: bool) {
        SETTINGS.write().unwrap().highlight = value;
    }

    // ---- DefaultHighlightTime ----
    #[getter(DefaultHighlightTime)]
    fn get_default_highlight_time(&self) -> f64 {
        SETTINGS.read().unwrap().default_highlight_time
    }

    #[setter(DefaultHighlightTime)]
    fn set_default_highlight_time(&self, value: f64) {
        SETTINGS.write().unwrap().default_highlight_time = value.max(0.0);
    }

    fn __repr__(&self) -> String {
        let s = SETTINGS.read().unwrap();
        format!(
            "Settings(MinSimilarity={:.2}, AutoWaitTimeout={:.1}, MoveMouseDelay={:.2}, ClickDelay={:.2}, TypeDelay={:.3})",
            s.min_similarity, s.auto_wait_timeout, s.move_mouse_delay, s.click_delay, s.type_delay
        )
    }
}

/// Helper to apply move mouse delay / マウス移動遅延を適用するヘルパー
#[cfg(feature = "python")]
fn apply_move_delay() {
    let delay = SETTINGS.read().unwrap().move_mouse_delay;
    if delay > 0.0 {
        std::thread::sleep(std::time::Duration::from_secs_f64(delay));
    }
}

/// Helper to apply click delay / クリック遅延を適用するヘルパー
#[cfg(feature = "python")]
fn apply_click_delay() {
    let delay = SETTINGS.read().unwrap().click_delay;
    if delay > 0.0 {
        std::thread::sleep(std::time::Duration::from_secs_f64(delay));
    }
}

/// Helper to apply type delay / 入力遅延を適用するヘルパー
#[cfg(feature = "python")]
fn apply_type_delay() {
    let delay = SETTINGS.read().unwrap().type_delay;
    if delay > 0.0 {
        std::thread::sleep(std::time::Duration::from_secs_f64(delay));
    }
}

// ============================================================================
// Module-Level Functions / モジュールレベル関数
// ============================================================================

/// Find pattern on screen (best match)
/// 画面上でパターンを検索 (最良マッチ)
/// Returns None if image file doesn't exist (SikuliX compatible)
/// 画像ファイルが存在しない場合はNoneを返す（SikuliX互換）
#[cfg(feature = "python")]
#[pyfunction]
fn find(py: Python, pattern: &str, similarity: Option<f64>) -> PyResult<Option<PyMatch>> {
    py.allow_threads(|| {
        let screen = Screen::primary();

        // Return None if pattern file doesn't exist (SikuliX compatible)
        // パターンファイルが存在しない場合はNoneを返す（SikuliX互換）
        let pat = match Pattern::from_file(pattern) {
            Ok(p) => p.similar(similarity.unwrap_or(0.7)),
            Err(SikulixError::IoError(_)) => return Ok(None),
            Err(e) => return Err(to_pyerr(e)),
        };

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

        let result = matcher
            .wait(&screen, &pat, timeout.unwrap_or(3.0))
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

        let result = matcher
            .exists(&screen, &pat, timeout.unwrap_or(0.0))
            .map_err(to_pyerr)?;

        Ok(result.map(|m| PyMatch { inner: m }))
    })
}

// ============================================================================
// Mouse/Keyboard Input Functions / マウス・キーボード入力関数
// ============================================================================

/// Move mouse to location
/// マウスを位置に移動
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "mouseMove")]
fn mouse_move(target: &PyAny) -> PyResult<()> {
    let (x, y) = resolve_target(target)?;
    apply_move_delay();
    Mouse::move_to(x, y).map_err(to_pyerr)
}

/// Hover (alias for mouseMove)
/// ホバー（mouseMoveのエイリアス）
#[cfg(feature = "python")]
#[pyfunction]
fn hover(target: &PyAny) -> PyResult<()> {
    mouse_move(target)
}

/// Resolve target to (x, y) coordinates
/// ターゲットを (x, y) 座標に解決
#[cfg(feature = "python")]
fn resolve_target(target: &PyAny) -> PyResult<(i32, i32)> {
    // Check if it's a tuple
    if let Ok((x, y)) = target.extract::<(i32, i32)>() {
        return Ok((x, y));
    }

    // Check if it's a PyLocation
    if let Ok(loc) = target.extract::<PyRef<PyLocation>>() {
        return Ok((loc.x, loc.y));
    }

    // Check if it's a PyMatch (use target() method)
    if let Ok(m) = target.extract::<PyRef<PyMatch>>() {
        return Ok(m.target());
    }

    // Check if it's a PyRegion (use center)
    if let Ok(r) = target.extract::<PyRef<PyRegion>>() {
        return Ok(r.center());
    }

    Err(PyValueError::new_err(
        "Invalid target: expected (x, y), Location, Match, or Region",
    ))
}

/// Click at coordinates or current position
/// 座標または現在位置でクリック
#[cfg(feature = "python")]
#[pyfunction]
fn click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        apply_move_delay();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::click().map_err(to_pyerr)?;
    apply_click_delay();
    Ok(())
}

/// Double click at coordinates or current position
/// 座標または現在位置でダブルクリック
#[cfg(feature = "python")]
#[pyfunction]
fn double_click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        apply_move_delay();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::double_click().map_err(to_pyerr)?;
    apply_click_delay();
    Ok(())
}

/// Right click at coordinates or current position
/// 座標または現在位置で右クリック
#[cfg(feature = "python")]
#[pyfunction]
fn right_click(x: Option<i32>, y: Option<i32>) -> PyResult<()> {
    if let (Some(x), Some(y)) = (x, y) {
        apply_move_delay();
        Mouse::move_to(x, y).map_err(to_pyerr)?;
    }
    Mouse::right_click().map_err(to_pyerr)?;
    apply_click_delay();
    Ok(())
}

/// Type text with optional character delay
/// オプションの文字遅延でテキストを入力
#[cfg(feature = "python")]
#[pyfunction]
fn type_text(text: &str) -> PyResult<()> {
    let delay = SETTINGS.read().unwrap().type_delay;
    if delay > 0.0 {
        // Type each character with delay / 遅延を入れて1文字ずつ入力
        for c in text.chars() {
            Keyboard::type_text(&c.to_string()).map_err(to_pyerr)?;
            std::thread::sleep(std::time::Duration::from_secs_f64(delay));
        }
        Ok(())
    } else {
        Keyboard::type_text(text).map_err(to_pyerr)
    }
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
    let key_enum: Vec<Key> = keys
        .iter()
        .map(|s| parse_key(s))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| PyValueError::new_err(e))?;

    Keyboard::hotkey(&key_enum).map_err(to_pyerr)
}

// ============================================================================
// Scroll Functions / スクロール関数
// ============================================================================

/// Scroll vertically (positive = up, negative = down)
/// 垂直スクロール（正 = 上、負 = 下）
#[cfg(feature = "python")]
#[pyfunction]
fn scroll(clicks: i32) -> PyResult<()> {
    Mouse::scroll(clicks).map_err(to_pyerr)
}

/// Scroll up by specified number of clicks
/// 指定クリック数だけ上にスクロール
#[cfg(feature = "python")]
#[pyfunction]
fn scroll_up(clicks: Option<u32>) -> PyResult<()> {
    Mouse::scroll_up(clicks.unwrap_or(3)).map_err(to_pyerr)
}

/// Scroll down by specified number of clicks
/// 指定クリック数だけ下にスクロール
#[cfg(feature = "python")]
#[pyfunction]
fn scroll_down(clicks: Option<u32>) -> PyResult<()> {
    Mouse::scroll_down(clicks.unwrap_or(3)).map_err(to_pyerr)
}

/// Scroll horizontally (positive = right, negative = left)
/// 水平スクロール（正 = 右、負 = 左）
#[cfg(feature = "python")]
#[pyfunction]
fn scroll_horizontal(clicks: i32) -> PyResult<()> {
    Mouse::scroll_horizontal(clicks).map_err(to_pyerr)
}

/// Scroll left by specified number of clicks
/// 指定クリック数だけ左にスクロール
#[cfg(feature = "python")]
#[pyfunction]
fn scroll_left(clicks: Option<u32>) -> PyResult<()> {
    Mouse::scroll_left(clicks.unwrap_or(3)).map_err(to_pyerr)
}

/// Scroll right by specified number of clicks
/// 指定クリック数だけ右にスクロール
#[cfg(feature = "python")]
#[pyfunction]
fn scroll_right(clicks: Option<u32>) -> PyResult<()> {
    Mouse::scroll_right(clicks.unwrap_or(3)).map_err(to_pyerr)
}

// ============================================================================
// Mouse Button Functions / マウスボタン関数
// ============================================================================

/// Press and hold the left mouse button
/// 左マウスボタンを押し続ける
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "mouseDown")]
fn mouse_down() -> PyResult<()> {
    Mouse::mouse_down().map_err(to_pyerr)
}

/// Release the left mouse button
/// 左マウスボタンを離す
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "mouseUp")]
fn mouse_up() -> PyResult<()> {
    Mouse::mouse_up().map_err(to_pyerr)
}

// ============================================================================
// Drag Functions / ドラッグ関数
// ============================================================================

/// Drag from current position to target
/// 現在位置からターゲットへドラッグ
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "dragTo")]
fn drag_to(x: i32, y: i32) -> PyResult<()> {
    apply_move_delay();
    Mouse::drag_to(x, y).map_err(to_pyerr)
}

/// Drag from one position to another
/// ある位置から別の位置へドラッグ
#[cfg(feature = "python")]
#[pyfunction]
fn drag(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> PyResult<()> {
    apply_move_delay();
    Mouse::drag(from_x, from_y, to_x, to_y).map_err(to_pyerr)
}

/// Drag and drop - move to start, drag to end
/// ドラッグ＆ドロップ - 開始位置に移動し、終了位置へドラッグ
#[cfg(feature = "python")]
#[pyfunction]
#[pyo3(name = "dragDrop")]
fn drag_drop(start_x: i32, start_y: i32, end_x: i32, end_y: i32) -> PyResult<()> {
    apply_move_delay();
    Mouse::move_to(start_x, start_y).map_err(to_pyerr)?;
    Mouse::drag_to(end_x, end_y).map_err(to_pyerr)
}

// ============================================================================
// PyObserver - Screen Region Observer / 画面領域オブザーバー
// ============================================================================

/// Python Observer for screen region monitoring
/// 画面領域監視用Pythonオブザーバー
///
/// Provides event-driven monitoring for pattern appearance, vanishing, and changes.
/// パターンの出現、消失、変化のイベント駆動型監視を提供します。
#[cfg(feature = "python")]
#[pyclass(name = "Observer")]
struct PyObserver {
    /// Region to observe / 監視対象領域
    region: Region,
    /// Running state / 実行状態
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// Observation interval in milliseconds / 監視間隔（ミリ秒）
    interval_ms: u64,
    /// Appear handlers (pattern_path, similarity, callback) / 出現ハンドラー
    appear_handlers: Vec<(String, f64, Py<PyAny>)>,
    /// Vanish handlers (pattern_path, similarity, callback) / 消失ハンドラー
    vanish_handlers: Vec<(String, f64, Py<PyAny>)>,
    /// Change handlers (threshold, callback) / 変化ハンドラー
    change_handlers: Vec<(f64, Py<PyAny>)>,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyObserver {
    /// Create a new Observer for a Region
    /// Regionに対する新しいObserverを作成
    #[new]
    fn new(region: &PyRegion) -> Self {
        Self {
            region: region.inner,
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            interval_ms: 500,
            appear_handlers: Vec::new(),
            vanish_handlers: Vec::new(),
            change_handlers: Vec::new(),
        }
    }

    /// Set observation interval in milliseconds
    /// 監視間隔をミリ秒で設定
    #[pyo3(name = "setInterval")]
    fn set_interval(&mut self, interval_ms: u64) {
        self.interval_ms = interval_ms.max(10);
    }

    /// Register a callback for when a pattern appears
    /// パターンが出現した時のコールバックを登録
    ///
    /// Args:
    ///     pattern: Image path / 画像パス
    ///     callback: Function called with Match object / Matchオブジェクトと呼ばれる関数
    ///     similarity: Optional similarity threshold (default 0.7) / 類似度閾値
    #[pyo3(name = "onAppear")]
    fn on_appear(&mut self, pattern: &str, callback: Py<PyAny>, similarity: Option<f64>) {
        self.appear_handlers
            .push((pattern.to_string(), similarity.unwrap_or(0.7), callback));
    }

    /// Register a callback for when a pattern vanishes
    /// パターンが消失した時のコールバックを登録
    ///
    /// Args:
    ///     pattern: Image path / 画像パス
    ///     callback: Function called when pattern vanishes / パターン消失時に呼ばれる関数
    ///     similarity: Optional similarity threshold (default 0.7) / 類似度閾値
    #[pyo3(name = "onVanish")]
    fn on_vanish(&mut self, pattern: &str, callback: Py<PyAny>, similarity: Option<f64>) {
        self.vanish_handlers
            .push((pattern.to_string(), similarity.unwrap_or(0.7), callback));
    }

    /// Register a callback for visual changes in the region
    /// 領域の視覚的変化のコールバックを登録
    ///
    /// Args:
    ///     threshold: Change threshold (0.0 - 1.0) / 変化閾値
    ///     callback: Function called with change amount / 変化量と呼ばれる関数
    #[pyo3(name = "onChange")]
    fn on_change(&mut self, threshold: f64, callback: Py<PyAny>) {
        self.change_handlers
            .push((threshold.clamp(0.0, 1.0), callback));
    }

    /// Start observing with timeout (blocking)
    /// タイムアウト付きで監視開始（ブロッキング）
    ///
    /// Args:
    ///     timeout: Maximum observation time in seconds (0 = infinite) / 最大監視時間（秒）
    fn observe(&self, py: Python, timeout: Option<f64>) -> PyResult<()> {
        use std::sync::atomic::Ordering;
        use std::time::{Duration, Instant};

        self.running.store(true, Ordering::SeqCst);
        let start = Instant::now();
        let timeout_duration = timeout.map(|t| Duration::from_secs_f64(t));
        let interval = Duration::from_millis(self.interval_ms);

        // Track pattern visibility for vanish detection
        // 消失検出用のパターン可視性を追跡
        let mut pattern_visible: Vec<bool> = vec![false; self.vanish_handlers.len()];
        let mut last_capture: Option<image::DynamicImage> = None;

        let screen = Screen::primary();
        let matcher = ImageMatcher::new();

        while self.running.load(Ordering::SeqCst) {
            // Check timeout
            if let Some(td) = timeout_duration {
                if start.elapsed() >= td {
                    break;
                }
            }

            // Allow Python threads during capture
            let screenshot = py
                .allow_threads(|| screen.capture_region(&self.region))
                .map_err(to_pyerr)?;

            // Process appear handlers
            for (pattern_path, similarity, callback) in &self.appear_handlers {
                if let Ok(pat) = Pattern::from_file(pattern_path) {
                    let pat = pat.similar(*similarity);
                    if let Ok(Some(m)) = matcher.find(&screenshot, &pat) {
                        let py_match = PyMatch { inner: m };
                        let _ = callback.call1(py, (py_match,));
                    }
                }
            }

            // Process vanish handlers
            for (i, (pattern_path, similarity, callback)) in self.vanish_handlers.iter().enumerate()
            {
                if let Ok(pat) = Pattern::from_file(pattern_path) {
                    let pat = pat.similar(*similarity);
                    match matcher.find(&screenshot, &pat) {
                        Ok(Some(_)) => {
                            pattern_visible[i] = true;
                        }
                        Ok(None) => {
                            if pattern_visible[i] {
                                // Pattern vanished
                                let _ = callback.call0(py);
                                pattern_visible[i] = false;
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            // Process change handlers
            if !self.change_handlers.is_empty() {
                if let Some(ref last) = last_capture {
                    let change = calculate_change(last, &screenshot);
                    for (threshold, callback) in &self.change_handlers {
                        if change >= *threshold {
                            let _ = callback.call1(py, (change,));
                        }
                    }
                }
                last_capture = Some(screenshot);
            }

            // Sleep between observations
            py.allow_threads(|| {
                std::thread::sleep(interval);
            });
        }

        self.running.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Stop observation
    /// 監視を停止
    fn stop(&self) {
        use std::sync::atomic::Ordering;
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if observer is running
    /// オブザーバーが実行中か確認
    #[pyo3(name = "isRunning")]
    fn is_running(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.running.load(Ordering::SeqCst)
    }

    fn __repr__(&self) -> String {
        format!(
            "Observer(region=({}, {}, {}, {}), interval={}ms, appear={}, vanish={}, change={})",
            self.region.x,
            self.region.y,
            self.region.width,
            self.region.height,
            self.interval_ms,
            self.appear_handlers.len(),
            self.vanish_handlers.len(),
            self.change_handlers.len()
        )
    }
}

/// Calculate change between two images (helper)
/// 2つの画像間の変化を計算（ヘルパー）
#[cfg(feature = "python")]
fn calculate_change(img1: &image::DynamicImage, img2: &image::DynamicImage) -> f64 {
    let gray1 = img1.to_luma8();
    let gray2 = img2.to_luma8();

    if gray1.dimensions() != gray2.dimensions() {
        return 1.0;
    }

    let (width, height) = gray1.dimensions();
    let total_pixels = (width * height) as f64;

    let sum_squared_diff: f64 = gray1
        .pixels()
        .zip(gray2.pixels())
        .map(|(p1, p2)| {
            let diff = p1[0] as f64 - p2[0] as f64;
            diff * diff
        })
        .sum();

    let mse = sum_squared_diff / total_pixels;
    (mse / (255.0 * 255.0)).min(1.0)
}

// ============================================================================
// Key Parsing / キー解析
// ============================================================================

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
                    'A' => Ok(Key::A),
                    'B' => Ok(Key::B),
                    'C' => Ok(Key::C),
                    'D' => Ok(Key::D),
                    'E' => Ok(Key::E),
                    'F' => Ok(Key::F),
                    'G' => Ok(Key::G),
                    'H' => Ok(Key::H),
                    'I' => Ok(Key::I),
                    'J' => Ok(Key::J),
                    'K' => Ok(Key::K),
                    'L' => Ok(Key::L),
                    'M' => Ok(Key::M),
                    'N' => Ok(Key::N),
                    'O' => Ok(Key::O),
                    'P' => Ok(Key::P),
                    'Q' => Ok(Key::Q),
                    'R' => Ok(Key::R),
                    'S' => Ok(Key::S),
                    'T' => Ok(Key::T),
                    'U' => Ok(Key::U),
                    'V' => Ok(Key::V),
                    'W' => Ok(Key::W),
                    'X' => Ok(Key::X),
                    'Y' => Ok(Key::Y),
                    'Z' => Ok(Key::Z),
                    _ => Err(format!("Unknown key: {}", s)),
                }
            } else if c.is_ascii_digit() {
                match c {
                    '0' => Ok(Key::Num0),
                    '1' => Ok(Key::Num1),
                    '2' => Ok(Key::Num2),
                    '3' => Ok(Key::Num3),
                    '4' => Ok(Key::Num4),
                    '5' => Ok(Key::Num5),
                    '6' => Ok(Key::Num6),
                    '7' => Ok(Key::Num7),
                    '8' => Ok(Key::Num8),
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
fn sikulid(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add classes
    m.add_class::<PyScreen>()?;
    m.add_class::<PyRegion>()?;
    m.add_class::<PyMatch>()?;
    m.add_class::<PyPattern>()?;
    m.add_class::<PyLocation>()?;
    m.add_class::<PySettings>()?;
    m.add_class::<PyObserver>()?;

    // Add image finding functions
    m.add_function(wrap_pyfunction!(find, m)?)?;
    m.add_function(wrap_pyfunction!(find_all, m)?)?;
    m.add_function(wrap_pyfunction!(wait, m)?)?;
    m.add_function(wrap_pyfunction!(exists, m)?)?;

    // Add input functions
    m.add_function(wrap_pyfunction!(mouse_move, m)?)?;
    m.add_function(wrap_pyfunction!(hover, m)?)?;
    m.add_function(wrap_pyfunction!(click, m)?)?;
    m.add_function(wrap_pyfunction!(double_click, m)?)?;
    m.add_function(wrap_pyfunction!(right_click, m)?)?;
    m.add_function(wrap_pyfunction!(type_text, m)?)?;
    m.add_function(wrap_pyfunction!(paste, m)?)?;
    m.add_function(wrap_pyfunction!(hotkey, m)?)?;

    // Add scroll functions
    m.add_function(wrap_pyfunction!(scroll, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_up, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_down, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_horizontal, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_left, m)?)?;
    m.add_function(wrap_pyfunction!(scroll_right, m)?)?;

    // Add mouse button functions
    m.add_function(wrap_pyfunction!(mouse_down, m)?)?;
    m.add_function(wrap_pyfunction!(mouse_up, m)?)?;

    // Add drag functions
    m.add_function(wrap_pyfunction!(drag, m)?)?;
    m.add_function(wrap_pyfunction!(drag_to, m)?)?;
    m.add_function(wrap_pyfunction!(drag_drop, m)?)?;

    Ok(())
}
