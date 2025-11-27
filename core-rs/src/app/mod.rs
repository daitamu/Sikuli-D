//! Application control module
//! アプリケーション制御モジュール
//!
//! This module provides methods to control applications (focus, open, close, etc.)
//! このモジュールはアプリケーション制御メソッド（フォーカス、起動、終了など）を提供します
//!
//! # Example / 使用例
//!
//! ```no_run
//! use sikulid::App;
//!
//! // Open an application / アプリケーションを起動
//! let app = App::open("notepad.exe").unwrap();
//!
//! // Focus an existing application / 既存のアプリケーションにフォーカス
//! let mut app = App::new("Notepad");
//! app.focus().unwrap();
//!
//! // Check if running / 実行中か確認
//! if app.is_running() {
//!     println!("App is running");
//! }
//!
//! // Close application / アプリケーションを終了
//! app.close().unwrap();
//! ```

#[allow(unused_imports)]
use crate::{Region, Result, SikulixError};

#[cfg(target_os = "windows")]
mod windows_impl;

#[cfg(target_os = "macos")]
mod macos_impl;

#[cfg(target_os = "linux")]
mod linux_impl;

/// Application control structure
/// アプリケーション制御構造体
///
/// Represents a system application that can be controlled.
/// 制御可能なシステムアプリケーションを表します。
///
/// # Platform Support / プラットフォームサポート
///
/// - Windows: Full support via Win32 API / Win32 API経由で完全サポート
/// - macOS: Stub implementation (to be implemented) / スタブ実装（実装予定）
/// - Linux: Stub implementation (to be implemented) / スタブ実装（実装予定）
pub struct App {
    /// Application name or title / アプリケーション名またはタイトル
    pub name: String,

    #[cfg(target_os = "windows")]
    /// Windows window handle / Windowsウィンドウハンドル
    hwnd: Option<windows_impl::HWND>,

    #[cfg(target_os = "macos")]
    /// macOS process identifier / macOSプロセス識別子
    pid: Option<i32>,

    #[cfg(target_os = "linux")]
    /// Linux window identifier / Linuxウィンドウ識別子
    window_id: Option<u32>,
}

