//! Windows-specific screen capture and input implementation

use crate::screen::Key;
use crate::{Region, Result, SikulixError};
use image::{DynamicImage, RgbaImage};

#[cfg(target_os = "windows")]
use std::sync::Mutex;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::*, Win32::Graphics::Gdi::*, Win32::UI::Input::KeyboardAndMouse::*,
    Win32::UI::WindowsAndMessaging::*,
};

// ============================================================================
// Multi-monitor support (マルチモニターサポート)
// ============================================================================

/// Get the number of connected screens/monitors
/// 接続されている画面/モニターの数を取得
#[cfg(target_os = "windows")]
pub fn get_number_screens() -> u32 {
    unsafe { GetSystemMetrics(SM_CMONITORS) as u32 }
}

/// Monitor information structure
/// モニター情報構造体
#[cfg(target_os = "windows")]
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Monitor index (0 = primary) / モニターインデックス（0 = プライマリ）
    pub index: u32,
    /// X position / X座標
    pub x: i32,
    /// Y position / Y座標
    pub y: i32,
    /// Width in pixels / 幅（ピクセル）
    pub width: u32,
    /// Height in pixels / 高さ（ピクセル）
    pub height: u32,
    /// Is primary monitor / プライマリモニターかどうか
    pub is_primary: bool,
}

#[cfg(target_os = "windows")]
static MONITOR_LIST: Mutex<Vec<MonitorInfo>> = Mutex::new(Vec::new());

/// Callback for EnumDisplayMonitors
#[cfg(target_os = "windows")]
unsafe extern "system" fn monitor_enum_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _lprect: *mut RECT,
    _lparam: LPARAM,
) -> BOOL {
    let mut mi = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        rcMonitor: RECT::default(),
        rcWork: RECT::default(),
        dwFlags: 0,
    };

    if GetMonitorInfoW(hmonitor, &mut mi).as_bool() {
        let info = MonitorInfo {
            index: 0, // Will be assigned after sorting
            x: mi.rcMonitor.left,
            y: mi.rcMonitor.top,
            width: (mi.rcMonitor.right - mi.rcMonitor.left) as u32,
            height: (mi.rcMonitor.bottom - mi.rcMonitor.top) as u32,
            is_primary: (mi.dwFlags & MONITORINFOF_PRIMARY) != 0,
        };

        if let Ok(mut list) = MONITOR_LIST.lock() {
            list.push(info);
        }
    }

    BOOL::from(true) // Continue enumeration
}

/// Get information about all connected monitors
/// 接続されているすべてのモニターの情報を取得
#[cfg(target_os = "windows")]
pub fn get_all_monitors() -> Vec<MonitorInfo> {
    unsafe {
        // Clear the list first
        if let Ok(mut list) = MONITOR_LIST.lock() {
            list.clear();
        }

        // Enumerate all monitors
        let _ = EnumDisplayMonitors(HDC::default(), None, Some(monitor_enum_callback), LPARAM(0));

        // Get the results and sort by index (assign indices)
        if let Ok(mut list) = MONITOR_LIST.lock() {
            // Sort: primary first, then by x position
            list.sort_by(|a, b| {
                if a.is_primary && !b.is_primary {
                    std::cmp::Ordering::Less
                } else if !a.is_primary && b.is_primary {
                    std::cmp::Ordering::Greater
                } else {
                    a.x.cmp(&b.x)
                }
            });

            // Assign indices after sorting
            for (i, monitor) in list.iter_mut().enumerate() {
                monitor.index = i as u32;
            }

            list.clone()
        } else {
            Vec::new()
        }
    }
}

/// Get monitor info by index
/// インデックスでモニター情報を取得
#[cfg(target_os = "windows")]
pub fn get_monitor_info(index: u32) -> Option<MonitorInfo> {
    let monitors = get_all_monitors();
    monitors.into_iter().find(|m| m.index == index)
}

