//! Linux-specific application control implementation
//! Linux固有のアプリケーション制御実装
//!
//! This implementation uses X11 APIs for window management and /proc filesystem for process management.
//! この実装はウィンドウ管理にX11 APIを、プロセス管理に/procファイルシステムを使用します。

use crate::{Region, Result, SikulixError};
use std::process::Command;

#[cfg(target_os = "linux")]
use x11rb::connection::Connection;
#[cfg(target_os = "linux")]
use x11rb::protocol::xproto::{AtomEnum, ClientMessageEvent, ConnectionExt, EventMask, Window};
#[cfg(target_os = "linux")]
use x11rb::rust_connection::RustConnection;

/// Open an application by path or name
/// パスまたは名前でアプリケーションを起動
///
/// # Arguments / 引数
///
/// * `path` - Application executable path or name / アプリケーション実行可能ファイルのパスまたは名前
///
/// # Implementation / 実装
///
/// Uses `std::process::Command` to spawn the application process.
/// `std::process::Command`を使用してアプリケーションプロセスを起動します。
///
/// After launching, it attempts to find the window ID using _NET_WM_PID.
/// 起動後、_NET_WM_PIDを使用してウィンドウIDの検索を試みます。
#[cfg(target_os = "linux")]
pub fn open_application(path: &str) -> Result<crate::App> {
    // Split path and arguments if needed
    // 必要に応じてパスと引数を分割
    let parts: Vec<&str> = path.split_whitespace().collect();
    if parts.is_empty() {
        return Err(SikulixError::AppError("Empty application path".to_string()));
    }

    let exe = parts[0];
    let args = &parts[1..];

    // Spawn the process
    // プロセスを起動
    let child = Command::new(exe).args(args).spawn().map_err(|e| {
        SikulixError::AppError(format!("Failed to spawn application '{}': {}", exe, e))
    })?;

    let pid = child.id();

    // Wait a bit for the window to appear
    // ウィンドウが表示されるまで少し待つ
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Try to find the window ID
    // ウィンドウIDの検索を試行
    let window_id = find_window_by_pid(pid).ok().flatten();

    Ok(crate::App {
        name: exe.to_string(),
        window_id: window_id.map(|w| w as u32),
    })
}

/// Open an application by path or name (non-Linux stub)
/// パスまたは名前でアプリケーションを起動（非Linuxスタブ）
#[cfg(not(target_os = "linux"))]
pub fn open_application(path: &str) -> Result<crate::App> {
    Err(SikulixError::PlatformError(format!(
        "App::open not available on this platform (path: {})",
        path
    )))
}

/// Find window ID by process ID using _NET_WM_PID property
/// _NET_WM_PIDプロパティを使用してプロセスIDでウィンドウIDを検索
///
/// # Arguments / 引数
///
/// * `pid` - Process ID to search for / 検索するプロセスID
///
/// # Returns / 戻り値
///
/// Window ID if found / 見つかった場合はウィンドウID
#[cfg(target_os = "linux")]
fn find_window_by_pid(pid: u32) -> Result<Option<Window>> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::AppError(format!("X11 connection failed: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Get _NET_CLIENT_LIST atom
    // _NET_CLIENT_LISTアトムを取得
    let net_client_list = conn
        .intern_atom(false, b"_NET_CLIENT_LIST")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Get list of all client windows
    // すべてのクライアントウィンドウのリストを取得
    let windows = conn
        .get_property(false, root, net_client_list, AtomEnum::WINDOW, 0, u32::MAX)
        .map_err(|e| SikulixError::AppError(format!("Failed to get property: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get property reply: {}", e)))?;

    // Parse window list
    // ウィンドウリストを解析
    let window_list: &[Window] = if windows.value.len() >= 4 {
        unsafe {
            std::slice::from_raw_parts(
                windows.value.as_ptr() as *const Window,
                windows.value.len() / 4,
            )
        }
    } else {
        return Ok(None);
    };

    // Get _NET_WM_PID atom
    // _NET_WM_PIDアトムを取得
    let net_wm_pid = conn
        .intern_atom(false, b"_NET_WM_PID")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Check each window's PID
    // 各ウィンドウのPIDを確認
    for &window in window_list {
        let prop = conn
            .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)
            .map_err(|e| SikulixError::AppError(format!("Failed to get property: {}", e)))?
            .reply()
            .map_err(|e| SikulixError::AppError(format!("Failed to get property reply: {}", e)))?;

        if prop.value.len() >= 4 {
            let window_pid =
                u32::from_ne_bytes([prop.value[0], prop.value[1], prop.value[2], prop.value[3]]);

            if window_pid == pid {
                return Ok(Some(window));
            }
        }
    }

    Ok(None)
}

