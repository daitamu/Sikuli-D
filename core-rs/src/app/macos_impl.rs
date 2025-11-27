//! macOS-specific application control implementation
//! macOS固有のアプリケーション制御実装
//!
//! This implementation uses NSWorkspace and Accessibility APIs to control applications.
//! この実装はNSWorkspaceとAccessibility APIを使用してアプリケーションを制御します。

use crate::{Region, Result, SikulixError};
use cocoa::appkit::{NSRunningApplication, NSWorkspace};
use cocoa::base::{id, nil, NO, YES};
use cocoa::foundation::{NSArray, NSString};
use core_foundation::base::TCFType;
use core_foundation::string::{CFString, CFStringRef};
use objc::runtime::{Class, Object};
use objc::{class, msg_send, sel, sel_impl};
use std::process::Command;

/// Open an application by path or name
/// パスまたは名前でアプリケーションを起動
///
/// # Arguments / 引数
///
/// * `path` - Application path, name, or bundle identifier
///            アプリケーションのパス、名前、またはバンドルID
///
/// # Example / 使用例
///
/// ```no_run
/// open_application("/Applications/Safari.app");
/// open_application("Safari");
/// open_application("com.apple.Safari");
/// ```
pub fn open_application(path: &str) -> Result<crate::App> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];

        // Try different methods to open the application
        // 異なる方法でアプリケーションを開く試み
        let pid = if path.ends_with(".app") || path.starts_with("/") {
            // Open by path / パスで開く
            open_by_path(workspace, path)?
        } else if path.contains('.') && !path.contains('/') {
            // Assume bundle identifier / バンドルIDと仮定
            open_by_bundle_id(workspace, path)?
        } else {
            // Try as application name / アプリケーション名として試す
            open_by_name(workspace, path)?
        };

        Ok(crate::App {
            name: path.to_string(),
            pid: Some(pid),
        })
    }
}

/// Open application by file path
/// ファイルパスでアプリケーションを開く
unsafe fn open_by_path(workspace: id, path: &str) -> Result<i32> {
    let path_str = NSString::alloc(nil);
    let path_str = path_str.init_str(path);
    let url: id = msg_send![class!(NSURL), fileURLWithPath: path_str];

    if url == nil {
        return Err(SikulixError::AppError(format!(
            "Invalid application path: {}",
            path
        )));
    }

    // Create configuration / 設定を作成
    let config: id = msg_send![class!(NSWorkspaceOpenConfiguration), configuration];

    // Launch application / アプリケーションを起動
    let error: id = nil;
    let app: id = msg_send![workspace, openApplicationAtURL:url
                                        configuration:config
                                        completionHandler:nil];

    if app == nil {
        // Try using NSTask as fallback / フォールバックとしてNSTaskを使用
        return open_with_command(path);
    }

    let pid: i32 = msg_send![app, processIdentifier];
    Ok(pid)
}

/// Open application by bundle identifier
/// バンドルIDでアプリケーションを開く
unsafe fn open_by_bundle_id(workspace: id, bundle_id: &str) -> Result<i32> {
    let bundle_str = NSString::alloc(nil);
    let bundle_str = bundle_str.init_str(bundle_id);

    // Get URL for bundle identifier / バンドルIDのURLを取得
    let url: id = msg_send![workspace, URLForApplicationWithBundleIdentifier: bundle_str];

    if url == nil {
        return Err(SikulixError::AppError(format!(
            "Application with bundle ID '{}' not found",
            bundle_id
        )));
    }

    // Create configuration / 設定を作成
    let config: id = msg_send![class!(NSWorkspaceOpenConfiguration), configuration];

    // Launch application / アプリケーションを起動
    let app: id = msg_send![workspace, openApplicationAtURL:url
                                        configuration:config
                                        completionHandler:nil];

    if app == nil {
        return Err(SikulixError::AppError(format!(
            "Failed to launch application: {}",
            bundle_id
        )));
    }

    let pid: i32 = msg_send![app, processIdentifier];
    Ok(pid)
}

/// Open application by name
/// 名前でアプリケーションを開く
unsafe fn open_by_name(workspace: id, name: &str) -> Result<i32> {
    // Try common bundle ID patterns / 一般的なバンドルIDパターンを試す
    let possible_ids = vec![
        format!("com.apple.{}", name),
        format!("com.{}.{}", name.to_lowercase(), name),
        name.to_string(),
    ];

    for bundle_id in possible_ids {
        if let Ok(pid) = open_by_bundle_id(workspace, &bundle_id) {
            return Ok(pid);
        }
    }

    // Try with .app extension / .app拡張子で試す
    let app_path = format!("/Applications/{}.app", name);
    if let Ok(pid) = open_by_path(workspace, &app_path) {
        return Ok(pid);
    }

    // Last resort: use 'open' command / 最後の手段：'open'コマンドを使用
    open_with_command(name)
}