/// Get screen dimensions for the given monitor index
#[cfg(target_os = "windows")]
pub fn get_screen_dimensions(index: u32) -> Result<(u32, u32)> {
    // Use multi-monitor API
    if let Some(info) = get_monitor_info(index) {
        Ok((info.width, info.height))
    } else {
        // Fallback for primary monitor
        unsafe {
            if index == 0 {
                let width = GetSystemMetrics(SM_CXSCREEN) as u32;
                let height = GetSystemMetrics(SM_CYSCREEN) as u32;
                Ok((width, height))
            } else {
                Err(SikulixError::ScreenCaptureError(format!(
                    "Monitor {} not found",
                    index
                )))
            }
        }
    }
}

/// Capture the entire screen
#[cfg(target_os = "windows")]
pub fn capture_screen(index: u32) -> Result<DynamicImage> {
    let (width, height) = get_screen_dimensions(index)?;
    capture_region(index, &Region::new(0, 0, width, height))
}

/// Capture a specific region of the screen
#[cfg(target_os = "windows")]
pub fn capture_region(_index: u32, region: &Region) -> Result<DynamicImage> {
    unsafe {
        let hdc_screen = GetDC(HWND::default());
        if hdc_screen.is_invalid() {
            return Err(SikulixError::ScreenCaptureError(
                "Failed to get screen DC".to_string(),
            ));
        }

        let hdc_mem = CreateCompatibleDC(hdc_screen);
        if hdc_mem.is_invalid() {
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(SikulixError::ScreenCaptureError(
                "Failed to create memory DC".to_string(),
            ));
        }

        let width = region.width as i32;
        let height = region.height as i32;

        let hbitmap = CreateCompatibleBitmap(hdc_screen, width, height);
        if hbitmap.is_invalid() {
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(SikulixError::ScreenCaptureError(
                "Failed to create bitmap".to_string(),
            ));
        }

        let old_bitmap = SelectObject(hdc_mem, hbitmap);

        let result = BitBlt(
            hdc_mem, 0, 0, width, height, hdc_screen, region.x, region.y, SRCCOPY,
        );

        if result.is_err() {
            SelectObject(hdc_mem, old_bitmap);
            let _ = DeleteObject(hbitmap);
            let _ = DeleteDC(hdc_mem);
            ReleaseDC(HWND::default(), hdc_screen);
            return Err(SikulixError::ScreenCaptureError(
                "BitBlt failed".to_string(),
            ));
        }

        // Get bitmap data
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD::default()],
        };

        let mut buffer: Vec<u8> = vec![0; (width * height * 4) as usize];

        GetDIBits(
            hdc_mem,
            hbitmap,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Convert BGRA to RGBA
        for chunk in buffer.chunks_exact_mut(4) {
            chunk.swap(0, 2); // Swap B and R
        }

        SelectObject(hdc_mem, old_bitmap);
        let _ = DeleteObject(hbitmap);
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(HWND::default(), hdc_screen);

        let img = RgbaImage::from_raw(width as u32, height as u32, buffer).ok_or_else(|| {
            SikulixError::ScreenCaptureError("Failed to create image".to_string())
        })?;

        Ok(DynamicImage::ImageRgba8(img))
    }
}

/// Move mouse to position
#[cfg(target_os = "windows")]
pub fn mouse_move(x: i32, y: i32) -> Result<()> {
    unsafe {
        let result = SetCursorPos(x, y);
        if result.is_ok() {
            Ok(())
        } else {
            Err(SikulixError::MouseError(
                "Failed to move cursor".to_string(),
            ))
        }
    }
}

/// Click mouse button
#[cfg(target_os = "windows")]
pub fn mouse_click() -> Result<()> {
    unsafe {
        let input_down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_LEFTDOWN,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let input_up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_LEFTUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let inputs = [input_down, input_up];
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        Ok(())
    }
}

