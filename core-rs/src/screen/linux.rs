//! Linux-specific screen capture and input implementation
//!
//! Uses x11rb for X11 screen capture and XTest for input simulation.

use crate::screen::Key;
use crate::{Region, Result, SikulixError};
use image::{DynamicImage, RgbaImage};

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat, Screen as X11Screen, Window};
#[cfg(target_os = "linux")]
use x11rb::rust_connection::RustConnection;

#[cfg(target_os = "linux")]
use x11rb::protocol::xtest::ConnectionExt as XTestConnectionExt;

/// Get screen dimensions for the given monitor index
#[cfg(target_os = "linux")]
pub fn get_screen_dimensions(index: u32) -> Result<(u32, u32)> {
    let (conn, screen_num) = RustConnection::connect(None).map_err(|e| {
        SikulixError::ScreenCaptureError(format!("Failed to connect to X11: {}", e))
    })?;

    let setup = conn.setup();
    let screens = &setup.roots;

    if (index as usize) < screens.len() {
        let screen = &screens[index as usize];
        Ok((
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        ))
    } else if index == 0 && !screens.is_empty() {
        let screen = &screens[screen_num];
        Ok((
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        ))
    } else {
        Err(SikulixError::ScreenCaptureError(format!(
            "Monitor {} not found",
            index
        )))
    }
}

/// Capture the entire screen
#[cfg(target_os = "linux")]
pub fn capture_screen(index: u32) -> Result<DynamicImage> {
    let (width, height) = get_screen_dimensions(index)?;
    capture_region(index, &Region::new(0, 0, width, height))
}

/// Capture a specific region of the screen
#[cfg(target_os = "linux")]
pub fn capture_region(index: u32, region: &Region) -> Result<DynamicImage> {
    let (conn, screen_num) = RustConnection::connect(None).map_err(|e| {
        SikulixError::ScreenCaptureError(format!("Failed to connect to X11: {}", e))
    })?;

    let setup = conn.setup();
    let screens = &setup.roots;

    let screen_index = if (index as usize) < screens.len() {
        index as usize
    } else {
        screen_num
    };

    let screen = &screens[screen_index];
    let root = screen.root;

    // Get image from X server
    let reply = conn
        .get_image(
            ImageFormat::Z_PIXMAP,
            root,
            region.x as i16,
            region.y as i16,
            region.width as u16,
            region.height as u16,
            !0u32, // all planes
        )
        .map_err(|e| SikulixError::ScreenCaptureError(format!("Failed to get image: {}", e)))?
        .reply()
        .map_err(|e| {
            SikulixError::ScreenCaptureError(format!("Failed to get image reply: {}", e))
        })?;

    let depth = reply.depth;
    let data = reply.data;

    // Convert to RGBA
    let rgba_buffer = convert_x11_image_to_rgba(&data, region.width, region.height, depth)?;

    let img = RgbaImage::from_raw(region.width, region.height, rgba_buffer)
        .ok_or_else(|| SikulixError::ScreenCaptureError("Failed to create image".to_string()))?;

    Ok(DynamicImage::ImageRgba8(img))
}

/// Convert X11 image data to RGBA
#[cfg(target_os = "linux")]
fn convert_x11_image_to_rgba(data: &[u8], width: u32, height: u32, depth: u8) -> Result<Vec<u8>> {
    let mut rgba_buffer = Vec::with_capacity((width * height * 4) as usize);

    match depth {
        24 | 32 => {
            // Typically BGRA or BGRX format
            let bytes_per_pixel = if depth == 32 { 4 } else { 3 };
            let row_bytes = ((width as usize * bytes_per_pixel + 3) / 4) * 4; // X11 pads rows

            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset = y * row_bytes + x * bytes_per_pixel;
                    if offset + 2 < data.len() {
                        // BGRA -> RGBA
                        rgba_buffer.push(data[offset + 2]); // R
                        rgba_buffer.push(data[offset + 1]); // G
                        rgba_buffer.push(data[offset]); // B
                        if bytes_per_pixel == 4 && offset + 3 < data.len() {
                            rgba_buffer.push(data[offset + 3]); // A
                        } else {
                            rgba_buffer.push(255); // A
                        }
                    }
                }
            }
        }
        16 => {
            // RGB565 format
            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset = (y * width as usize + x) * 2;
                    if offset + 1 < data.len() {
                        let pixel = u16::from_le_bytes([data[offset], data[offset + 1]]);
                        let r = ((pixel >> 11) & 0x1F) as u8 * 8;
                        let g = ((pixel >> 5) & 0x3F) as u8 * 4;
                        let b = (pixel & 0x1F) as u8 * 8;
                        rgba_buffer.push(r);
                        rgba_buffer.push(g);
                        rgba_buffer.push(b);
                        rgba_buffer.push(255);
                    }
                }
            }
        }
        _ => {
            return Err(SikulixError::ScreenCaptureError(format!(
                "Unsupported depth: {}",
                depth
            )));
        }
    }

    Ok(rgba_buffer)
}