/// Open application using command line 'open'
/// コマンドライン'open'を使用してアプリケーションを開く
fn open_with_command(name: &str) -> Result<i32> {
    let output = Command::new("open")
        .arg("-a")
        .arg(name)
        .output()
        .map_err(|e| SikulixError::AppError(format!("Failed to execute 'open': {}", e)))?;

    if !output.status.success() {
        return Err(SikulixError::AppError(format!(
            "Failed to open application: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    // Wait a bit for app to start / アプリの起動を少し待つ
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Find PID by name / 名前でPIDを検索
    if let Some(pid) = find_pid_by_name(name) {
        Ok(pid)
    } else {
        // Return a placeholder, the app might still be starting
        // プレースホルダーを返す、アプリはまだ起動中かもしれない
        Ok(-1)
    }
}

/// Find process ID by application name
/// アプリケーション名でプロセスIDを検索
fn find_pid_by_name(name: &str) -> Option<i32> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let apps: id = msg_send![workspace, runningApplications];
        let count: usize = msg_send![apps, count];

        for i in 0..count {
            let app: id = msg_send![apps, objectAtIndex: i];
            let app_name: id = msg_send![app, localizedName];

            if app_name != nil {
                let app_name_str = nsstring_to_string(app_name);
                if app_name_str.contains(name) || name.contains(&app_name_str) {
                    let pid: i32 = msg_send![app, processIdentifier];
                    return Some(pid);
                }
            }
        }
        None
    }
}

/// Focus an application window
/// アプリケーションウィンドウにフォーカス
pub fn focus_application(app: &mut crate::App) -> Result<()> {
    let pid = app.pid.ok_or_else(|| {
        SikulixError::AppError("No PID available, application may not be running".to_string())
    })?;

    // If PID is -1, try to find it / PIDが-1の場合、検索を試みる
    let actual_pid = if pid == -1 {
        if let Some(found_pid) = find_pid_by_name(&app.name) {
            app.pid = Some(found_pid);
            found_pid
        } else {
            return Err(SikulixError::AppError(format!(
                "Application '{}' not found",
                app.name
            )));
        }
    } else {
        pid
    };

    unsafe {
        // Get running application by PID / PIDで実行中のアプリケーションを取得
        let ns_app: id = msg_send![
            class!(NSRunningApplication),
            runningApplicationWithProcessIdentifier: actual_pid
        ];

        if ns_app == nil {
            return Err(SikulixError::AppError(format!(
                "Application with PID {} not found",
                actual_pid
            )));
        }

        // Activate application (bring to front)
        // アプリケーションをアクティブ化（前面に表示）
        // NSApplicationActivateIgnoringOtherApps = 1 << 1 = 2
        let success: bool = msg_send![ns_app, activateWithOptions: 2];

        if success {
            Ok(())
        } else {
            Err(SikulixError::AppError(
                "Failed to activate application".to_string(),
            ))
        }
    }
}

/// Close an application window
/// アプリケーションウィンドウを閉じる
pub fn close_application(app: &mut crate::App) -> Result<()> {
    let pid = app.pid.ok_or_else(|| {
        SikulixError::AppError("No PID available, application may not be running".to_string())
    })?;

    // If PID is -1, try to find it / PIDが-1の場合、検索を試みる
    let actual_pid = if pid == -1 {
        if let Some(found_pid) = find_pid_by_name(&app.name) {
            app.pid = Some(found_pid);
            found_pid
        } else {
            return Err(SikulixError::AppError(format!(
                "Application '{}' not found",
                app.name
            )));
        }
    } else {
        pid
    };

    unsafe {
        let ns_app: id = msg_send![
            class!(NSRunningApplication),
            runningApplicationWithProcessIdentifier: actual_pid
        ];

        if ns_app == nil {
            return Err(SikulixError::AppError(format!(
                "Application with PID {} not found",
                actual_pid
            )));
        }

        // Try graceful termination first / まず正常終了を試みる
        let success: bool = msg_send![ns_app, terminate];

        if success {
            Ok(())
        } else {
            // Force termination / 強制終了
            let force_success: bool = msg_send![ns_app, forceTerminate];
            if force_success {
                Ok(())
            } else {
                Err(SikulixError::AppError(
                    "Failed to terminate application".to_string(),
                ))
            }
        }
    }
}

/// Check if an application is running
/// アプリケーションが実行中か確認
pub fn is_application_running(app: &crate::App) -> bool {
    if let Some(pid) = app.pid {
        if pid == -1 {
            // Try to find by name / 名前で検索を試みる
            return find_pid_by_name(&app.name).is_some();
        }

        unsafe {
            let ns_app: id = msg_send![
                class!(NSRunningApplication),
                runningApplicationWithProcessIdentifier: pid
            ];
            ns_app != nil
        }
    } else {
        // No PID, try to find by name / PIDなし、名前で検索を試みる
        find_pid_by_name(&app.name).is_some()
    }
}

/// Get the window region of an application
/// アプリケーションのウィンドウ領域を取得
pub fn get_window_region(app: &mut crate::App) -> Result<Region> {
    let pid = app.pid.ok_or_else(|| {
        SikulixError::AppError("No PID available, application may not be running".to_string())
    })?;

    // If PID is -1, try to find it / PIDが-1の場合、検索を試みる
    let actual_pid = if pid == -1 {
        if let Some(found_pid) = find_pid_by_name(&app.name) {
            app.pid = Some(found_pid);
            found_pid
        } else {
            return Err(SikulixError::AppError(format!(
                "Application '{}' not found",
                app.name
            )));
        }
    } else {
        pid
    };

    // Use Accessibility API to get window bounds
    // Accessibility APIを使用してウィンドウ境界を取得
    get_window_bounds_accessibility(actual_pid)
}

/// Get window bounds using Accessibility API
/// Accessibility APIを使用してウィンドウ境界を取得
fn get_window_bounds_accessibility(pid: i32) -> Result<Region> {
    // We need to link with ApplicationServices framework
    // ApplicationServicesフレームワークとリンクする必要があります

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn CFRelease(cf: CFTypeRef);
    }

    type AXUIElementRef = *const std::ffi::c_void;
    type CFTypeRef = *const std::ffi::c_void;

    unsafe {
        let app_element = AXUIElementCreateApplication(pid);
        if app_element.is_null() {
            return Err(SikulixError::AppError(
                "Failed to create AXUIElement for application".to_string(),
            ));
        }

        // Get focused window / フォーカスされたウィンドウを取得
        let focused_window_attr = CFString::new("AXFocusedWindow");
        let mut window: CFTypeRef = std::ptr::null();
        let result = AXUIElementCopyAttributeValue(
            app_element,
            focused_window_attr.as_concrete_TypeRef(),
            &mut window,
        );

        if result != 0 || window.is_null() {
            // Try to get main window instead / 代わりにメインウィンドウを取得
            let main_window_attr = CFString::new("AXMainWindow");
            let result = AXUIElementCopyAttributeValue(
                app_element,
                main_window_attr.as_concrete_TypeRef(),
                &mut window,
            );

            if result != 0 || window.is_null() {
                CFRelease(app_element as CFTypeRef);
                return Err(SikulixError::AppError(
                    "No window found for application".to_string(),
                ));
            }
        }

        // Get window position / ウィンドウ位置を取得
        let position_attr = CFString::new("AXPosition");
        let mut position: CFTypeRef = std::ptr::null();
        let pos_result = AXUIElementCopyAttributeValue(
            window as AXUIElementRef,
            position_attr.as_concrete_TypeRef(),
            &mut position,
        );

        // Get window size / ウィンドウサイズを取得
        let size_attr = CFString::new("AXSize");
        let mut size: CFTypeRef = std::ptr::null();
        let size_result = AXUIElementCopyAttributeValue(
            window as AXUIElementRef,
            size_attr.as_concrete_TypeRef(),
            &mut size,
        );

        if pos_result != 0 || size_result != 0 {
            if !position.is_null() {
                CFRelease(position);
            }
            if !size.is_null() {
                CFRelease(size);
            }
            CFRelease(window);
            CFRelease(app_element as CFTypeRef);
            return Err(SikulixError::AppError(
                "Failed to get window position or size".to_string(),
            ));
        }

        // Extract position and size from CFTypeRef
        // CFTypeRefから位置とサイズを抽出
        let (x, y) = extract_cgpoint(position)?;
        let (width, height) = extract_cgsize(size)?;

        // Clean up / クリーンアップ
        CFRelease(position);
        CFRelease(size);
        CFRelease(window);
        CFRelease(app_element as CFTypeRef);

        Ok(Region::new(x, y, width, height))
    }
}

/// Extract CGPoint from CFTypeRef
/// CFTypeRefからCGPointを抽出
fn extract_cgpoint(value: *const std::ffi::c_void) -> Result<(i32, i32)> {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXValueGetValue(
            value: *const std::ffi::c_void,
            value_type: i32,
            value_ptr: *mut std::ffi::c_void,
        ) -> bool;
    }

    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    const kAXValueCGPointType: i32 = 1;

    unsafe {
        let mut point = CGPoint { x: 0.0, y: 0.0 };
        let success = AXValueGetValue(
            value,
            kAXValueCGPointType,
            &mut point as *mut _ as *mut std::ffi::c_void,
        );

        if success {
            Ok((point.x as i32, point.y as i32))
        } else {
            Err(SikulixError::AppError(
                "Failed to extract CGPoint".to_string(),
            ))
        }
    }
}

