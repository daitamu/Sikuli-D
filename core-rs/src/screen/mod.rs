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
//! use sikulix_core::screen::{Screen, Mouse, Keyboard, Key};
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

use crate::{Region, Result};

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
use crate::SikulixError;
use image::DynamicImage;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

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
/// use sikulix_core::Screen;
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

    /// Get the primary screen
    pub fn primary() -> Self {
        Self::new(0)
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
        windows::capture_region(self.index, region)
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
        macos::capture_region(self.index, region)
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
        linux::capture_region(self.index, region)
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
/// use sikulix_core::Mouse;
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
/// use sikulix_core::screen::{Keyboard, Key};
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
}