impl App {
    /// Create a new App instance for an existing application by name
    /// 既存のアプリケーションの新しいAppインスタンスを名前で作成
    ///
    /// This does not start the application, it only creates a handle to find it.
    /// これはアプリケーションを起動せず、検索するためのハンドルのみを作成します。
    ///
    /// # Arguments / 引数
    ///
    /// * `name` - Application name or window title / アプリケーション名またはウィンドウタイトル
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// let notepad = App::new("Notepad");
    /// ```
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),

            #[cfg(target_os = "windows")]
            hwnd: None,

            #[cfg(target_os = "macos")]
            pid: None,

            #[cfg(target_os = "linux")]
            window_id: None,
        }
    }

    /// Open an application by executable path or name
    /// 実行可能ファイルのパスまたは名前でアプリケーションを起動
    ///
    /// # Arguments / 引数
    ///
    /// * `path` - Path to executable or application name / 実行可能ファイルのパスまたはアプリケーション名
    ///
    /// # Returns / 戻り値
    ///
    /// App instance representing the opened application / 起動されたアプリケーションを表すAppインスタンス
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// // Open by name / 名前で起動
    /// let app = App::open("notepad.exe").unwrap();
    ///
    /// // Open by full path / フルパスで起動
    /// let app = App::open("C:\\Windows\\System32\\notepad.exe").unwrap();
    /// ```
    pub fn open(path: &str) -> Result<Self> {
        #[cfg(target_os = "windows")]
        {
            windows_impl::open_application(path)
        }

        #[cfg(target_os = "macos")]
        {
            macos_impl::open_application(path)
        }

        #[cfg(target_os = "linux")]
        {
            linux_impl::open_application(path)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(SikulixError::PlatformError(
                "App::open is not supported on this platform".to_string(),
            ))
        }
    }

    /// Bring the application window to foreground
    /// アプリケーションウィンドウを前面に表示
    ///
    /// # Returns / 戻り値
    ///
    /// Ok(()) if successful / 成功時はOk(())
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// let mut app = App::new("Notepad");
    /// app.focus().unwrap();
    /// ```
    pub fn focus(&mut self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            windows_impl::focus_application(self)
        }

        #[cfg(target_os = "macos")]
        {
            macos_impl::focus_application(self)
        }

        #[cfg(target_os = "linux")]
        {
            linux_impl::focus_application(self)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(SikulixError::PlatformError(
                "App::focus is not supported on this platform".to_string(),
            ))
        }
    }

    /// Close the application
    /// アプリケーションを終了
    ///
    /// Attempts to close the application gracefully.
    /// アプリケーションを正常に終了しようとします。
    ///
    /// # Returns / 戻り値
    ///
    /// Ok(()) if successful / 成功時はOk(())
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// let mut app = App::new("Notepad");
    /// app.close().unwrap();
    /// ```
    pub fn close(&mut self) -> Result<()> {
        #[cfg(target_os = "windows")]
        {
            windows_impl::close_application(self)
        }

        #[cfg(target_os = "macos")]
        {
            macos_impl::close_application(self)
        }

        #[cfg(target_os = "linux")]
        {
            linux_impl::close_application(self)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(SikulixError::PlatformError(
                "App::close is not supported on this platform".to_string(),
            ))
        }
    }

    /// Check if the application is currently running
    /// アプリケーションが現在実行中かを確認
    ///
    /// # Returns / 戻り値
    ///
    /// true if the application is running / アプリケーションが実行中の場合true
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// let app = App::new("Notepad");
    /// if app.is_running() {
    ///     println!("Notepad is running");
    /// }
    /// ```
    pub fn is_running(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            windows_impl::is_application_running(self)
        }

        #[cfg(target_os = "macos")]
        {
            macos_impl::is_application_running(self)
        }

        #[cfg(target_os = "linux")]
        {
            linux_impl::is_application_running(self)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            false
        }
    }

    /// Get the window region of the application
    /// アプリケーションのウィンドウ領域を取得
    ///
    /// Returns the bounding rectangle of the application's main window.
    /// アプリケーションのメインウィンドウの境界矩形を返します。
    ///
    /// # Returns / 戻り値
    ///
    /// Region representing the window bounds / ウィンドウ境界を表すRegion
    ///
    /// # Example / 使用例
    ///
    /// ```no_run
    /// use sikulid::App;
    ///
    /// let mut app = App::new("Notepad");
    /// let window = app.get_window().unwrap();
    /// println!("Window at ({}, {}), size {}x{}",
    ///     window.x, window.y, window.width, window.height);
    /// ```
    pub fn get_window(&mut self) -> Result<Region> {
        #[cfg(target_os = "windows")]
        {
            windows_impl::get_window_region(self)
        }

        #[cfg(target_os = "macos")]
        {
            macos_impl::get_window_region(self)
        }

        #[cfg(target_os = "linux")]
        {
            linux_impl::get_window_region(self)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(SikulixError::PlatformError(
                "App::get_window is not supported on this platform".to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new() {
        // Test creating a new App instance
        // 新しいAppインスタンスの作成をテスト
        let app = App::new("TestApp");
        assert_eq!(app.name, "TestApp");
    }

    #[test]
    fn test_app_name_with_spaces() {
        // Test app name with spaces
        // スペースを含むアプリ名をテスト
        let app = App::new("My Application");
        assert_eq!(app.name, "My Application");
    }

    #[test]
    fn test_app_empty_name() {
        // Test with empty name (valid but unlikely to match anything)
        // 空の名前でテスト（有効だが何もマッチしない可能性が高い）
        let app = App::new("");
        assert_eq!(app.name, "");
    }

    #[test]
    fn test_app_unicode_name() {
        // Test with Unicode characters
        // Unicode文字でテスト
        let app = App::new("メモ帳 Notepad");
        assert_eq!(app.name, "メモ帳 Notepad");
    }

    // Platform-specific tests would require actual applications running
    // These are integration tests and should be run manually or in CI
    // プラットフォーム固有のテストには実際のアプリケーションの実行が必要
    // これらは統合テストであり、手動またはCIで実行すべき

    #[test]
    #[ignore = "Requires actual application running"]
    fn integration_test_is_running() {
        // This test requires a known application to be running
        // このテストには既知のアプリケーションが実行されている必要がある
        #[cfg(target_os = "windows")]
        {
            let app = App::new("explorer.exe");
            // Windows Explorer should typically be running
            // Windowsエクスプローラーは通常実行されている
            assert!(app.is_running());
        }
    }

    #[test]
    #[ignore = "Requires screen interaction"]
    #[cfg(target_os = "windows")]
    fn integration_test_open_notepad() {
        // Test opening Notepad
        // メモ帳を開くテスト
        let result = App::open("notepad.exe");
        assert!(result.is_ok());

        if let Ok(mut app) = result {
            // Wait a moment for the app to start
            // アプリの起動を少し待つ
            std::thread::sleep(std::time::Duration::from_millis(500));

            assert!(app.is_running());

            // Clean up - close the app
            // クリーンアップ - アプリを閉じる
            let _ = app.close();
        }
    }

    #[test]
    #[ignore = "Requires screen interaction"]
    #[cfg(target_os = "windows")]
    fn integration_test_focus() {
        // Test focusing an application
        // アプリケーションのフォーカステスト
        let result = App::open("notepad.exe");
        assert!(result.is_ok());

        if let Ok(mut app) = result {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let focus_result = app.focus();
            assert!(focus_result.is_ok());

            // Clean up
            // クリーンアップ
            let _ = app.close();
        }
    }

    #[test]
    #[ignore = "Requires screen interaction"]
    #[cfg(target_os = "windows")]
    fn integration_test_get_window() {
        // Test getting window region
        // ウィンドウ領域取得のテスト
        let result = App::open("notepad.exe");
        assert!(result.is_ok());

        if let Ok(mut app) = result {
            std::thread::sleep(std::time::Duration::from_millis(500));

            let window_result = app.get_window();
            assert!(window_result.is_ok());

            if let Ok(window) = window_result {
                // Window should have positive dimensions
                // ウィンドウは正の寸法を持つべき
                assert!(window.width > 0);
                assert!(window.height > 0);
            }

            // Clean up
            // クリーンアップ
            let _ = app.close();
        }
    }
}
