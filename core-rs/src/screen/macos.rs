//! macOS-specific screen capture and input implementation
//!
//! Uses core-graphics and core-foundation for screen capture and input.

use crate::screen::Key;
use crate::{Region, Result, SikulixError};
use image::{DynamicImage, RgbaImage};

#[cfg(target_os = "macos")]
use core_foundation::base::TCFType;
#[cfg(target_os = "macos")]
use core_graphics::display::{
    kCGNullWindowID, kCGWindowListOptionOnScreenOnly, CGDisplay, CGDisplayBounds, CGMainDisplayID,
    CGWindowListCopyWindowInfo,
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

/// Capture a specific region of the screen
#[cfg(target_os = "macos")]
pub fn capture_region(index: u32, region: &Region) -> Result<DynamicImage> {
    use core_graphics::geometry::{CGRect, CGSize};

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

    let rect = CGRect::new(
        &CGPoint::new(region.x as f64, region.y as f64),
        &CGSize::new(region.width as f64, region.height as f64),
    );

    let cg_image = CGDisplay::screenshot(rect, kCGWindowListOptionOnScreenOnly, kCGNullWindowID, 0)
        .ok_or_else(|| SikulixError::ScreenCaptureError("Failed to capture region".to_string()))?;

    cg_image_to_dynamic_image(&cg_image)
}

/// Convert CGImage to DynamicImage
#[cfg(target_os = "macos")]
fn cg_image_to_dynamic_image(cg_image: &CGImage) -> Result<DynamicImage> {
    let width = cg_image.width();
    let height = cg_image.height();
    let bytes_per_row = cg_image.bytes_per_row();
    let bits_per_pixel = cg_image.bits_per_pixel();

    // Get raw pixel data
    let data_provider = cg_image
        .data_provider()
        .ok_or_else(|| SikulixError::ScreenCaptureError("No data provider".to_string()))?;
    let raw_data = data_provider.copy_data();
    let bytes: &[u8] = raw_data.bytes();

    // Handle different pixel formats
    let mut rgba_buffer = Vec::with_capacity(width * height * 4);

    match bits_per_pixel {
        32 => {
            // BGRA or RGBA format
            for y in 0..height {
                for x in 0..width {
                    let offset = y * bytes_per_row + x * 4;
                    if offset + 3 < bytes.len() {
                        // Assume BGRA, convert to RGBA
                        rgba_buffer.push(bytes[offset + 2]); // R
                        rgba_buffer.push(bytes[offset + 1]); // G
                        rgba_buffer.push(bytes[offset]); // B
                        rgba_buffer.push(bytes[offset + 3]); // A
                    }
                }
            }
        }
        24 => {
            // RGB format
            for y in 0..height {
                for x in 0..width {
                    let offset = y * bytes_per_row + x * 3;
                    if offset + 2 < bytes.len() {
                        rgba_buffer.push(bytes[offset]); // R
                        rgba_buffer.push(bytes[offset + 1]); // G
                        rgba_buffer.push(bytes[offset + 2]); // B
                        rgba_buffer.push(255); // A
                    }
                }
            }
        }
        _ => {
            return Err(SikulixError::ScreenCaptureError(format!(
                "Unsupported bits per pixel: {}",
                bits_per_pixel
            )));
        }
    }

    let img = RgbaImage::from_raw(width as u32, height as u32, rgba_buffer)
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
pub fn capture_region(_index: u32, _region: &Region) -> Result<DynamicImage> {
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
