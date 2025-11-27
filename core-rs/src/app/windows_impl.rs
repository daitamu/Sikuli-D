//! Windows-specific application control implementation
//! Windows固有のアプリケーション制御実装

use crate::{Region, Result, SikulixError};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use windows::core::PCWSTR;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub use windows::Win32::Foundation::HWND;

/// Convert a Rust string to a Windows wide string
/// Rust文字列をWindowsワイド文字列に変換
fn to_wide_string(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

/// Find a window by its title
/// タイトルでウィンドウを検索
#[allow(dead_code)]
fn find_window_by_title(title: &str) -> Option<HWND> {
    let title_wide = to_wide_string(title);
    unsafe {
        match FindWindowW(None, PCWSTR(title_wide.as_ptr())) {
            Ok(hwnd) if !hwnd.is_invalid() => Some(hwnd),
            _ => None,
        }
    }
}

/// Callback data for EnumWindows
/// EnumWindowsのコールバックデータ
struct EnumWindowsData {
    search_text: String,
    found_hwnd: Option<HWND>,
}

/// Callback function for EnumWindows
/// EnumWindows用コールバック関数
unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let data = &mut *(lparam.0 as *mut EnumWindowsData);

    // Skip invisible windows
    // 非表示ウィンドウをスキップ
    if !IsWindowVisible(hwnd).as_bool() {
        return TRUE;
    }

    // Get window text
    // ウィンドウテキストを取得
    let mut text: [u16; 512] = [0; 512];
    let len = GetWindowTextW(hwnd, &mut text);

    if len > 0 {
        let window_title = String::from_utf16_lossy(&text[..len as usize]);

        // Check if the title contains the search text (case-insensitive)
        // タイトルに検索テキストが含まれているか確認（大文字小文字を区別しない）
        if window_title
            .to_lowercase()
            .contains(&data.search_text.to_lowercase())
        {
            data.found_hwnd = Some(hwnd);
            return FALSE; // Stop enumeration / 列挙を停止
        }
    }

    TRUE // Continue enumeration / 列挙を続行
}

/// Find a window by partial title match
/// 部分タイトルマッチでウィンドウを検索
fn find_window_by_partial_title(search_text: &str) -> Option<HWND> {
    let mut data = EnumWindowsData {
        search_text: search_text.to_string(),
        found_hwnd: None,
    };

    unsafe {
        let _ = EnumWindows(
            Some(enum_windows_callback),
            LPARAM(&mut data as *mut _ as isize),
        );
    }

    data.found_hwnd
}

/// Open an application by path or name
/// パスまたは名前でアプリケーションを起動
pub fn open_application(path: &str) -> Result<crate::App> {
    let path_wide = to_wide_string(path);

    unsafe {
        let result = ShellExecuteW(
            None,
            PCWSTR(to_wide_string("open").as_ptr()),
            PCWSTR(path_wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        );

        // ShellExecuteW returns a value > 32 on success
        // ShellExecuteWは成功時に32より大きい値を返す
        if result.0 as usize > 32 {
            // Extract application name from path
            // パスからアプリケーション名を抽出
            let app_name = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(path);

            // Wait a moment for the window to appear
            // ウィンドウが表示されるまで少し待つ
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Try to find the window
            // ウィンドウを検索
            let hwnd = find_window_by_partial_title(app_name);

            Ok(crate::App {
                name: app_name.to_string(),
                hwnd,
            })
        } else {
            Err(SikulixError::IoError(std::io::Error::last_os_error()))
        }
    }
}

/// Focus an application window
/// アプリケーションウィンドウにフォーカス
pub fn focus_application(app: &mut crate::App) -> Result<()> {
    // If we don't have an HWND, try to find the window
    // HWNDがない場合、ウィンドウを検索
    if app.hwnd.is_none() {
        app.hwnd = find_window_by_partial_title(&app.name);
    }

    if let Some(hwnd) = app.hwnd {
        unsafe {
            // Restore window if minimized
            // 最小化されている場合はウィンドウを復元
            if IsIconic(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_RESTORE);
            }

            // Bring window to foreground
            // ウィンドウを前面に表示
            if SetForegroundWindow(hwnd).as_bool() {
                Ok(())
            } else {
                Err(SikulixError::IoError(std::io::Error::last_os_error()))
            }
        }
    } else {
        Err(SikulixError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Window not found: {}", app.name),
        )))
    }
}