/// Move mouse to position
#[cfg(target_os = "linux")]
pub fn mouse_move(x: i32, y: i32) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::MouseError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Use XTest extension to move pointer
    conn.xtest_fake_input(6, 0, 0, root, x as i16, y as i16, 0) // MotionNotify = 6
        .map_err(|e| SikulixError::MouseError(format!("Failed to move mouse: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::MouseError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Click mouse button
#[cfg(target_os = "linux")]
pub fn mouse_click() -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::MouseError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Button press (1 = left button)
    conn.xtest_fake_input(4, 1, 0, root, 0, 0, 0) // ButtonPress = 4
        .map_err(|e| SikulixError::MouseError(format!("Failed to press button: {}", e)))?;

    // Button release
    conn.xtest_fake_input(5, 1, 0, root, 0, 0, 0) // ButtonRelease = 5
        .map_err(|e| SikulixError::MouseError(format!("Failed to release button: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::MouseError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Right-click mouse button
#[cfg(target_os = "linux")]
pub fn mouse_right_click() -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::MouseError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Button press (3 = right button)
    conn.xtest_fake_input(4, 3, 0, root, 0, 0, 0) // ButtonPress = 4
        .map_err(|e| SikulixError::MouseError(format!("Failed to press button: {}", e)))?;

    // Button release
    conn.xtest_fake_input(5, 3, 0, root, 0, 0, 0) // ButtonRelease = 5
        .map_err(|e| SikulixError::MouseError(format!("Failed to release button: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::MouseError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Get current mouse position
#[cfg(target_os = "linux")]
pub fn mouse_position() -> Result<(i32, i32)> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::MouseError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    let reply = conn
        .query_pointer(root)
        .map_err(|e| SikulixError::MouseError(format!("Failed to query pointer: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::MouseError(format!("Failed to get pointer reply: {}", e)))?;

    Ok((reply.root_x as i32, reply.root_y as i32))
}

/// Type text using keyboard
#[cfg(target_os = "linux")]
pub fn keyboard_type(text: &str) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    for ch in text.chars() {
        // For simplicity, only handle ASCII characters
        // Full Unicode support would require XKB extension
        if ch.is_ascii() {
            let keycode = char_to_keycode(ch);
            let need_shift = ch.is_ascii_uppercase() || is_shifted_char(ch);

            if need_shift {
                // Press Shift
                conn.xtest_fake_input(2, 50, 0, root, 0, 0, 0) // KeyPress, Shift_L = 50
                    .map_err(|e| {
                        SikulixError::KeyboardError(format!("Failed to press shift: {}", e))
                    })?;
            }

            // Key press
            conn.xtest_fake_input(2, keycode, 0, root, 0, 0, 0) // KeyPress = 2
                .map_err(|e| SikulixError::KeyboardError(format!("Failed to press key: {}", e)))?;

            // Key release
            conn.xtest_fake_input(3, keycode, 0, root, 0, 0, 0) // KeyRelease = 3
                .map_err(|e| {
                    SikulixError::KeyboardError(format!("Failed to release key: {}", e))
                })?;

            if need_shift {
                // Release Shift
                conn.xtest_fake_input(3, 50, 0, root, 0, 0, 0) // KeyRelease, Shift_L = 50
                    .map_err(|e| {
                        SikulixError::KeyboardError(format!("Failed to release shift: {}", e))
                    })?;
            }
        }
    }

    conn.flush()
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Press a key
#[cfg(target_os = "linux")]
pub fn keyboard_press(key: Key) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    let keycode = key_to_x11_keycode(key);

    conn.xtest_fake_input(2, keycode, 0, root, 0, 0, 0) // KeyPress = 2
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to press key: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Release a key
#[cfg(target_os = "linux")]
pub fn keyboard_release(key: Key) -> Result<()> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to connect to X11: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    let keycode = key_to_x11_keycode(key);

    conn.xtest_fake_input(3, keycode, 0, root, 0, 0, 0) // KeyRelease = 3
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to release key: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::KeyboardError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Convert ASCII character to X11 keycode (approximate, US layout)
#[cfg(target_os = "linux")]
fn char_to_keycode(ch: char) -> u8 {
    match ch.to_ascii_lowercase() {
        'a' => 38,
        'b' => 56,
        'c' => 54,
        'd' => 40,
        'e' => 26,
        'f' => 41,
        'g' => 42,
        'h' => 43,
        'i' => 31,
        'j' => 44,
        'k' => 45,
        'l' => 46,
        'm' => 58,
        'n' => 57,
        'o' => 32,
        'p' => 33,
        'q' => 24,
        'r' => 27,
        's' => 39,
        't' => 28,
        'u' => 30,
        'v' => 55,
        'w' => 25,
        'x' => 53,
        'y' => 29,
        'z' => 52,
        '0' | ')' => 19,
        '1' | '!' => 10,
        '2' | '@' => 11,
        '3' | '#' => 12,
        '4' | '$' => 13,
        '5' | '%' => 14,
        '6' | '^' => 15,
        '7' | '&' => 16,
        '8' | '*' => 17,
        '9' | '(' => 18,
        ' ' => 65,  // space
        '\n' => 36, // Return
        '\t' => 23, // Tab
        '-' | '_' => 20,
        '=' | '+' => 21,
        '[' | '{' => 34,
        ']' | '}' => 35,
        '\\' | '|' => 51,
        ';' | ':' => 47,
        '\'' | '"' => 48,
        ',' | '<' => 59,
        '.' | '>' => 60,
        '/' | '?' => 61,
        '`' | '~' => 49,
        _ => 65, // Default to space
    }
}

/// Check if character requires shift
#[cfg(target_os = "linux")]
fn is_shifted_char(ch: char) -> bool {
    matches!(
        ch,
        '!' | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '('
            | ')'
            | '_'
            | '+'
            | '{'
            | '}'
            | '|'
            | ':'
            | '"'
            | '<'
            | '>'
            | '?'
            | '~'
    )
}

/// Convert Key enum to X11 keycode
#[cfg(target_os = "linux")]
fn key_to_x11_keycode(key: Key) -> u8 {
    match key {
        Key::Shift => 50, // Shift_L
        Key::Ctrl => 37,  // Control_L
        Key::Alt => 64,   // Alt_L
        Key::Meta => 133, // Super_L
        Key::F1 => 67,
        Key::F2 => 68,
        Key::F3 => 69,
        Key::F4 => 70,
        Key::F5 => 71,
        Key::F6 => 72,
        Key::F7 => 73,
        Key::F8 => 74,
        Key::F9 => 75,
        Key::F10 => 76,
        Key::F11 => 95,
        Key::F12 => 96,
        Key::Enter => 36, // Return
        Key::Tab => 23,
        Key::Space => 65,
        Key::Backspace => 22,
        Key::Delete => 119,
        Key::Escape => 9,
        Key::Home => 110,
        Key::End => 115,
        Key::PageUp => 112,
        Key::PageDown => 117,
        Key::Up => 111,
        Key::Down => 116,
        Key::Left => 113,
        Key::Right => 114,
        Key::A => 38,
        Key::B => 56,
        Key::C => 54,
        Key::D => 40,
        Key::E => 26,
        Key::F => 41,
        Key::G => 42,
        Key::H => 43,
        Key::I => 31,
        Key::J => 44,
        Key::K => 45,
        Key::L => 46,
        Key::M => 58,
        Key::N => 57,
        Key::O => 32,
        Key::P => 33,
        Key::Q => 24,
        Key::R => 27,
        Key::S => 39,
        Key::T => 28,
        Key::U => 30,
        Key::V => 55,
        Key::W => 25,
        Key::X => 53,
        Key::Y => 29,
        Key::Z => 52,
        Key::Num0 => 19,
        Key::Num1 => 10,
        Key::Num2 => 11,
        Key::Num3 => 12,
        Key::Num4 => 13,
        Key::Num5 => 14,
        Key::Num6 => 15,
        Key::Num7 => 16,
        Key::Num8 => 17,
        Key::Num9 => 18,
    }
}

// Stub implementations for non-Linux builds
#[cfg(not(target_os = "linux"))]
pub fn get_screen_dimensions(_index: u32) -> Result<(u32, u32)> {
    Err(SikulixError::ScreenCaptureError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn capture_screen(_index: u32) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn capture_region(_index: u32, _region: &Region) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn mouse_move(_x: i32, _y: i32) -> Result<()> {
    Err(SikulixError::MouseError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn mouse_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn mouse_right_click() -> Result<()> {
    Err(SikulixError::MouseError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn mouse_position() -> Result<(i32, i32)> {
    Err(SikulixError::MouseError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn keyboard_type(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn keyboard_press(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "Linux implementation pending".to_string(),
    ))
}

#[cfg(not(target_os = "linux"))]
pub fn keyboard_release(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError(
        "Linux implementation pending".to_string(),
    ))
}
