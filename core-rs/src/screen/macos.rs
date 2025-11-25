//! macOS-specific screen capture and input implementation

use crate::screen::Key;
use crate::{Region, Result, SikulixError};
use image::DynamicImage;

/// Get screen dimensions for the given monitor index
pub fn get_screen_dimensions(_index: u32) -> Result<(u32, u32)> {
    // TODO: Implement using core-graphics
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

/// Capture the entire screen
pub fn capture_screen(_index: u32) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

/// Capture a specific region of the screen
pub fn capture_region(_index: u32, _region: &Region) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

/// Move mouse to position
pub fn mouse_move(_x: i32, _y: i32) -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

/// Click mouse button
pub fn mouse_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

/// Right-click mouse button
pub fn mouse_right_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

/// Get current mouse position
pub fn mouse_position() -> Result<(i32, i32)> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

/// Type text using keyboard
pub fn keyboard_type(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

/// Press a key
pub fn keyboard_press(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

/// Release a key
pub fn keyboard_release(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}
