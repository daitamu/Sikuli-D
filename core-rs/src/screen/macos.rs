//! macOS-specific screen capture and input implementation
//!
//! Uses core-graphics and core-foundation for screen capture and input.

use crate::screen::Key;
use crate::{Region, Result, SikulixError};
use image::{DynamicImage, RgbaImage};

#[cfg(target_os = "macos")]
use core_graphics::display::{
    kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGDisplay, CGDisplayBounds, CGMainDisplayID,
};
#[cfg(target_os = "macos")]
use core_graphics::event::{
    CGEvent, CGEventFlags, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton,
};
#[cfg(target_os = "macos")]
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
#[cfg(target_os = "macos")]
use core_graphics::geometry::CGPoint;
#[cfg(target_os = "macos")]
use core_graphics::image::CGImage;

/// Get screen dimensions for the given monitor index
#[cfg(target_os = "macos")]
pub fn get_screen_dimensions(index: u32) -> Result<(u32, u32)> {
    if index == 0 {
        let display_id = unsafe { CGMainDisplayID() };
        let bounds = unsafe { CGDisplayBounds(display_id) };
        Ok((bounds.size.width as u32, bounds.size.height as u32))
    } else {
        // Get all displays
        let displays = CGDisplay::active_displays().map_err(|e| {
            SikulixError::ScreenCaptureError(format!("Failed to get displays: {:?}", e))
        })?;

        if (index as usize) < displays.len() {
            let display_id = displays[index as usize];
            let bounds = unsafe { CGDisplayBounds(display_id) };
            Ok((bounds.size.width as u32, bounds.size.height as u32))
        } else {
            Err(SikulixError::ScreenCaptureError(format!(
                "Monitor {} not found",
                index
            )))
        }
    }
}

/// Get the number of connected screens/monitors
/// 接続されている画面/モニターの数を取得
#[cfg(target_os = "macos")]
pub fn get_number_screens() -> u32 {
    match CGDisplay::active_displays() {
        Ok(displays) => displays.len() as u32,
        Err(_) => 1, // Fallback to 1 on error
    }
}

/// Capture the entire screen
#[cfg(target_os = "macos")]
pub fn capture_screen(index: u32) -> Result<DynamicImage> {
    let display_id = if index == 0 {
        unsafe { CGMainDisplayID() }
    } else {
        let displays = CGDisplay::active_displays().map_err(|e| {
            SikulixError::ScreenCaptureError(format!("Failed to get displays: {:?}", e))
        })?;

        if (index as usize) < displays.len() {
            displays[index as usize]
        } else {
            return Err(SikulixError::ScreenCaptureError(format!(
                "Monitor {} not found",
                index
            )));
        }
    };

    let display = CGDisplay::new(display_id);
    let cg_image = display
        .image()
        .ok_or_else(|| SikulixError::ScreenCaptureError("Failed to capture screen".to_string()))?;

    cg_image_to_dynamic_image(&cg_image)
}

/// Capture a specific region of the screen (using global coordinates)
/// 画面の特定領域をキャプチャ（グローバル座標使用）
///
/// Region coordinates are in global logical pixels across all monitors.
/// Regionの座標は全モニターにわたるグローバル論理ピクセルです。
#[cfg(target_os = "macos")]
pub fn capture_region(region: &Region) -> Result<DynamicImage> {
    use core_graphics::geometry::{CGRect, CGSize};

    let rect = CGRect::new(
        &CGPoint::new(region.x as f64, region.y as f64),
        &CGSize::new(region.width as f64, region.height as f64),
    );

    let cg_image = CGDisplay::screenshot(rect, kCGWindowListOptionOnScreenOnly, kCGNullWindowID, 0)
        .ok_or_else(|| SikulixError::ScreenCaptureError("Failed to capture region".to_string()))?;

    cg_image_to_dynamic_image(&cg_image)
}

/// Convert CGImage to DynamicImage using CGBitmapContext
#[cfg(target_os = "macos")]
fn cg_image_to_dynamic_image(cg_image: &CGImage) -> Result<DynamicImage> {
    use core_graphics::base::kCGImageAlphaPremultipliedLast;
    use core_graphics::color_space::CGColorSpace;
    use core_graphics::context::CGContext;
    use core_graphics::geometry::{CGRect, CGSize};
    use std::ffi::c_void;

    let width = cg_image.width();
    let height = cg_image.height();
    let bytes_per_row = width * 4;

    // Create buffer for RGBA data
    let mut buffer = vec![0u8; height * bytes_per_row];

    // Create color space and bitmap context
    let color_space = CGColorSpace::create_device_rgb();
    let context = CGContext::create_bitmap_context(
        Some(buffer.as_mut_ptr() as *mut c_void),
        width,
        height,
        8, // bits per component
        bytes_per_row,
        &color_space,
        kCGImageAlphaPremultipliedLast,
    );

    // Draw the CGImage into the context
    let rect = CGRect::new(
        &CGPoint::new(0.0, 0.0),
        &CGSize::new(width as f64, height as f64),
    );
    context.draw_image(rect, cg_image);

    // The buffer now contains RGBA pixel data
    let img = RgbaImage::from_raw(width as u32, height as u32, buffer)
        .ok_or_else(|| SikulixError::ScreenCaptureError("Failed to create image".to_string()))?;

    Ok(DynamicImage::ImageRgba8(img))
}