/// Extract CGSize from CFTypeRef
/// CFTypeRefからCGSizeを抽出
fn extract_cgsize(value: *const std::ffi::c_void) -> Result<(u32, u32)> {
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXValueGetValue(
            value: *const std::ffi::c_void,
            value_type: i32,
            value_ptr: *mut std::ffi::c_void,
        ) -> bool;
    }

    #[repr(C)]
    struct CGSize {
        width: f64,
        height: f64,
    }

    const kAXValueCGSizeType: i32 = 2;

    unsafe {
        let mut size = CGSize {
            width: 0.0,
            height: 0.0,
        };
        let success = AXValueGetValue(
            value,
            kAXValueCGSizeType,
            &mut size as *mut _ as *mut std::ffi::c_void,
        );

        if success {
            Ok((size.width as u32, size.height as u32))
        } else {
            Err(SikulixError::AppError(
                "Failed to extract CGSize".to_string(),
            ))
        }
    }
}

/// Helper function to convert NSString to Rust String
/// NSStringをRust Stringに変換するヘルパー関数
fn nsstring_to_string(ns_string: id) -> String {
    unsafe {
        let c_str: *const i8 = msg_send![ns_string, UTF8String];
        if c_str.is_null() {
            String::new()
        } else {
            std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned()
        }
    }
}

