//! Screen capture and input control module
//! スクリーンキャプチャと入力制御モジュール
//!
//! This module provides cross-platform screen capture, mouse control,
//! and keyboard input functionality.
//! このモジュールはクロスプラットフォームのスクリーンキャプチャ、マウス制御、
//! キーボード入力機能を提供します。
//!
//! # Example / 使用例
//!
//! ```no_run
//! use sikulid::screen::{Screen, Mouse, Keyboard, Key};
//!
//! // Capture screen / スクリーンキャプチャ
//! let mut screen = Screen::primary();
//! let screenshot = screen.capture().unwrap();
//!
//! // Mouse control / マウス制御
//! Mouse::move_to(100, 100).unwrap();
//! Mouse::click().unwrap();
//!
//! // Keyboard control / キーボード制御
//! Keyboard::type_text("Hello").unwrap();
//! Keyboard::hotkey(&[Key::Ctrl, Key::S]).unwrap();
//! ```

use crate::image::ImageMatcher;
use crate::{Match, Pattern, Region, Result, SikulixError};
use image::DynamicImage;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

/// Coordinate conversion utilities for DPI scaling
/// DPIスケーリング用の座標変換ユーティリティ
pub mod coordinates;

/// Screen capture and control
/// スクリーンキャプチャと制御
///
/// Provides methods for capturing screenshots and querying screen properties.
/// Supports multi-monitor setups with index-based monitor selection.
/// スクリーンショットの取得と画面プロパティの照会メソッドを提供します。
/// インデックスベースのモニター選択によるマルチモニター設定をサポートします。
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::Screen;
///
/// let mut screen = Screen::primary();
/// let (width, height) = screen.dimensions().unwrap();
/// let screenshot = screen.capture().unwrap();
/// ```
pub struct Screen {
    /// Screen index (0 = primary) / スクリーンインデックス（0 = プライマリ）
    index: u32,
    /// Cached screen dimensions / キャッシュされた画面サイズ
    dimensions: Option<(u32, u32)>,
}

impl Default for Screen {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Screen {
    /// Create a new Screen instance for the given monitor index
    pub fn new(index: u32) -> Self {
        Self {
            index,
            dimensions: None,
        }
    }

    /// Get the screen index
    /// スクリーンインデックスを取得
    pub fn get_index(&self) -> u32 {
        self.index
    }

    /// Get the primary screen
    pub fn primary() -> Self {
        Self::new(0)
    }

    /// Get all connected screens
    /// 接続されているすべてのスクリーンを取得
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Screen;
    ///
    /// let screens = Screen::all();
    /// for screen in screens {
    ///     println!("Screen {}", screen.get_index());
    /// }
    /// ```
    pub fn all() -> Vec<Screen> {
        let count = Self::get_number_screens();
        (0..count).map(Screen::new).collect()
    }

    /// Get DPI scale factor for this screen
    /// このスクリーンのDPIスケールファクターを取得
    ///
    /// Returns 1.0 (100%) by default if scale factor cannot be determined.
    /// スケールファクターを取得できない場合は1.0（100%）を返します。
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Screen;
    ///
    /// let screen = Screen::primary();
    /// let scale = screen.get_scale_factor();
    /// println!("Scale factor: {}%", scale * 100.0);
    /// ```
    pub fn get_scale_factor(&self) -> f64 {
        Self::get_scale_factor_impl(self.index)
    }

    #[cfg(target_os = "windows")]
    fn get_scale_factor_impl(index: u32) -> f64 {
        windows::get_monitor_info(index)
            .map(|m| m.scale_factor)
            .unwrap_or(1.0)
    }

    #[cfg(target_os = "macos")]
    fn get_scale_factor_impl(_index: u32) -> f64 {
        // TODO: Implement macOS DPI detection
        1.0
    }