/// Move mouse to position
#[cfg(target_os = "macos")]
pub fn mouse_move(x: i32, y: i32) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    let point = CGPoint::new(x as f64, y as f64);
    let event =
        CGEvent::new_mouse_event(source, CGEventType::MouseMoved, point, CGMouseButton::Left)
            .map_err(|_| SikulixError::MouseError("Failed to create mouse event".to_string()))?;

    event.post(CGEventTapLocation::HID);
    Ok(())
}

/// Click mouse button
#[cfg(target_os = "macos")]
pub fn mouse_click() -> Result<()> {
    let (x, y) = mouse_position()?;
    let point = CGPoint::new(x as f64, y as f64);

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    // Mouse down
    let down_event = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::LeftMouseDown,
        point,
        CGMouseButton::Left,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse down event".to_string()))?;
    down_event.post(CGEventTapLocation::HID);

    // Mouse up
    let up_event =
        CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, point, CGMouseButton::Left)
            .map_err(|_| SikulixError::MouseError("Failed to create mouse up event".to_string()))?;
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Right-click mouse button
#[cfg(target_os = "macos")]
pub fn mouse_right_click() -> Result<()> {
    let (x, y) = mouse_position()?;
    let point = CGPoint::new(x as f64, y as f64);

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    // Mouse down
    let down_event = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::RightMouseDown,
        point,
        CGMouseButton::Right,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse down event".to_string()))?;
    down_event.post(CGEventTapLocation::HID);

    // Mouse up
    let up_event = CGEvent::new_mouse_event(
        source,
        CGEventType::RightMouseUp,
        point,
        CGMouseButton::Right,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse up event".to_string()))?;
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Get current mouse position
#[cfg(target_os = "macos")]
pub fn mouse_position() -> Result<(i32, i32)> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    let event = CGEvent::new(source)
        .map_err(|_| SikulixError::MouseError("Failed to create event".to_string()))?;

    let location = event.location();
    Ok((location.x as i32, location.y as i32))
}

/// Press mouse button down (without releasing)
#[cfg(target_os = "macos")]
pub fn mouse_down() -> Result<()> {
    let (x, y) = mouse_position()?;
    let point = CGPoint::new(x as f64, y as f64);

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    let event = CGEvent::new_mouse_event(
        source,
        CGEventType::LeftMouseDown,
        point,
        CGMouseButton::Left,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse down event".to_string()))?;
    event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Release mouse button
#[cfg(target_os = "macos")]
pub fn mouse_up() -> Result<()> {
    let (x, y) = mouse_position()?;
    let point = CGPoint::new(x as f64, y as f64);

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    let event =
        CGEvent::new_mouse_event(source, CGEventType::LeftMouseUp, point, CGMouseButton::Left)
            .map_err(|_| SikulixError::MouseError("Failed to create mouse up event".to_string()))?;
    event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Middle-click mouse button
#[cfg(target_os = "macos")]
pub fn mouse_middle_click() -> Result<()> {
    let (x, y) = mouse_position()?;
    let point = CGPoint::new(x as f64, y as f64);

    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    // Mouse down (center button)
    let down_event = CGEvent::new_mouse_event(
        source.clone(),
        CGEventType::OtherMouseDown,
        point,
        CGMouseButton::Center,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse down event".to_string()))?;
    down_event.post(CGEventTapLocation::HID);

    // Mouse up
    let up_event = CGEvent::new_mouse_event(
        source,
        CGEventType::OtherMouseUp,
        point,
        CGMouseButton::Center,
    )
    .map_err(|_| SikulixError::MouseError("Failed to create mouse up event".to_string()))?;
    up_event.post(CGEventTapLocation::HID);

    Ok(())
}

/// Scroll mouse wheel vertically
/// マウスホイールを垂直スクロール
///
/// # Arguments
/// 引数
///
/// * `clicks` - Number of wheel clicks (positive = up, negative = down)
///   ホイールクリック数（正 = 上、負 = 下）
#[cfg(target_os = "macos")]
pub fn mouse_scroll(clicks: i32) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    // Create scroll wheel event
    // On macOS, positive values scroll up (wheel away from user)
    let event = CGEvent::new_scroll_event(
        source,
        core_graphics::event::ScrollEventUnit::Line,
        1,        // wheel_count
        clicks,   // delta1 (vertical)
        0,        // delta2 (horizontal)
        0,        // delta3
    )
    .map_err(|_| SikulixError::MouseError("Failed to create scroll event".to_string()))?;

    event.post(CGEventTapLocation::HID);
    Ok(())
}

/// Scroll mouse wheel horizontally
/// マウスホイールを水平スクロール
///
/// # Arguments
/// 引数
///
/// * `clicks` - Number of wheel clicks (positive = right, negative = left)
///   ホイールクリック数（正 = 右、負 = 左）
#[cfg(target_os = "macos")]
pub fn mouse_scroll_horizontal(clicks: i32) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::MouseError("Failed to create event source".to_string()))?;

    // Create scroll wheel event for horizontal scrolling
    let event = CGEvent::new_scroll_event(
        source,
        core_graphics::event::ScrollEventUnit::Line,
        2,        // wheel_count (2 for horizontal)
        0,        // delta1 (vertical)
        clicks,   // delta2 (horizontal)
        0,        // delta3
    )
    .map_err(|_| SikulixError::MouseError("Failed to create scroll event".to_string()))?;

    event.post(CGEventTapLocation::HID);
    Ok(())
}

/// Type text using keyboard
#[cfg(target_os = "macos")]
pub fn keyboard_type(text: &str) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::KeyboardError("Failed to create event source".to_string()))?;

    for ch in text.chars() {
        // Create key down event with Unicode
        let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| SikulixError::KeyboardError("Failed to create key event".to_string()))?;

        // Set Unicode string
        let chars = [ch as u16];
        event.set_string_from_utf16_unchecked(&chars);
        event.post(CGEventTapLocation::HID);

        // Key up
        let up_event = CGEvent::new_keyboard_event(source.clone(), 0, false).map_err(|_| {
            SikulixError::KeyboardError("Failed to create key up event".to_string())
        })?;
        up_event.post(CGEventTapLocation::HID);
    }

    Ok(())
}