/// Get frontmost (currently focused) application
/// 最前面の（現在フォーカスされている）アプリケーションを取得
#[allow(dead_code)]
pub fn get_foreground_app() -> Option<crate::App> {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let frontmost_app: id = msg_send![workspace, frontmostApplication];

        if frontmost_app == nil {
            return None;
        }

        let pid: i32 = msg_send![frontmost_app, processIdentifier];
        let app_name: id = msg_send![frontmost_app, localizedName];
        let name = nsstring_to_string(app_name);

        Some(crate::App {
            name,
            pid: Some(pid),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nsstring_conversion() {
        // Test NSString conversion (basic functionality)
        // NSString変換のテスト（基本機能）
        unsafe {
            let test_str = NSString::alloc(nil);
            let test_str = test_str.init_str("Hello");
            let result = nsstring_to_string(test_str);
            assert_eq!(result, "Hello");
        }
    }

    #[test]
    #[ignore = "Requires macOS system"]
    fn test_find_pid_by_name() {
        // Test finding Finder (should always be running on macOS)
        // Finderの検索をテスト（macOSでは常に実行されているはず）
        let pid = find_pid_by_name("Finder");
        assert!(pid.is_some());
        assert!(pid.unwrap() > 0);
    }

    #[test]
    #[ignore = "Requires macOS system"]
    fn test_is_running_finder() {
        // Test that Finder is running / Finderが実行中であることをテスト
        let app = crate::App::new("Finder");
        assert!(is_application_running(&app));
    }

    #[test]
    #[ignore = "Requires macOS system and user interaction"]
    fn integration_test_open_textedit() {
        // Test opening TextEdit / TextEditを開くテスト
        let result = open_application("TextEdit");
        assert!(result.is_ok());

        if let Ok(mut app) = result {
            std::thread::sleep(std::time::Duration::from_secs(1));

            // Check it's running / 実行中か確認
            assert!(is_application_running(&app));

            // Try to focus / フォーカスを試みる
            let focus_result = focus_application(&mut app);
            assert!(focus_result.is_ok());

            // Get window / ウィンドウを取得
            std::thread::sleep(std::time::Duration::from_millis(500));
            let window_result = get_window_region(&mut app);

            // Clean up / クリーンアップ
            let _ = close_application(&mut app);

            if let Ok(region) = window_result {
                assert!(region.width > 0);
                assert!(region.height > 0);
            }
        }
    }

    #[test]
    #[ignore = "Requires macOS system"]
    fn test_get_foreground_app() {
        // Test getting frontmost application / 最前面のアプリケーション取得をテスト
        let app = get_foreground_app();
        assert!(app.is_some());

        if let Some(foreground) = app {
            assert!(!foreground.name.is_empty());
            assert!(foreground.pid.is_some());
        }
    }
}