/// Close an application window
/// アプリケーションウィンドウを閉じる
pub fn close_application(app: &mut crate::App) -> Result<()> {
    // If we don't have an HWND, try to find the window
    // HWNDがない場合、ウィンドウを検索
    if app.hwnd.is_none() {
        app.hwnd = find_window_by_partial_title(&app.name);
    }

    if let Some(hwnd) = app.hwnd {
        unsafe {
            // Send WM_CLOSE message to close gracefully
            // WM_CLOSEメッセージを送信して正常に閉じる
            let result = PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0));

            if result.is_ok() {
                app.hwnd = None; // Clear the handle / ハンドルをクリア
                Ok(())
            } else {
                Err(SikulixError::IoError(std::io::Error::last_os_error()))
            }
        }
    } else {
        Err(SikulixError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Window not found: {}", app.name),
        )))
    }
}

/// Check if an application is running
/// アプリケーションが実行中か確認
pub fn is_application_running(app: &crate::App) -> bool {
    // If we have an HWND, check if it's still valid
    // HWNDがある場合、まだ有効か確認
    if let Some(hwnd) = app.hwnd {
        unsafe {
            if IsWindow(hwnd).as_bool() {
                return true;
            }
        }
    }

    // Otherwise, try to find the window by name
    // それ以外の場合、名前でウィンドウを検索
    find_window_by_partial_title(&app.name).is_some()
}

/// Get the window region of an application
/// アプリケーションのウィンドウ領域を取得
pub fn get_window_region(app: &mut crate::App) -> Result<Region> {
    // If we don't have an HWND, try to find the window
    // HWNDがない場合、ウィンドウを検索
    if app.hwnd.is_none() {
        app.hwnd = find_window_by_partial_title(&app.name);
    }

    if let Some(hwnd) = app.hwnd {
        unsafe {
            let mut rect = RECT::default();
            if GetWindowRect(hwnd, &mut rect).is_ok() {
                Ok(Region::new(
                    rect.left,
                    rect.top,
                    (rect.right - rect.left) as u32,
                    (rect.bottom - rect.top) as u32,
                ))
            } else {
                Err(SikulixError::IoError(std::io::Error::last_os_error()))
            }
        }
    } else {
        Err(SikulixError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Window not found: {}", app.name),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_wide_string() {
        // Test ASCII string
        // ASCII文字列をテスト
        let wide = to_wide_string("test");
        assert_eq!(wide, vec![116, 101, 115, 116, 0]); // "test\0" in UTF-16

        // Test empty string
        // 空文字列をテスト
        let wide = to_wide_string("");
        assert_eq!(wide, vec![0]);
    }

    #[test]
    fn test_to_wide_string_unicode() {
        // Test Unicode string
        // Unicode文字列をテスト
        let wide = to_wide_string("テスト");
        assert!(wide.len() > 1);
        assert_eq!(wide[wide.len() - 1], 0); // Should end with null terminator
    }

    // Integration tests requiring actual Windows environment
    // 実際のWindows環境が必要な統合テスト

    #[test]
    #[ignore = "Requires Windows environment"]
    fn integration_test_find_window() {
        // Try to find Windows Explorer (should always be running)
        // Windowsエクスプローラーを探す（常に実行されているはず）
        let hwnd = find_window_by_partial_title("explorer");
        // May or may not find it depending on system state
        // システムの状態により見つかる場合と見つからない場合がある
        println!("Found window: {:?}", hwnd);
    }

    #[test]
    #[ignore = "Requires Windows environment and user interaction"]
    fn integration_test_open_notepad() {
        // Test opening notepad
        // メモ帳を開くテスト
        let result = open_application("notepad.exe");
        assert!(result.is_ok());

        if let Ok(mut app) = result {
            // Wait for window to stabilize
            // ウィンドウが安定するまで待つ
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Check if running
            // 実行中か確認
            assert!(is_application_running(&app));

            // Clean up - close the application
            // クリーンアップ - アプリケーションを閉じる
            let _ = close_application(&mut app);

            // Wait for close
            // 閉じるのを待つ
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Should no longer be running
            // もう実行されていないはず
            assert!(!is_application_running(&app));
        }
    }
}