/// Press a key
#[cfg(target_os = "macos")]
pub fn keyboard_press(key: Key) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::KeyboardError("Failed to create event source".to_string()))?;

    let keycode = key_to_keycode(key);
    let event = CGEvent::new_keyboard_event(source, keycode, true)
        .map_err(|_| SikulixError::KeyboardError("Failed to create key event".to_string()))?;

    // Add modifier flags if needed
    if let Some(flags) = key_to_flags(key) {
        event.set_flags(flags);
    }

    event.post(CGEventTapLocation::HID);
    Ok(())
}

/// Release a key
#[cfg(target_os = "macos")]
pub fn keyboard_release(key: Key) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::KeyboardError("Failed to create event source".to_string()))?;

    let keycode = key_to_keycode(key);
    let event = CGEvent::new_keyboard_event(source, keycode, false)
        .map_err(|_| SikulixError::KeyboardError("Failed to create key event".to_string()))?;

    event.post(CGEventTapLocation::HID);
    Ok(())
}

/// Convert Key enum to macOS keycode
#[cfg(target_os = "macos")]
fn key_to_keycode(key: Key) -> CGKeyCode {
    match key {
        Key::Shift => 0x38, // kVK_Shift
        Key::Ctrl => 0x3B,  // kVK_Control
        Key::Alt => 0x3A,   // kVK_Option
        Key::Meta => 0x37,  // kVK_Command
        Key::F1 => 0x7A,
        Key::F2 => 0x78,
        Key::F3 => 0x63,
        Key::F4 => 0x76,
        Key::F5 => 0x60,
        Key::F6 => 0x61,
        Key::F7 => 0x62,
        Key::F8 => 0x64,
        Key::F9 => 0x65,
        Key::F10 => 0x6D,
        Key::F11 => 0x67,
        Key::F12 => 0x6F,
        Key::Enter => 0x24,     // kVK_Return
        Key::Tab => 0x30,       // kVK_Tab
        Key::Space => 0x31,     // kVK_Space
        Key::Backspace => 0x33, // kVK_Delete
        Key::Delete => 0x75,    // kVK_ForwardDelete
        Key::Escape => 0x35,    // kVK_Escape
        Key::Home => 0x73,      // kVK_Home
        Key::End => 0x77,       // kVK_End
        Key::PageUp => 0x74,    // kVK_PageUp
        Key::PageDown => 0x79,  // kVK_PageDown
        Key::Up => 0x7E,        // kVK_UpArrow
        Key::Down => 0x7D,      // kVK_DownArrow
        Key::Left => 0x7B,      // kVK_LeftArrow
        Key::Right => 0x7C,     // kVK_RightArrow
        Key::A => 0x00,
        Key::B => 0x0B,
        Key::C => 0x08,
        Key::D => 0x02,
        Key::E => 0x0E,
        Key::F => 0x03,
        Key::G => 0x05,
        Key::H => 0x04,
        Key::I => 0x22,
        Key::J => 0x26,
        Key::K => 0x28,
        Key::L => 0x25,
        Key::M => 0x2E,
        Key::N => 0x2D,
        Key::O => 0x1F,
        Key::P => 0x23,
        Key::Q => 0x0C,
        Key::R => 0x0F,
        Key::S => 0x01,
        Key::T => 0x11,
        Key::U => 0x20,
        Key::V => 0x09,
        Key::W => 0x0D,
        Key::X => 0x07,
        Key::Y => 0x10,
        Key::Z => 0x06,
        Key::Num0 => 0x1D,
        Key::Num1 => 0x12,
        Key::Num2 => 0x13,
        Key::Num3 => 0x14,
        Key::Num4 => 0x15,
        Key::Num5 => 0x17,
        Key::Num6 => 0x16,
        Key::Num7 => 0x1A,
        Key::Num8 => 0x1C,
        Key::Num9 => 0x19,
    }
}

