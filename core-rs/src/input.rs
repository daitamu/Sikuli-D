//! Input convenience functions
//! 入力用便利関数
//!
//! This module provides simple functions for mouse and keyboard input.
//! このモジュールはマウスとキーボード入力のためのシンプルな関数を提供します。

use crate::screen::{Keyboard, Mouse};
use crate::Result;

/// Click at the specified position
/// 指定位置でクリック
pub fn click(x: i32, y: i32) -> Result<()> {
    Mouse::move_to(x, y)?;
    Mouse::click()
}

/// Double click at the specified position
/// 指定位置でダブルクリック
pub fn double_click(x: i32, y: i32) -> Result<()> {
    Mouse::move_to(x, y)?;
    Mouse::double_click()
}

/// Right click at the specified position
/// 指定位置で右クリック
pub fn right_click(x: i32, y: i32) -> Result<()> {
    Mouse::move_to(x, y)?;
    Mouse::right_click()
}

/// Middle click at the specified position
/// 指定位置で中クリック
pub fn middle_click(x: i32, y: i32) -> Result<()> {
    Mouse::move_to(x, y)?;
    Mouse::middle_click()
}

/// Type text
/// テキストを入力
pub fn type_text(text: &str) -> Result<()> {
    Keyboard::type_text(text)
}

/// Type text slowly with delay between characters
/// 文字間に遅延を入れてゆっくりテキストを入力
pub fn type_text_slow(text: &str, delay_ms: u64) -> Result<()> {
    Keyboard::type_text_slow(text, delay_ms)
}

/// Paste text via clipboard
/// クリップボード経由でテキストをペースト
pub fn paste(text: &str) -> Result<()> {
    Keyboard::paste_text(text)
}

/// Drag from one position to another
/// ある位置から別の位置へドラッグ
pub fn drag(from_x: i32, from_y: i32, to_x: i32, to_y: i32) -> Result<()> {
    Mouse::drag(from_x, from_y, to_x, to_y)
}

/// Move mouse to position
/// マウスを位置に移動
pub fn move_to(x: i32, y: i32) -> Result<()> {
    Mouse::move_to(x, y)
}

/// Get current mouse position
/// 現在のマウス位置を取得
pub fn mouse_position() -> Result<(i32, i32)> {
    Mouse::position()
}

#[cfg(test)]
mod tests {
    // Input tests would affect actual system, so they are in screen/mod.rs as integration tests
}