/// Focus an application window using _NET_ACTIVE_WINDOW
/// _NET_ACTIVE_WINDOWを使用してアプリケーションウィンドウをフォーカス
///
/// # Arguments / 引数
///
/// * `app` - Application to focus / フォーカスするアプリケーション
///
/// # Implementation / 実装
///
/// Sends a ClientMessage event to the root window to activate the target window.
/// ルートウィンドウにClientMessageイベントを送信してターゲットウィンドウをアクティブ化します。
#[cfg(target_os = "linux")]
pub fn focus_application(app: &mut crate::App) -> Result<()> {
    // Try to get window ID from app, or search by name
    // アプリからウィンドウIDを取得するか、名前で検索
    let window = if let Some(wid) = app.window_id {
        wid as Window
    } else {
        // Try to find window by name
        // 名前でウィンドウを検索
        find_window_by_name(&app.name)?
            .ok_or_else(|| SikulixError::AppError(format!("Window '{}' not found", app.name)))?
    };

    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::AppError(format!("X11 connection failed: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Get _NET_ACTIVE_WINDOW atom
    // _NET_ACTIVE_WINDOWアトムを取得
    let net_active_window = conn
        .intern_atom(false, b"_NET_ACTIVE_WINDOW")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Send client message to activate window
    // ウィンドウをアクティブ化するためのクライアントメッセージを送信
    let event = ClientMessageEvent {
        response_type: 33, // ClientMessage
        format: 32,
        sequence: 0,
        window,
        type_: net_active_window,
        data: [2, 0, 0, 0, 0].into(), // source=2 (pager)
    };

    conn.send_event(
        false,
        root,
        EventMask::SUBSTRUCTURE_NOTIFY | EventMask::SUBSTRUCTURE_REDIRECT,
        &event,
    )
    .map_err(|e| SikulixError::AppError(format!("Failed to send event: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::AppError(format!("Failed to flush: {}", e)))?;

    // Update window_id in app if it was found
    // 見つかった場合はアプリのwindow_idを更新
    if app.window_id.is_none() {
        app.window_id = Some(window as u32);
    }

    Ok(())
}

/// Focus an application window (non-Linux stub)
/// アプリケーションウィンドウをフォーカス（非Linuxスタブ）
#[cfg(not(target_os = "linux"))]
pub fn focus_application(_app: &mut crate::App) -> Result<()> {
    Err(SikulixError::PlatformError(
        "App::focus not available on this platform".to_string(),
    ))
}

/// Find window by window title or class name
/// ウィンドウタイトルまたはクラス名でウィンドウを検索
#[cfg(target_os = "linux")]
fn find_window_by_name(name: &str) -> Result<Option<Window>> {
    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| SikulixError::AppError(format!("X11 connection failed: {}", e)))?;

    let setup = conn.setup();
    let screen = &setup.roots[screen_num];
    let root = screen.root;

    // Get _NET_CLIENT_LIST atom
    // _NET_CLIENT_LISTアトムを取得
    let net_client_list = conn
        .intern_atom(false, b"_NET_CLIENT_LIST")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Get list of all client windows
    // すべてのクライアントウィンドウのリストを取得
    let windows = conn
        .get_property(false, root, net_client_list, AtomEnum::WINDOW, 0, u32::MAX)
        .map_err(|e| SikulixError::AppError(format!("Failed to get property: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get property reply: {}", e)))?;

    // Parse window list
    // ウィンドウリストを解析
    let window_list: &[Window] = if windows.value.len() >= 4 {
        unsafe {
            std::slice::from_raw_parts(
                windows.value.as_ptr() as *const Window,
                windows.value.len() / 4,
            )
        }
    } else {
        return Ok(None);
    };

    // Get WM_NAME atom for window titles
    // ウィンドウタイトル用のWM_NAMEアトムを取得
    let wm_name = conn
        .intern_atom(false, b"WM_NAME")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    let net_wm_name = conn
        .intern_atom(false, b"_NET_WM_NAME")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    let utf8_string = conn
        .intern_atom(false, b"UTF8_STRING")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Check each window's title
    // 各ウィンドウのタイトルを確認
    for &window in window_list {
        // Try _NET_WM_NAME first (UTF-8)
        // まず_NET_WM_NAMEを試す（UTF-8）
        if let Ok(prop) = conn
            .get_property(false, window, net_wm_name, utf8_string, 0, 1024)
            .and_then(|cookie| cookie.reply())
        {
            if !prop.value.is_empty() {
                if let Ok(title) = String::from_utf8(prop.value.clone()) {
                    if title.contains(name) {
                        return Ok(Some(window));
                    }
                }
            }
        }

        // Fallback to WM_NAME
        // WM_NAMEにフォールバック
        if let Ok(prop) = conn
            .get_property(false, window, wm_name, AtomEnum::STRING, 0, 1024)
            .and_then(|cookie| cookie.reply())
        {
            if !prop.value.is_empty() {
                if let Ok(title) = String::from_utf8(prop.value.clone()) {
                    if title.contains(name) {
                        return Ok(Some(window));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Close an application window using WM_DELETE_WINDOW protocol
/// WM_DELETE_WINDOWプロトコルを使用してアプリケーションウィンドウを閉じる
///
/// # Arguments / 引数
///
/// * `app` - Application to close / 閉じるアプリケーション
///
/// # Implementation / 実装
///
/// Sends a WM_DELETE_WINDOW message to gracefully close the window.
/// WM_DELETE_WINDOWメッセージを送信してウィンドウを正常に閉じます。
#[cfg(target_os = "linux")]
pub fn close_application(app: &mut crate::App) -> Result<()> {
    // Try to get window ID from app, or search by name
    // アプリからウィンドウIDを取得するか、名前で検索
    let window = if let Some(wid) = app.window_id {
        wid as Window
    } else {
        find_window_by_name(&app.name)?
            .ok_or_else(|| SikulixError::AppError(format!("Window '{}' not found", app.name)))?
    };

    let (conn, _) = RustConnection::connect(None)
        .map_err(|e| SikulixError::AppError(format!("X11 connection failed: {}", e)))?;

    // Get WM_DELETE_WINDOW and WM_PROTOCOLS atoms
    // WM_DELETE_WINDOWとWM_PROTOCOLSアトムを取得
    let wm_protocols = conn
        .intern_atom(false, b"WM_PROTOCOLS")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    let wm_delete_window = conn
        .intern_atom(false, b"WM_DELETE_WINDOW")
        .map_err(|e| SikulixError::AppError(format!("Failed to intern atom: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get atom reply: {}", e)))?
        .atom;

    // Send WM_DELETE_WINDOW message
    // WM_DELETE_WINDOWメッセージを送信
    let event = ClientMessageEvent {
        response_type: 33, // ClientMessage
        format: 32,
        sequence: 0,
        window,
        type_: wm_protocols,
        data: [wm_delete_window, 0, 0, 0, 0].into(),
    };

    conn.send_event(false, window, EventMask::NO_EVENT, &event)
        .map_err(|e| SikulixError::AppError(format!("Failed to send event: {}", e)))?;

    conn.flush()
        .map_err(|e| SikulixError::AppError(format!("Failed to flush: {}", e)))?;

    Ok(())
}

/// Close an application window (non-Linux stub)
/// アプリケーションウィンドウを閉じる（非Linuxスタブ）
#[cfg(not(target_os = "linux"))]
pub fn close_application(_app: &mut crate::App) -> Result<()> {
    Err(SikulixError::PlatformError(
        "App::close not available on this platform".to_string(),
    ))
}

/// Check if an application is running using /proc filesystem
/// /procファイルシステムを使用してアプリケーションが実行中か確認
///
/// # Arguments / 引数
///
/// * `app` - Application to check / 確認するアプリケーション
///
/// # Implementation / 実装
///
/// Checks if the application has a valid window or searches by name.
/// アプリケーションが有効なウィンドウを持っているか、または名前で検索して確認します。
#[cfg(target_os = "linux")]
pub fn is_application_running(app: &crate::App) -> bool {
    // If we have a window ID, check if it still exists
    // ウィンドウIDがある場合、それがまだ存在するか確認
    if let Some(wid) = app.window_id {
        if let Ok((conn, _)) = RustConnection::connect(None) {
            // Try to get window attributes
            // ウィンドウ属性の取得を試行
            if conn.get_window_attributes(wid).is_ok() {
                return true;
            }
        }
    }

    // Fallback: search by name
    // フォールバック: 名前で検索
    if let Ok(Some(_)) = find_window_by_name(&app.name) {
        return true;
    }

    false
}

/// Check if an application is running (non-Linux stub)
/// アプリケーションが実行中か確認（非Linuxスタブ）
#[cfg(not(target_os = "linux"))]
pub fn is_application_running(_app: &crate::App) -> bool {
    false
}

/// Get the window region of an application using X11 geometry
/// X11ジオメトリを使用してアプリケーションのウィンドウ領域を取得
///
/// # Arguments / 引数
///
/// * `app` - Application to get window region for / ウィンドウ領域を取得するアプリケーション
///
/// # Returns / 戻り値
///
/// Region representing the window bounds / ウィンドウ境界を表すRegion
#[cfg(target_os = "linux")]
pub fn get_window_region(app: &mut crate::App) -> Result<Region> {
    // Try to get window ID from app, or search by name
    // アプリからウィンドウIDを取得するか、名前で検索
    let window = if let Some(wid) = app.window_id {
        wid as Window
    } else {
        let found_window = find_window_by_name(&app.name)?
            .ok_or_else(|| SikulixError::AppError(format!("Window '{}' not found", app.name)))?;

        // Update window_id in app
        // アプリのwindow_idを更新
        app.window_id = Some(found_window as u32);
        found_window
    };

    let (conn, _) = RustConnection::connect(None)
        .map_err(|e| SikulixError::AppError(format!("X11 connection failed: {}", e)))?;

    // Get window geometry
    // ウィンドウジオメトリを取得
    let geom = conn
        .get_geometry(window)
        .map_err(|e| SikulixError::AppError(format!("Failed to get geometry: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get geometry reply: {}", e)))?;

    // Get window translation to root coordinates
    // ルート座標へのウィンドウ変換を取得
    let translate = conn
        .translate_coordinates(window, geom.root, 0, 0)
        .map_err(|e| SikulixError::AppError(format!("Failed to translate coordinates: {}", e)))?
        .reply()
        .map_err(|e| SikulixError::AppError(format!("Failed to get translate reply: {}", e)))?;

    Ok(Region::new(
        translate.dst_x as i32,
        translate.dst_y as i32,
        geom.width as u32,
        geom.height as u32,
    ))
}

/// Get the window region of an application (non-Linux stub)
/// アプリケーションのウィンドウ領域を取得（非Linuxスタブ）
#[cfg(not(target_os = "linux"))]
pub fn get_window_region(_app: &mut crate::App) -> Result<Region> {
    Err(SikulixError::PlatformError(
        "App::get_window not available on this platform".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_open_empty_path() {
        // Test with empty path
        // 空のパスでテスト
        let result = open_application("");
        assert!(result.is_err());
        if let Err(SikulixError::AppError(msg)) = result {
            assert!(msg.contains("Empty"));
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_open_nonexistent_app() {
        // Test opening a non-existent application
        // 存在しないアプリケーションの起動をテスト
        let result = open_application("nonexistent_app_12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_is_running_nonexistent() {
        // Test checking if non-existent app is running
        // 存在しないアプリが実行中か確認するテスト
        let app = crate::App::new("NonExistentApp12345");
        assert!(!is_application_running(&app));
    }

    #[test]
    #[ignore = "Requires X11 session"]
    #[cfg(target_os = "linux")]
    fn integration_test_find_window() {
        // Integration test for finding windows
        // ウィンドウ検索の統合テスト
        // This test requires an X11 session with at least one window
        // このテストにはX11セッションと少なくとも1つのウィンドウが必要

        // Try to connect to X11
        // X11への接続を試行
        if let Ok((conn, screen_num)) = RustConnection::connect(None) {
            let setup = conn.setup();
            let screen = &setup.roots[screen_num];

            // Just verify we can connect
            // 接続できることを確認
            assert!(screen.width_in_pixels > 0);
        }
    }

    #[test]
    #[ignore = "Requires actual application"]
    #[cfg(target_os = "linux")]
    fn integration_test_open_xterm() {
        // Integration test for opening xterm (if available)
        // xterm起動の統合テスト（利用可能な場合）

        // Try to open xterm
        // xtermを起動
        if let Ok(mut app) = open_application("xterm") {
            // Wait for window to appear
            // ウィンドウが表示されるまで待つ
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Check if running
            // 実行中か確認
            assert!(is_application_running(&app));

            // Try to get window
            // ウィンドウを取得
            if let Ok(region) = get_window_region(&mut app) {
                assert!(region.width > 0);
                assert!(region.height > 0);
            }

            // Close the app
            // アプリを閉じる
            let _ = close_application(&mut app);

            // Wait for close
            // 閉じるまで待つ
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    }
}