/// Right-click mouse button
#[cfg(target_os = "windows")]
pub fn mouse_right_click() -> Result<()> {
    unsafe {
        let input_down = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_RIGHTDOWN,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let input_up = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_RIGHTUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let inputs = [input_down, input_up];
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        Ok(())
    }
}

/// Get current mouse position
#[cfg(target_os = "windows")]
pub fn mouse_position() -> Result<(i32, i32)> {
    unsafe {
        let mut point = POINT::default();
        let result = GetCursorPos(&mut point);
        if result.is_ok() {
            Ok((point.x, point.y))
        } else {
            Err(SikulixError::MouseError(
                "Failed to get cursor position".to_string(),
            ))
        }
    }
}

/// Type text using keyboard
#[cfg(target_os = "windows")]
pub fn keyboard_type(text: &str) -> Result<()> {
    for ch in text.chars() {
        keyboard_type_char(ch)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn keyboard_type_char(ch: char) -> Result<()> {
    unsafe {
        // Use Unicode input
        let inputs = [
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: ch as u16,
                        dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        Ok(())
    }
}

// ============================================================================
// Internal helper functions (共通内部関数)
// These are NOT public APIs - they provide shared low-level functionality
// ============================================================================

/// Internal: Send key down event by virtual key code
#[cfg(target_os = "windows")]
fn send_key_down_vk(vk: VIRTUAL_KEY) {
    unsafe {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Internal: Send key up event by virtual key code
#[cfg(target_os = "windows")]
fn send_key_up_vk(vk: VIRTUAL_KEY) {
    unsafe {
        let input = INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: KEYEVENTF_KEYUP,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Internal: Send mouse button down event
#[cfg(target_os = "windows")]
fn send_mouse_down(flags: MOUSE_EVENT_FLAGS) {
    unsafe {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Internal: Send mouse button up event
#[cfg(target_os = "windows")]
fn send_mouse_up(flags: MOUSE_EVENT_FLAGS) {
    unsafe {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

/// Internal: Send mouse wheel event
#[cfg(target_os = "windows")]
fn send_mouse_wheel(flags: MOUSE_EVENT_FLAGS, delta: i32) {
    unsafe {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

// ============================================================================
// Public API functions (公開API関数)
// ============================================================================

/// Press a key
#[cfg(target_os = "windows")]
pub fn keyboard_press(key: Key) -> Result<()> {
    let vk = key_to_vk(key);
    send_key_down_vk(vk);
    Ok(())
}

/// Release a key
#[cfg(target_os = "windows")]
pub fn keyboard_release(key: Key) -> Result<()> {
    let vk = key_to_vk(key);
    send_key_up_vk(vk);
    Ok(())
}

/// Type text with delay between characters
#[cfg(target_os = "windows")]
pub fn keyboard_type_slow(text: &str, delay_ms: u64) -> Result<()> {
    for ch in text.chars() {
        keyboard_type_char(ch)?;
        if delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(delay_ms));
        }
    }
    Ok(())
}

/// Paste text via clipboard (uses internal helpers, not public APIs)
#[cfg(target_os = "windows")]
pub fn clipboard_paste_text(text: &str) -> Result<()> {
    use windows::Win32::System::DataExchange::*;
    use windows::Win32::System::Memory::*;

    unsafe {
        // Open clipboard
        if OpenClipboard(HWND::default()).is_err() {
            return Err(SikulixError::KeyboardError(
                "Failed to open clipboard".to_string(),
            ));
        }

        // Empty clipboard
        let _ = EmptyClipboard();

        // Convert text to wide string (UTF-16)
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let byte_len = wide.len() * 2;

        // Allocate global memory
        let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
        if hmem.is_err() {
            let _ = CloseClipboard();
            return Err(SikulixError::KeyboardError(
                "Failed to allocate memory".to_string(),
            ));
        }
        let hmem = hmem.unwrap();

        // Lock and copy data
        let ptr = GlobalLock(hmem);
        if ptr.is_null() {
            let _ = GlobalFree(hmem);
            let _ = CloseClipboard();
            return Err(SikulixError::KeyboardError(
                "Failed to lock memory".to_string(),
            ));
        }
        std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr as *mut u16, wide.len());
        let _ = GlobalUnlock(hmem);

        // Set clipboard data (CF_UNICODETEXT = 13)
        if SetClipboardData(13, windows::Win32::Foundation::HANDLE(hmem.0)).is_err() {
            let _ = GlobalFree(hmem);
            let _ = CloseClipboard();
            return Err(SikulixError::KeyboardError(
                "Failed to set clipboard data".to_string(),
            ));
        }

        let _ = CloseClipboard();

        // Send Ctrl+V using internal helpers (NOT public APIs)
        send_key_down_vk(VK_CONTROL);
        send_key_down_vk(VIRTUAL_KEY(0x56)); // V key
        std::thread::sleep(std::time::Duration::from_millis(10));
        send_key_up_vk(VIRTUAL_KEY(0x56)); // V key
        send_key_up_vk(VK_CONTROL);

        Ok(())
    }
}

/// Press mouse button down (without releasing)
#[cfg(target_os = "windows")]
pub fn mouse_down() -> Result<()> {
    send_mouse_down(MOUSEEVENTF_LEFTDOWN);
    Ok(())
}

/// Release mouse button
#[cfg(target_os = "windows")]
pub fn mouse_up() -> Result<()> {
    send_mouse_up(MOUSEEVENTF_LEFTUP);
    Ok(())
}

/// Middle-click mouse button
#[cfg(target_os = "windows")]
pub fn mouse_middle_click() -> Result<()> {
    send_mouse_down(MOUSEEVENTF_MIDDLEDOWN);
    send_mouse_up(MOUSEEVENTF_MIDDLEUP);
    Ok(())
}

/// Scroll mouse wheel vertically
#[cfg(target_os = "windows")]
pub fn mouse_scroll(clicks: i32) -> Result<()> {
    send_mouse_wheel(MOUSEEVENTF_WHEEL, clicks * 120); // WHEEL_DELTA = 120
    Ok(())
}

/// Scroll mouse wheel horizontally
#[cfg(target_os = "windows")]
pub fn mouse_scroll_horizontal(clicks: i32) -> Result<()> {
    send_mouse_wheel(MOUSEEVENTF_HWHEEL, clicks * 120); // WHEEL_DELTA = 120
    Ok(())
}

/// Convert Key enum to Windows virtual key code
#[cfg(target_os = "windows")]
fn key_to_vk(key: Key) -> VIRTUAL_KEY {
    match key {
        Key::Shift => VK_SHIFT,
        Key::Ctrl => VK_CONTROL,
        Key::Alt => VK_MENU,
        Key::Meta => VK_LWIN,
        Key::F1 => VK_F1,
        Key::F2 => VK_F2,
        Key::F3 => VK_F3,
        Key::F4 => VK_F4,
        Key::F5 => VK_F5,
        Key::F6 => VK_F6,
        Key::F7 => VK_F7,
        Key::F8 => VK_F8,
        Key::F9 => VK_F9,
        Key::F10 => VK_F10,
        Key::F11 => VK_F11,
        Key::F12 => VK_F12,
        Key::Enter => VK_RETURN,
        Key::Tab => VK_TAB,
        Key::Space => VK_SPACE,
        Key::Backspace => VK_BACK,
        Key::Delete => VK_DELETE,
        Key::Escape => VK_ESCAPE,
        Key::Home => VK_HOME,
        Key::End => VK_END,
        Key::PageUp => VK_PRIOR,
        Key::PageDown => VK_NEXT,
        Key::Up => VK_UP,
        Key::Down => VK_DOWN,
        Key::Left => VK_LEFT,
        Key::Right => VK_RIGHT,
        Key::A => VIRTUAL_KEY(0x41),
        Key::B => VIRTUAL_KEY(0x42),
        Key::C => VIRTUAL_KEY(0x43),
        Key::D => VIRTUAL_KEY(0x44),
        Key::E => VIRTUAL_KEY(0x45),
        Key::F => VIRTUAL_KEY(0x46),
        Key::G => VIRTUAL_KEY(0x47),
        Key::H => VIRTUAL_KEY(0x48),
        Key::I => VIRTUAL_KEY(0x49),
        Key::J => VIRTUAL_KEY(0x4A),
        Key::K => VIRTUAL_KEY(0x4B),
        Key::L => VIRTUAL_KEY(0x4C),
        Key::M => VIRTUAL_KEY(0x4D),
        Key::N => VIRTUAL_KEY(0x4E),
        Key::O => VIRTUAL_KEY(0x4F),
        Key::P => VIRTUAL_KEY(0x50),
        Key::Q => VIRTUAL_KEY(0x51),
        Key::R => VIRTUAL_KEY(0x52),
        Key::S => VIRTUAL_KEY(0x53),
        Key::T => VIRTUAL_KEY(0x54),
        Key::U => VIRTUAL_KEY(0x55),
        Key::V => VIRTUAL_KEY(0x56),
        Key::W => VIRTUAL_KEY(0x57),
        Key::X => VIRTUAL_KEY(0x58),
        Key::Y => VIRTUAL_KEY(0x59),
        Key::Z => VIRTUAL_KEY(0x5A),
        Key::Num0 => VIRTUAL_KEY(0x30),
        Key::Num1 => VIRTUAL_KEY(0x31),
        Key::Num2 => VIRTUAL_KEY(0x32),
        Key::Num3 => VIRTUAL_KEY(0x33),
        Key::Num4 => VIRTUAL_KEY(0x34),
        Key::Num5 => VIRTUAL_KEY(0x35),
        Key::Num6 => VIRTUAL_KEY(0x36),
        Key::Num7 => VIRTUAL_KEY(0x37),
        Key::Num8 => VIRTUAL_KEY(0x38),
        Key::Num9 => VIRTUAL_KEY(0x39),
    }
}

// Stub implementations for non-Windows builds

/// Non-Windows stub for MonitorInfo
#[cfg(not(target_os = "windows"))]
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    pub index: u32,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

#[cfg(not(target_os = "windows"))]
pub fn get_number_screens() -> u32 {
    1 // Return 1 as fallback
}

#[cfg(not(target_os = "windows"))]
pub fn get_all_monitors() -> Vec<MonitorInfo> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
pub fn get_monitor_info(_index: u32) -> Option<MonitorInfo> {
    None
}

#[cfg(not(target_os = "windows"))]
pub fn get_screen_dimensions(_index: u32) -> Result<(u32, u32)> {
    Err(SikulixError::ScreenCaptureError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn capture_screen(_index: u32) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn capture_region(_index: u32, _region: &Region) -> Result<DynamicImage> {
    Err(SikulixError::ScreenCaptureError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_move(_x: i32, _y: i32) -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_click() -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_right_click() -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_position() -> Result<(i32, i32)> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn keyboard_type(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn keyboard_press(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn keyboard_release(_key: Key) -> Result<()> {
    Err(SikulixError::KeyboardError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn keyboard_type_slow(_text: &str, _delay_ms: u64) -> Result<()> {
    Err(SikulixError::KeyboardError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn clipboard_paste_text(_text: &str) -> Result<()> {
    Err(SikulixError::KeyboardError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_down() -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_up() -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_middle_click() -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_scroll(_clicks: i32) -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}

#[cfg(not(target_os = "windows"))]
pub fn mouse_scroll_horizontal(_clicks: i32) -> Result<()> {
    Err(SikulixError::MouseError("Windows-only".to_string()))
}