/// Get modifier flags for special keys
#[cfg(target_os = "macos")]
fn key_to_flags(key: Key) -> Option<CGEventFlags> {
    match key {
        Key::Shift => Some(CGEventFlags::CGEventFlagShift),
        Key::Ctrl => Some(CGEventFlags::CGEventFlagControl),
        Key::Alt => Some(CGEventFlags::CGEventFlagAlternate),
        Key::Meta => Some(CGEventFlags::CGEventFlagCommand),
        _ => None,
    }
}

/// Paste text via clipboard (supports Japanese and other Unicode text)
/// Uses NSPasteboard via objc runtime
#[cfg(target_os = "macos")]
pub fn clipboard_paste_text(text: &str) -> Result<()> {
    use std::process::Command;

    // Use pbcopy to set clipboard content (simplest cross-compatible approach)
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to spawn pbcopy: {}", e)))?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(text.as_bytes()).map_err(|e| {
            SikulixError::KeyboardError(format!("Failed to write to pbcopy: {}", e))
        })?;
    }

    child
        .wait()
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to wait for pbcopy: {}", e)))?;

    // Small delay to ensure clipboard is ready
    std::thread::sleep(std::time::Duration::from_millis(10));

    // Send Cmd+V to paste (macOS uses Command key for paste)
    keyboard_press(Key::Meta)?;
    keyboard_press(Key::V)?;
    std::thread::sleep(std::time::Duration::from_millis(50));
    keyboard_release(Key::V)?;
    keyboard_release(Key::Meta)?;

    Ok(())
}

/// Type text with delay between characters
#[cfg(target_os = "macos")]
pub fn keyboard_type_slow(text: &str, delay_ms: u64) -> Result<()> {
    let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|_| SikulixError::KeyboardError("Failed to create event source".to_string()))?;

    for ch in text.chars() {
        // Create key down event with Unicode
        let event = CGEvent::new_keyboard_event(source.clone(), 0, true)
            .map_err(|_| SikulixError::KeyboardError("Failed to create key event".to_string()))?;

        // Set Unicode string
        let chars = [ch as u16];
        event.set_string_from_utf16_unchecked(&chars);
        event.post(CGEventTapLocation::HID);

        // Key up
        let up_event = CGEvent::new_keyboard_event(source.clone(), 0, false).map_err(|_| {
            SikulixError::KeyboardError("Failed to create key up event".to_string())
        })?;
        up_event.post(CGEventTapLocation::HID);

        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }

    Ok(())
}

// Stub implementations for non-macOS builds
#[cfg(not(target_os = "macos"))]
pub fn get_screen_dimensions(_index: u32) -> Result<(u32, u32)> {
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn capture_screen(_index: u32) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn capture_region(_region: &Region) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_move(_x: i32, _y: i32) -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_right_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_position() -> Result<(i32, i32)> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn keyboard_type(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn keyboard_press(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn keyboard_release(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_down() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_up() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_middle_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_scroll(_clicks: i32) -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn mouse_scroll_horizontal(_clicks: i32) -> Result<()> {
    Err(SikulixError::MouseError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn clipboard_paste_text(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "macos"))]
pub fn keyboard_type_slow(_text: &str, _delay_ms: u64) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "macOS implementation pending".to_string(),
    ))
}