    #[cfg(target_os = "linux")]
    fn get_scale_factor_impl(_index: u32) -> f64 {
        // TODO: Implement Linux DPI detection
        1.0
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_scale_factor_impl(_index: u32) -> f64 {
        1.0
    }

    /// Get the number of connected screens/monitors
    /// 接続されている画面/モニターの数を取得
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::Screen;
    ///
    /// let num_screens = Screen::get_number_screens();
    /// println!("Number of monitors: {}", num_screens);
    /// ```
    pub fn get_number_screens() -> u32 {
        Self::get_number_screens_impl()
    }

    #[cfg(target_os = "windows")]
    fn get_number_screens_impl() -> u32 {
        windows::get_number_screens()
    }

    #[cfg(target_os = "macos")]
    fn get_number_screens_impl() -> u32 {
        macos::get_number_screens()
    }

    #[cfg(target_os = "linux")]
    fn get_number_screens_impl() -> u32 {
        linux::get_number_screens()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_number_screens_impl() -> u32 {
        1
    }

    /// Get screen dimensions (width, height)
    pub fn dimensions(&mut self) -> Result<(u32, u32)> {
        if let Some(dims) = self.dimensions {
            return Ok(dims);
        }

        let dims = self.get_dimensions_impl()?;
        self.dimensions = Some(dims);
        Ok(dims)
    }

    /// Capture the entire screen
    pub fn capture(&self) -> Result<DynamicImage> {
        self.capture_impl()
    }

    /// Capture a specific region of the screen
    pub fn capture_region(&self, region: &Region) -> Result<DynamicImage> {
        self.capture_region_impl(region)
    }

    /// Get the full screen region
    pub fn get_region(&mut self) -> Result<Region> {
        let (w, h) = self.dimensions()?;
        Ok(Region::new(0, 0, w, h))
    }

    /// Find a pattern on the screen
    /// 画面上でパターンを検索
    ///
    /// # Arguments / 引数
    ///
    /// * `pattern` - Pattern to find / 検索するパターン
    ///
    /// # Returns / 戻り値
    ///
    /// Returns the Match if found, or an error if not found
    /// 見つかった場合はMatch、見つからない場合はエラーを返す
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::{Screen, Pattern};
    ///
    /// let screen = Screen::primary();
    /// let pattern = Pattern::from_file("button.png").unwrap();
    /// let match_result = screen.find(&pattern).unwrap();
    /// println!("Found at: ({}, {})", match_result.region.x, match_result.region.y);
    /// ```
    pub fn find(&self, pattern: &Pattern) -> Result<Match> {
        let screenshot = self.capture()?;
        let matcher = ImageMatcher::new();

        match matcher.find(&screenshot, pattern)? {
            Some(match_result) => Ok(match_result),
            None => Err(SikulixError::ImageNotFound),
        }
    }

    /// Find all occurrences of a pattern on the screen
    /// 画面上のパターンの全出現箇所を検索
    pub fn find_all(&self, pattern: &Pattern) -> Result<Vec<Match>> {
        let screenshot = self.capture()?;
        let matcher = ImageMatcher::new();
        matcher.find_all(&screenshot, pattern)
    }

    /// Check if a pattern exists on the screen
    /// 画面上にパターンが存在するかチェック
    pub fn exists(&self, pattern: &Pattern) -> Result<bool> {
        let screenshot = self.capture()?;
        let matcher = ImageMatcher::new();
        Ok(matcher.find(&screenshot, pattern)?.is_some())
    }

    // Platform-specific implementations
    #[cfg(target_os = "windows")]
    fn get_dimensions_impl(&self) -> Result<(u32, u32)> {
        windows::get_screen_dimensions(self.index)
    }

    #[cfg(target_os = "windows")]
    fn capture_impl(&self) -> Result<DynamicImage> {
        windows::capture_screen(self.index)
    }

    #[cfg(target_os = "windows")]
    fn capture_region_impl(&self, region: &Region) -> Result<DynamicImage> {
        windows::capture_region(region)
    }

    #[cfg(target_os = "macos")]
    fn get_dimensions_impl(&self) -> Result<(u32, u32)> {
        macos::get_screen_dimensions(self.index)
    }

    #[cfg(target_os = "macos")]
    fn capture_impl(&self) -> Result<DynamicImage> {
        macos::capture_screen(self.index)
    }

    #[cfg(target_os = "macos")]
    fn capture_region_impl(&self, region: &Region) -> Result<DynamicImage> {
        macos::capture_region(region)
    }

    #[cfg(target_os = "linux")]
    fn get_dimensions_impl(&self) -> Result<(u32, u32)> {
        linux::get_screen_dimensions(self.index)
    }

    #[cfg(target_os = "linux")]
    fn capture_impl(&self) -> Result<DynamicImage> {
        linux::capture_screen(self.index)
    }

    #[cfg(target_os = "linux")]
    fn capture_region_impl(&self, region: &Region) -> Result<DynamicImage> {
        linux::capture_region(region)
    }

    // Fallback for unsupported platforms
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn get_dimensions_impl(&self) -> Result<(u32, u32)> {
        Err(SikulixError::ScreenCaptureError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn capture_impl(&self) -> Result<DynamicImage> {
        Err(SikulixError::ScreenCaptureError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn capture_region_impl(&self, _region: &Region) -> Result<DynamicImage> {
        Err(SikulixError::ScreenCaptureError(
            "Unsupported platform".to_string(),
        ))
    }
}

/// Mouse control
/// マウス制御
///
/// Provides methods for mouse movement and clicking.
/// All operations are immediate and do not require an instance.
/// マウスの移動とクリックのメソッドを提供します。
/// すべての操作は即時実行され、インスタンスを必要としません。
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::Mouse;
///
/// // Move and click / 移動してクリック
/// Mouse::move_to(500, 300).unwrap();
/// Mouse::click().unwrap();
///
/// // Double-click / ダブルクリック
/// Mouse::double_click().unwrap();
///
/// // Right-click / 右クリック
/// Mouse::right_click().unwrap();
/// ```
pub struct Mouse;

impl Mouse {
    /// Move mouse to absolute position
    /// マウスを絶対座標に移動
    pub fn move_to(x: i32, y: i32) -> Result<()> {
        Self::move_to_impl(x, y)
    }

    /// Move mouse to absolute position with smooth animation
    /// マウスを絶対座標にスムーズアニメーションで移動
    ///
    /// # Arguments / 引数
    /// * `x` - Target X coordinate / ターゲットX座標
    /// * `y` - Target Y coordinate / ターゲットY座標
    /// * `duration_ms` - Animation duration in milliseconds / アニメーション時間（ミリ秒）
    pub fn move_to_smooth(x: i32, y: i32, duration_ms: u64) -> Result<()> {
        let (start_x, start_y) = Self::position()?;

        // Use 60fps-ish interpolation
        let steps = (duration_ms / 16).max(1) as u32;
        let step_delay = duration_ms / steps as u64;

        for i in 1..=steps {
            // Use ease-in-out interpolation for natural movement
            let t = i as f64 / steps as f64;
            let eased_t = if t < 0.5 {
                2.0 * t * t
            } else {
                -1.0 + (4.0 - 2.0 * t) * t
            };

            let curr_x = start_x + ((x - start_x) as f64 * eased_t) as i32;
            let curr_y = start_y + ((y - start_y) as f64 * eased_t) as i32;
            Self::move_to(curr_x, curr_y)?;
            std::thread::sleep(std::time::Duration::from_millis(step_delay));
        }

        // Ensure we end at exact position
        Self::move_to(x, y)
    }

    /// Click at current position
    /// 現在位置でクリック
    pub fn click() -> Result<()> {
        Self::click_impl()
    }

    /// Double click at current position
    /// 現在位置でダブルクリック
    pub fn double_click() -> Result<()> {
        Self::click_impl()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::click_impl()
    }

    /// Right click at current position
    /// 現在位置で右クリック
    pub fn right_click() -> Result<()> {
        Self::right_click_impl()
    }

    /// Get current mouse position
    /// 現在のマウス位置を取得
    pub fn position() -> Result<(i32, i32)> {
        Self::position_impl()
    }

    /// Press mouse button down (without releasing)
    /// マウスボタンを押下（解放しない）
    pub fn mouse_down() -> Result<()> {
        Self::mouse_down_impl()
    }

    /// Release mouse button
    /// マウスボタンを解放
    pub fn mouse_up() -> Result<()> {
        Self::mouse_up_impl()
    }

    /// Middle click at current position
    /// 現在位置で中クリック
    pub fn middle_click() -> Result<()> {
        Self::middle_click_impl()
    }

    /// Scroll mouse wheel vertically
    /// マウスホイールを垂直スクロール
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `clicks` - Number of wheel clicks (positive = up/away from user, negative = down/toward user)
    ///   ホイールクリック数（正 = 上/ユーザーから離れる方向、負 = 下/ユーザー方向）
    pub fn scroll(clicks: i32) -> Result<()> {
        Self::scroll_impl(clicks)
    }

    /// Scroll mouse wheel up (away from user)
    /// マウスホイールを上スクロール（ユーザーから離れる方向）
    pub fn scroll_up(clicks: u32) -> Result<()> {
        Self::scroll_impl(clicks as i32)
    }

    /// Scroll mouse wheel down (toward user)
    /// マウスホイールを下スクロール（ユーザー方向）
    pub fn scroll_down(clicks: u32) -> Result<()> {
        Self::scroll_impl(-(clicks as i32))
    }

    /// Scroll mouse wheel horizontally
    /// マウスホイールを水平スクロール
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `clicks` - Number of wheel clicks (positive = right, negative = left)
    ///   ホイールクリック数（正 = 右、負 = 左）
    pub fn scroll_horizontal(clicks: i32) -> Result<()> {
        Self::scroll_horizontal_impl(clicks)
    }

    /// Scroll mouse wheel right
    /// マウスホイールを右スクロール
    pub fn scroll_right(clicks: u32) -> Result<()> {
        Self::scroll_horizontal_impl(clicks as i32)
    }

    /// Scroll mouse wheel left
    /// マウスホイールを左スクロール
    pub fn scroll_left(clicks: u32) -> Result<()> {
        Self::scroll_horizontal_impl(-(clicks as i32))
    }

    /// Hover over a position (move without clicking)
    /// 位置にホバー（クリックなしで移動）
    ///
    /// SikuliX-compatible hover operation. Moves mouse to position and optionally waits.
    /// SikuliX互換のホバー操作。マウスを位置に移動し、オプションで待機します。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `x` - Target X coordinate / ターゲットX座標
    /// * `y` - Target Y coordinate / ターゲットY座標
    pub fn hover(x: i32, y: i32) -> Result<()> {
        Self::move_to(x, y)
    }

    /// Hover over a position with delay
    /// 遅延付きで位置にホバー
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `x` - Target X coordinate / ターゲットX座標
    /// * `y` - Target Y coordinate / ターゲットY座標
    /// * `delay_ms` - Time to wait after hover in milliseconds / ホバー後の待機時間（ミリ秒）
    pub fn hover_with_delay(x: i32, y: i32, delay_ms: u64) -> Result<()> {
        Self::move_to(x, y)?;
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        Ok(())
    }

    /// Click with a pre-delay
    /// プレディレイ付きクリック
    ///
    /// # Arguments / 引数
    /// * `delay_ms` - Delay before click in milliseconds / クリック前の遅延（ミリ秒）
    pub fn click_with_delay(delay_ms: u64) -> Result<()> {
        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        Self::click()
    }

    /// Double click with delay between clicks
    /// クリック間ディレイ付きダブルクリック
    ///
    /// # Arguments / 引数
    /// * `interval_ms` - Delay between clicks in milliseconds / クリック間の遅延（ミリ秒）
    pub fn double_click_with_interval(interval_ms: u64) -> Result<()> {
        Self::click_impl()?;
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
        Self::click_impl()
    }

    /// Triple click at current position
    /// 現在位置でトリプルクリック
    pub fn triple_click() -> Result<()> {
        Self::click_impl()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::click_impl()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::click_impl()
    }

    /// Click multiple times with specified interval
    /// 指定された間隔で複数回クリック
    ///
    /// # Arguments / 引数
    /// * `count` - Number of clicks / クリック回数
    /// * `interval_ms` - Delay between clicks in milliseconds / クリック間の遅延（ミリ秒）
    pub fn multi_click(count: u32, interval_ms: u64) -> Result<()> {
        for i in 0..count {
            Self::click_impl()?;
            if i < count - 1 {
                std::thread::sleep(std::time::Duration::from_millis(interval_ms));
            }
        }
        Ok(())
    }

    /// Move to position and click with optional delay
    /// 位置に移動してクリック（オプションのディレイ付き）
    ///
    /// # Arguments / 引数
    /// * `x` - X coordinate / X座標
    /// * `y` - Y coordinate / Y座標
    /// * `delay_ms` - Delay before click in milliseconds / クリック前の遅延（ミリ秒）
    pub fn click_at(x: i32, y: i32, delay_ms: u64) -> Result<()> {
        Self::move_to(x, y)?;
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
        Self::click()
    }

    /// Move to position smoothly and click
    /// 位置にスムーズに移動してクリック
    ///
    /// # Arguments / 引数
    /// * `x` - X coordinate / X座標
    /// * `y` - Y coordinate / Y座標
    /// * `move_duration_ms` - Movement duration in milliseconds / 移動時間（ミリ秒）
    /// * `click_delay_ms` - Delay before click in milliseconds / クリック前の遅延（ミリ秒）
    pub fn click_at_smooth(
        x: i32,
        y: i32,
        move_duration_ms: u64,
        click_delay_ms: u64,
    ) -> Result<()> {
        Self::move_to_smooth(x, y, move_duration_ms)?;
        if click_delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(click_delay_ms));
        }
        Self::click()
    }

    /// Drag from current position to target position
    /// 現在位置からターゲット位置へドラッグ
    ///
    /// # Arguments / 引数
    /// * `to_x` - Target X coordinate / ターゲットX座標
    /// * `to_y` - Target Y coordinate / ターゲットY座標
    pub fn drag_to(to_x: i32, to_y: i32) -> Result<()> {
        Self::mouse_down()?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::move_to(to_x, to_y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::mouse_up()
    }

    /// Drag from start position to end position
    /// 開始位置から終了位置へドラッグ
    ///
    /// # Arguments / 引数
    /// * `from_x` - Start X coordinate / 開始X座標
    /// * `from_y` - Start Y coordinate / 開始Y座標
    /// * `to_x` - End X coordinate / 終了X座標
    /// * `to_y` - End Y coordinate / 終了Y座標
    pub fn drag(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()> {
        Self::move_to(from_x, from_y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::drag_to(to_x, to_y)
    }

    /// Smooth drag with interpolation steps
    /// 補間ステップによるスムーズドラッグ
    ///
    /// # Arguments / 引数
    /// * `from_x` - Start X coordinate / 開始X座標
    /// * `from_y` - Start Y coordinate / 開始Y座標
    /// * `to_x` - End X coordinate / 終了X座標
    /// * `to_y` - End Y coordinate / 終了Y座標
    /// * `steps` - Number of interpolation steps / 補間ステップ数
    /// * `duration_ms` - Total duration in milliseconds / 合計時間（ミリ秒）
    pub fn drag_smooth(
        from_x: i32,
        from_y: i32,
        to_x: i32,
        to_y: i32,
        steps: u32,
        duration_ms: u64,
    ) -> Result<()> {
        Self::move_to(from_x, from_y)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        Self::mouse_down()?;

        let step_delay = duration_ms / (steps as u64);
        for i in 1..=steps {
            let t = i as f64 / steps as f64;
            let x = from_x + ((to_x - from_x) as f64 * t) as i32;
            let y = from_y + ((to_y - from_y) as f64 * t) as i32;
            Self::move_to(x, y)?;
            std::thread::sleep(std::time::Duration::from_millis(step_delay));
        }

        Self::mouse_up()
    }

    #[cfg(target_os = "windows")]
    fn move_to_impl(x: i32, y: i32) -> Result<()> {
        windows::mouse_move(x, y)
    }

    #[cfg(target_os = "windows")]
    fn click_impl() -> Result<()> {
        windows::mouse_click()
    }

    #[cfg(target_os = "windows")]
    fn right_click_impl() -> Result<()> {
        windows::mouse_right_click()
    }

    #[cfg(target_os = "windows")]
    fn position_impl() -> Result<(i32, i32)> {
        windows::mouse_position()
    }

    #[cfg(target_os = "windows")]
    fn mouse_down_impl() -> Result<()> {
        windows::mouse_down()
    }

    #[cfg(target_os = "windows")]
    fn mouse_up_impl() -> Result<()> {
        windows::mouse_up()
    }

    #[cfg(target_os = "windows")]
    fn middle_click_impl() -> Result<()> {
        windows::mouse_middle_click()
    }

    #[cfg(target_os = "windows")]
    fn scroll_impl(clicks: i32) -> Result<()> {
        windows::mouse_scroll(clicks)
    }

    #[cfg(target_os = "windows")]
    fn scroll_horizontal_impl(clicks: i32) -> Result<()> {
        windows::mouse_scroll_horizontal(clicks)
    }

    #[cfg(target_os = "macos")]
    fn move_to_impl(x: i32, y: i32) -> Result<()> {
        macos::mouse_move(x, y)
    }

    #[cfg(target_os = "macos")]
    fn click_impl() -> Result<()> {
        macos::mouse_click()
    }

    #[cfg(target_os = "macos")]
    fn right_click_impl() -> Result<()> {
        macos::mouse_right_click()
    }

    #[cfg(target_os = "macos")]
    fn position_impl() -> Result<(i32, i32)> {
        macos::mouse_position()
    }

    #[cfg(target_os = "macos")]
    fn mouse_down_impl() -> Result<()> {
        macos::mouse_down()
    }

    #[cfg(target_os = "macos")]
    fn mouse_up_impl() -> Result<()> {
        macos::mouse_up()
    }

    #[cfg(target_os = "macos")]
    fn middle_click_impl() -> Result<()> {
        macos::mouse_middle_click()
    }

    #[cfg(target_os = "macos")]
    fn scroll_impl(clicks: i32) -> Result<()> {
        macos::mouse_scroll(clicks)
    }

    #[cfg(target_os = "macos")]
    fn scroll_horizontal_impl(clicks: i32) -> Result<()> {
        macos::mouse_scroll_horizontal(clicks)
    }

    #[cfg(target_os = "linux")]
    fn move_to_impl(x: i32, y: i32) -> Result<()> {
        linux::mouse_move(x, y)
    }

    #[cfg(target_os = "linux")]
    fn click_impl() -> Result<()> {
        linux::mouse_click()
    }

    #[cfg(target_os = "linux")]
    fn right_click_impl() -> Result<()> {
        linux::mouse_right_click()
    }

    #[cfg(target_os = "linux")]
    fn position_impl() -> Result<(i32, i32)> {
        linux::mouse_position()
    }

    #[cfg(target_os = "linux")]
    fn mouse_down_impl() -> Result<()> {
        linux::mouse_down()
    }

    #[cfg(target_os = "linux")]
    fn mouse_up_impl() -> Result<()> {
        linux::mouse_up()
    }

    #[cfg(target_os = "linux")]
    fn middle_click_impl() -> Result<()> {
        linux::mouse_middle_click()
    }

    #[cfg(target_os = "linux")]
    fn scroll_impl(clicks: i32) -> Result<()> {
        linux::mouse_scroll(clicks)
    }

    #[cfg(target_os = "linux")]
    fn scroll_horizontal_impl(clicks: i32) -> Result<()> {
        linux::mouse_scroll_horizontal(clicks)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn move_to_impl(_x: i32, _y: i32) -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn click_impl() -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn right_click_impl() -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn position_impl() -> Result<(i32, i32)> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn mouse_down_impl() -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn mouse_up_impl() -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn middle_click_impl() -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn scroll_impl(_clicks: i32) -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn scroll_horizontal_impl(_clicks: i32) -> Result<()> {
        Err(SikulixError::MouseError("Unsupported platform".to_string()))
    }
}

/// Keyboard control
/// キーボード制御
///
/// Provides methods for typing text and pressing keys.
/// Supports individual key press/release and hotkey combinations.
/// テキスト入力とキー押下のメソッドを提供します。
/// 個別のキー押下/解放とホットキーの組み合わせをサポートします。
///
/// # Example / 使用例
///
/// ```no_run
/// use sikulid::screen::{Keyboard, Key};
///
/// // Type text / テキスト入力
/// Keyboard::type_text("Hello, World!").unwrap();
///
/// // Press hotkey combination (Ctrl+S) / ホットキー（Ctrl+S）
/// Keyboard::hotkey(&[Key::Ctrl, Key::S]).unwrap();
///
/// // Individual key press/release / 個別キー押下/解放
/// Keyboard::press(Key::Shift).unwrap();
/// Keyboard::type_text("CAPS").unwrap();
/// Keyboard::release(Key::Shift).unwrap();
/// ```
pub struct Keyboard;

impl Keyboard {
    /// Type a string
    /// 文字列を入力
    pub fn type_text(text: &str) -> Result<()> {
        Self::type_text_impl(text)
    }

    /// Press a key
    /// キーを押下
    pub fn press(key: Key) -> Result<()> {
        Self::press_impl(key)
    }

    /// Release a key
    /// キーを解放
    pub fn release(key: Key) -> Result<()> {
        Self::release_impl(key)
    }

    /// Press and release a key combination
    /// キーの組み合わせを押下して解放
    pub fn hotkey(keys: &[Key]) -> Result<()> {
        for key in keys {
            Self::press(*key)?;
        }
        for key in keys.iter().rev() {
            Self::release(*key)?;
        }
        Ok(())
    }

    /// Type text with delay between characters
    /// 文字間に遅延を入れてテキストを入力
    ///
    /// # Arguments / 引数
    /// * `text` - Text to type / 入力するテキスト
    /// * `delay_ms` - Delay in milliseconds between each character / 文字間の遅延（ミリ秒）
    pub fn type_text_slow(text: &str, delay_ms: u64) -> Result<()> {
        Self::type_text_slow_impl(text, delay_ms)
    }

    /// Paste text via clipboard (supports Japanese and other Unicode text)
    /// クリップボード経由でテキストを貼り付け（日本語等のUnicode対応）
    ///
    /// This method sets the clipboard content and sends Ctrl+V (or Cmd+V on macOS).
    /// More reliable for non-ASCII characters than direct keyboard input.
    /// このメソッドはクリップボードにテキストを設定し、Ctrl+V（macOSではCmd+V）を送信します。
    /// 直接キーボード入力よりも非ASCII文字の入力に対して信頼性が高いです。
    pub fn paste_text(text: &str) -> Result<()> {
        Self::paste_text_impl(text)
    }

    /// Execute a sequence of keys with optional delay between each
    /// キーシーケンスを実行（各キー間にオプションのディレイ）
    ///
    /// Each key is tapped (pressed and released) in order.
    /// 各キーは順番にタップ（押下して解放）されます。
    ///
    /// # Arguments / 引数
    /// * `keys` - List of keys to tap in sequence / 順番にタップするキーのリスト
    /// * `delay_ms` - Delay between each key tap in milliseconds / 各キータップ間のディレイ（ミリ秒）
    pub fn key_sequence(keys: &[Key], delay_ms: u64) -> Result<()> {
        for (i, key) in keys.iter().enumerate() {
            Self::tap(*key)?;
            if i < keys.len() - 1 && delay_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
            }
        }
        Ok(())
    }

    /// Tap a key (press and immediately release)
    /// キーをタップ（押下してすぐに解放）
    pub fn tap(key: Key) -> Result<()> {
        Self::press(key)?;
        std::thread::sleep(std::time::Duration::from_millis(10));
        Self::release(key)
    }

    /// Type text with escape sequence support
    /// エスケープシーケンスをサポートしてテキストを入力
    ///
    /// Supports the following escape sequences:
    /// - `\n` - Press Enter key
    /// - `\t` - Press Tab key
    /// - `\b` - Press Backspace key
    /// - `\\` - Type backslash
    /// - `{KEY}` - Press special key (e.g., {ENTER}, {TAB}, {ESCAPE}, {F1}-{F12})
    ///
    /// # Example / 使用例
    /// ```ignore
    /// Keyboard::type_escaped("Hello{ENTER}World\tTabbed{F5}")?;
    /// ```
    pub fn type_escaped(text: &str) -> Result<()> {
        let mut chars = text.chars().peekable();
        let mut buffer = String::new();

        while let Some(c) = chars.next() {
            match c {
                '\\' => {
                    // Handle escape sequences
                    if let Some(&next) = chars.peek() {
                        match next {
                            'n' => {
                                chars.next();
                                if !buffer.is_empty() {
                                    Self::type_text(&buffer)?;
                                    buffer.clear();
                                }
                                Self::tap(Key::Enter)?;
                            }
                            't' => {
                                chars.next();
                                if !buffer.is_empty() {
                                    Self::type_text(&buffer)?;
                                    buffer.clear();
                                }
                                Self::tap(Key::Tab)?;
                            }
                            'b' => {
                                chars.next();
                                if !buffer.is_empty() {
                                    Self::type_text(&buffer)?;
                                    buffer.clear();
                                }
                                Self::tap(Key::Backspace)?;
                            }
                            '\\' => {
                                chars.next();
                                buffer.push('\\');
                            }
                            _ => buffer.push(c),
                        }
                    } else {
                        buffer.push(c);
                    }
                }
                '{' => {
                    // Handle special key notation {KEY}
                    if !buffer.is_empty() {
                        Self::type_text(&buffer)?;
                        buffer.clear();
                    }

                    let mut key_name = String::new();
                    for ch in chars.by_ref() {
                        if ch == '}' {
                            break;
                        }
                        key_name.push(ch);
                    }

                    if let Some(key) = Self::parse_key_name(&key_name) {
                        Self::tap(key)?;
                    } else {
                        // If not recognized, just type it literally
                        buffer.push('{');
                        buffer.push_str(&key_name);
                        buffer.push('}');
                    }
                }
                _ => buffer.push(c),
            }
        }

        if !buffer.is_empty() {
            Self::type_text(&buffer)?;
        }

        Ok(())
    }

    /// Parse a key name string to a Key enum
    /// キー名文字列をKey enumにパース
    pub fn parse_key_name(name: &str) -> Option<Key> {
        let name_upper = name.to_uppercase();
        match name_upper.as_str() {
            "ENTER" | "RETURN" => Some(Key::Enter),
            "TAB" => Some(Key::Tab),
            "ESCAPE" | "ESC" => Some(Key::Escape),
            "SPACE" => Some(Key::Space),
            "BACKSPACE" | "BS" => Some(Key::Backspace),
            "DELETE" | "DEL" => Some(Key::Delete),
            "HOME" => Some(Key::Home),
            "END" => Some(Key::End),
            "PAGEUP" | "PGUP" => Some(Key::PageUp),
            "PAGEDOWN" | "PGDN" => Some(Key::PageDown),
            "UP" => Some(Key::Up),
            "DOWN" => Some(Key::Down),
            "LEFT" => Some(Key::Left),
            "RIGHT" => Some(Key::Right),
            "F1" => Some(Key::F1),
            "F2" => Some(Key::F2),
            "F3" => Some(Key::F3),
            "F4" => Some(Key::F4),
            "F5" => Some(Key::F5),
            "F6" => Some(Key::F6),
            "F7" => Some(Key::F7),
            "F8" => Some(Key::F8),
            "F9" => Some(Key::F9),
            "F10" => Some(Key::F10),
            "F11" => Some(Key::F11),
            "F12" => Some(Key::F12),
            "CTRL" | "CONTROL" => Some(Key::Ctrl),
            "ALT" => Some(Key::Alt),
            "SHIFT" => Some(Key::Shift),
            "WIN" | "META" | "CMD" | "COMMAND" => Some(Key::Meta),
            _ => None,
        }
    }

    #[cfg(target_os = "windows")]
    fn type_text_impl(text: &str) -> Result<()> {
        windows::keyboard_type(text)
    }

    #[cfg(target_os = "windows")]
    fn press_impl(key: Key) -> Result<()> {
        windows::keyboard_press(key)
    }

    #[cfg(target_os = "windows")]
    fn release_impl(key: Key) -> Result<()> {
        windows::keyboard_release(key)
    }

    #[cfg(target_os = "windows")]
    fn type_text_slow_impl(text: &str, delay_ms: u64) -> Result<()> {
        windows::keyboard_type_slow(text, delay_ms)
    }

    #[cfg(target_os = "windows")]
    fn paste_text_impl(text: &str) -> Result<()> {
        windows::clipboard_paste_text(text)
    }

    #[cfg(target_os = "macos")]
    fn type_text_impl(text: &str) -> Result<()> {
        macos::keyboard_type(text)
    }

    #[cfg(target_os = "macos")]
    fn press_impl(key: Key) -> Result<()> {
        macos::keyboard_press(key)
    }

    #[cfg(target_os = "macos")]
    fn release_impl(key: Key) -> Result<()> {
        macos::keyboard_release(key)
    }

    #[cfg(target_os = "macos")]
    fn type_text_slow_impl(text: &str, delay_ms: u64) -> Result<()> {
        macos::keyboard_type_slow(text, delay_ms)
    }

    #[cfg(target_os = "macos")]
    fn paste_text_impl(text: &str) -> Result<()> {
        macos::clipboard_paste_text(text)
    }

    #[cfg(target_os = "linux")]
    fn type_text_impl(text: &str) -> Result<()> {
        linux::keyboard_type(text)
    }

    #[cfg(target_os = "linux")]
    fn press_impl(key: Key) -> Result<()> {
        linux::keyboard_press(key)
    }

    #[cfg(target_os = "linux")]
    fn release_impl(key: Key) -> Result<()> {
        linux::keyboard_release(key)
    }

    #[cfg(target_os = "linux")]
    fn type_text_slow_impl(text: &str, delay_ms: u64) -> Result<()> {
        linux::keyboard_type_slow(text, delay_ms)
    }

    #[cfg(target_os = "linux")]
    fn paste_text_impl(text: &str) -> Result<()> {
        linux::clipboard_paste_text(text)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn type_text_impl(_text: &str) -> Result<()> {
        Err(SikulixError::KeyboardError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn press_impl(_key: Key) -> Result<()> {
        Err(SikulixError::KeyboardError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn release_impl(_key: Key) -> Result<()> {
        Err(SikulixError::KeyboardError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn type_text_slow_impl(_text: &str, _delay_ms: u64) -> Result<()> {
        Err(SikulixError::KeyboardError(
            "Unsupported platform".to_string(),
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn paste_text_impl(_text: &str) -> Result<()> {
        Err(SikulixError::KeyboardError(
            "Unsupported platform".to_string(),
        ))
    }
}

/// Key codes for keyboard input
/// キーボード入力用キーコード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Key {
    // Modifier keys / 修飾キー
    Shift,
    Ctrl,
    Alt,
    Meta, // Windows key / Command key / Windowsキー / Commandキー

    // Function keys / ファンクションキー
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Navigation keys / ナビゲーションキー
    Enter,
    Tab,
    Space,
    Backspace,
    Delete,
    Escape,
    Home,
    End,
    PageUp,
    PageDown,
    Up,
    Down,
    Left,
    Right,

    // Letter keys / 文字キー
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Number keys / 数字キー
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_creation() {
        let screen = Screen::primary();
        assert_eq!(screen.index, 0);
    }

    // ========================================================================
    // Mouse Tests / マウステスト
    // ========================================================================

    #[test]
    #[cfg(target_os = "windows")]
    fn test_mouse_position_returns_result() {
        // On Windows, this should succeed and return coordinates
        let result = Mouse::position();
        assert!(result.is_ok());
        let (x, y) = result.unwrap();
        // Position should be reasonable (multi-monitor setups can have negative coordinates)
        assert!((-10000..=100000).contains(&x));
        assert!((-10000..=100000).contains(&y));
    }

    // ========================================================================
    // Keyboard Tests / キーボードテスト
    // ========================================================================

    #[test]
    fn test_key_enum_basic() {
        // Test that basic key variants exist
        let _a = Key::A;
        let _enter = Key::Enter;
        let _ctrl = Key::Ctrl;
        let _shift = Key::Shift;
        let _alt = Key::Alt;
        let _space = Key::Space;
        let _tab = Key::Tab;
        let _escape = Key::Escape;
    }

    #[test]
    fn test_key_enum_function_keys() {
        // Test function keys
        let _f1 = Key::F1;
        let _f2 = Key::F2;
        let _f3 = Key::F3;
        let _f12 = Key::F12;
    }

    #[test]
    fn test_key_enum_arrow_keys() {
        // Test arrow keys
        let _up = Key::Up;
        let _down = Key::Down;
        let _left = Key::Left;
        let _right = Key::Right;
    }

    #[test]
    fn test_key_enum_number_keys() {
        // Test number keys
        let _num0 = Key::Num0;
        let _num1 = Key::Num1;
        let _num9 = Key::Num9;
    }

    #[test]
    fn test_parse_key_name_basic() {
        // Test basic key name parsing
        assert_eq!(Keyboard::parse_key_name("ENTER"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("enter"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("Return"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("TAB"), Some(Key::Tab));
        assert_eq!(Keyboard::parse_key_name("SPACE"), Some(Key::Space));
        assert_eq!(Keyboard::parse_key_name("ESC"), Some(Key::Escape));
        assert_eq!(Keyboard::parse_key_name("ESCAPE"), Some(Key::Escape));
    }

    #[test]
    fn test_parse_key_name_modifiers() {
        // Test modifier key parsing
        assert_eq!(Keyboard::parse_key_name("CTRL"), Some(Key::Ctrl));
        assert_eq!(Keyboard::parse_key_name("CONTROL"), Some(Key::Ctrl));
        assert_eq!(Keyboard::parse_key_name("SHIFT"), Some(Key::Shift));
        assert_eq!(Keyboard::parse_key_name("ALT"), Some(Key::Alt));
        assert_eq!(Keyboard::parse_key_name("META"), Some(Key::Meta));
        assert_eq!(Keyboard::parse_key_name("WIN"), Some(Key::Meta));
        assert_eq!(Keyboard::parse_key_name("CMD"), Some(Key::Meta));
        assert_eq!(Keyboard::parse_key_name("COMMAND"), Some(Key::Meta));
    }

    #[test]
    fn test_parse_key_name_function_keys() {
        // Test function key parsing
        assert_eq!(Keyboard::parse_key_name("F1"), Some(Key::F1));
        assert_eq!(Keyboard::parse_key_name("f1"), Some(Key::F1));
        assert_eq!(Keyboard::parse_key_name("F12"), Some(Key::F12));
    }

    #[test]
    fn test_parse_key_name_arrows() {
        // Test arrow key parsing
        assert_eq!(Keyboard::parse_key_name("UP"), Some(Key::Up));
        assert_eq!(Keyboard::parse_key_name("DOWN"), Some(Key::Down));
        assert_eq!(Keyboard::parse_key_name("LEFT"), Some(Key::Left));
        assert_eq!(Keyboard::parse_key_name("RIGHT"), Some(Key::Right));
    }

    #[test]
    fn test_parse_key_name_editing_keys() {
        // Test editing key parsing
        assert_eq!(Keyboard::parse_key_name("BACKSPACE"), Some(Key::Backspace));
        assert_eq!(Keyboard::parse_key_name("DELETE"), Some(Key::Delete));
        assert_eq!(Keyboard::parse_key_name("DEL"), Some(Key::Delete));
        assert_eq!(Keyboard::parse_key_name("HOME"), Some(Key::Home));
        assert_eq!(Keyboard::parse_key_name("END"), Some(Key::End));
        assert_eq!(Keyboard::parse_key_name("PAGEUP"), Some(Key::PageUp));
        assert_eq!(Keyboard::parse_key_name("PAGEDOWN"), Some(Key::PageDown));
    }

    #[test]
    fn test_parse_key_name_invalid() {
        // Test invalid key names
        assert_eq!(Keyboard::parse_key_name("INVALID_KEY"), None);
        assert_eq!(Keyboard::parse_key_name(""), None);
        assert_eq!(Keyboard::parse_key_name("F99"), None);
    }

    #[test]
    fn test_parse_key_name_case_insensitive() {
        // Test case insensitivity
        assert_eq!(Keyboard::parse_key_name("Enter"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("ENTER"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("enter"), Some(Key::Enter));
        assert_eq!(Keyboard::parse_key_name("EnTeR"), Some(Key::Enter));
    }

    // ========================================================================
    // Key Clone/Copy Tests / キーのClone/Copyテスト
    // ========================================================================

    #[test]
    fn test_key_enum_copy() {
        // Test that Key implements Copy
        let key1 = Key::A;
        let key2 = key1; // Copy
        let key3 = key1; // Copy again
        assert_eq!(key2, key3);
    }

    #[test]
    fn test_key_debug() {
        // Test that Key implements Debug
        let key = Key::Enter;
        let debug_str = format!("{:?}", key);
        assert!(debug_str.contains("Enter"));
    }

    // ========================================================================
    // Letter Keys Tests / 文字キーテスト
    // ========================================================================

    #[test]
    fn test_all_letter_keys_exist() {
        // Verify all letter keys A-Z exist
        let letters = [
            Key::A,
            Key::B,
            Key::C,
            Key::D,
            Key::E,
            Key::F,
            Key::G,
            Key::H,
            Key::I,
            Key::J,
            Key::K,
            Key::L,
            Key::M,
            Key::N,
            Key::O,
            Key::P,
            Key::Q,
            Key::R,
            Key::S,
            Key::T,
            Key::U,
            Key::V,
            Key::W,
            Key::X,
            Key::Y,
            Key::Z,
        ];
        assert_eq!(letters.len(), 26);
    }

    #[test]
    fn test_all_number_keys_exist() {
        // Verify all number keys 0-9 exist
        let numbers = [
            Key::Num0,
            Key::Num1,
            Key::Num2,
            Key::Num3,
            Key::Num4,
            Key::Num5,
            Key::Num6,
            Key::Num7,
            Key::Num8,
            Key::Num9,
        ];
        assert_eq!(numbers.len(), 10);
    }

    #[test]
    fn test_all_function_keys_exist() {
        // Verify all function keys F1-F12 exist
        let function_keys = [
            Key::F1,
            Key::F2,
            Key::F3,
            Key::F4,
            Key::F5,
            Key::F6,
            Key::F7,
            Key::F8,
            Key::F9,
            Key::F10,
            Key::F11,
            Key::F12,
        ];
        assert_eq!(function_keys.len(), 12);
    }

    // ========================================================================
    // Screen Tests / スクリーンテスト
    // ========================================================================

    #[test]
    fn test_screen_default() {
        // Test Screen::default() creates primary screen
        let screen = Screen::default();
        assert_eq!(screen.index, 0);
    }

    #[test]
    fn test_screen_with_index() {
        // Test Screen::new() with custom index
        let screen = Screen::new(1);
        assert_eq!(screen.index, 1);
    }

    // ========================================================================
    // Integration Tests (require system interaction, run with --ignored)
    // 統合テスト（システム操作が必要、--ignoredで実行）
    // ========================================================================

    #[test]
    #[ignore = "Requires actual mouse movement - run with: cargo test -- --ignored"]
    fn integration_test_mouse_movement() {
        // Test mouse movement to specific coordinates
        // マウス移動のテスト

        // Move to center of screen (assuming at least 800x600 resolution)
        let result = Mouse::move_to(400, 300);
        assert!(result.is_ok(), "Mouse movement should succeed");

        // Verify position (may have slight offset due to OS behavior)
        let pos = Mouse::position();
        assert!(pos.is_ok(), "Position query should succeed");

        if let Ok((x, y)) = pos {
            assert!(
                (x - 400).abs() < 5 && (y - 300).abs() < 5,
                "Mouse should be near target position"
            );
        }
    }

    #[test]
    #[ignore = "Requires actual mouse click - run with: cargo test -- --ignored"]
    fn integration_test_mouse_click() {
        // Test mouse click at current position
        // マウスクリックのテスト

        // Move to safe position first
        let _ = Mouse::move_to(100, 100);

        // Left click
        let result = Mouse::click();
        assert!(result.is_ok(), "Left click should succeed");

        // Right click
        let result = Mouse::right_click();
        assert!(result.is_ok(), "Right click should succeed");

        // Double click
        let result = Mouse::double_click();
        assert!(result.is_ok(), "Double click should succeed");
    }

    #[test]
    #[ignore = "Requires actual keyboard input - run with: cargo test -- --ignored"]
    fn integration_test_keyboard_input() {
        // Test keyboard input simulation
        // キーボード入力のテスト

        // Type a simple string
        let result = Keyboard::type_text("test");
        assert!(result.is_ok(), "Typing text should succeed");

        // Press single key
        let result = Keyboard::press(Key::Escape);
        assert!(result.is_ok(), "Pressing Escape should succeed");

        // Release the key
        let result = Keyboard::release(Key::Escape);
        assert!(result.is_ok(), "Releasing Escape should succeed");

        // Key combination (Ctrl+A)
        let result = Keyboard::hotkey(&[Key::Ctrl, Key::A]);
        assert!(result.is_ok(), "Key combination should succeed");
    }

    #[test]
    #[ignore = "Requires screen capture - run with: cargo test -- --ignored"]
    fn integration_test_screen_capture() {
        // Test screen capture functionality
        // スクリーンキャプチャのテスト
        let screen = Screen::default();

        // Capture full screen
        let result = screen.capture();
        assert!(result.is_ok(), "Screen capture should succeed");

        if let Ok(image) = result {
            // Check image dimensions are reasonable
            assert!(image.width() > 0, "Image width should be positive");
            assert!(image.height() > 0, "Image height should be positive");
            assert!(image.width() >= 640, "Screen should be at least 640 wide");
            assert!(image.height() >= 480, "Screen should be at least 480 tall");
        }
    }

    #[test]
    #[ignore = "Requires screen capture and image matching - run with: cargo test -- --ignored"]
    fn integration_test_image_find() {
        // Test image finding on screen (basic check)
        // 画像検索のテスト（基本チェック）
        use crate::image::ImageMatcher;

        let screen = Screen::default();
        let _matcher = ImageMatcher::new();

        // Capture screen
        let screen_capture = screen.capture();
        assert!(screen_capture.is_ok(), "Screen capture should succeed");

        // ImageMatcher was created successfully if we reach here
    }

    #[test]
    #[ignore = "Requires drag operation - run with: cargo test -- --ignored"]
    fn integration_test_mouse_drag() {
        // Test mouse drag operation
        // ドラッグ操作のテスト

        // Use the drag function directly
        let result = Mouse::drag(200, 200, 400, 400);
        assert!(result.is_ok(), "Drag operation should succeed");

        // Verify end position
        let pos = Mouse::position();
        if let Ok((x, y)) = pos {
            assert!(
                (x - 400).abs() < 10 && (y - 400).abs() < 10,
                "Mouse should be near drag end position"
            );
        }
    }

    #[test]
    #[ignore = "Requires modifier key operations - run with: cargo test -- --ignored"]
    fn integration_test_modifier_keys() {
        // Test modifier key combinations
        // 修飾キーの組み合わせテスト

        // Test Shift+A (should type 'A')
        let result = Keyboard::hotkey(&[Key::Shift, Key::A]);
        assert!(result.is_ok(), "Shift+A should succeed");

        // Test Ctrl+Shift+Escape (common system shortcut)
        let result = Keyboard::hotkey(&[Key::Ctrl, Key::Shift, Key::Escape]);
        assert!(result.is_ok(), "Ctrl+Shift+Escape should succeed");

        // Test Alt+Tab (window switching)
        let result = Keyboard::hotkey(&[Key::Alt, Key::Tab]);
        assert!(result.is_ok(), "Alt+Tab should succeed");
    }
}
