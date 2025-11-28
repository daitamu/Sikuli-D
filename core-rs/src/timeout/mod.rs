//! Timeout handling utilities
//! タイムアウト処理ユーティリティ
//!
//! Provides comprehensive timeout management for SikuliX operations including:
//! - Synchronous and asynchronous timeout wrappers
//! - Cancellation tokens for long-running operations
//! - Default timeout configuration
//! - Integration with wait/exists/find operations
//!
//! SikuliX 操作のための包括的なタイムアウト管理を提供します：
//! - 同期および非同期タイムアウトラッパー
//! - 長時間実行操作用のキャンセルトークン
//! - デフォルトタイムアウト設定
//! - wait/exists/find 操作との統合

use crate::{Result, SikulixError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

/// Default timeout values for different operations
/// 各種操作のデフォルトタイムアウト値
#[derive(Debug, Clone, Copy)]
pub struct DefaultTimeouts {
    /// Pattern find timeout (default: 3s)
    /// パターン検索タイムアウト（デフォルト: 3秒）
    pub find: Duration,

    /// Pattern wait timeout (default: 3s)
    /// パターン待機タイムアウト（デフォルト: 3秒）
    pub wait: Duration,

    /// Pattern exists check timeout (default: 0s, immediate)
    /// パターン存在確認タイムアウト（デフォルト: 0秒、即時）
    pub exists: Duration,

    /// Script execution timeout (default: 10 minutes)
    /// スクリプト実行タイムアウト（デフォルト: 10分）
    pub script: Duration,

    /// Screen capture timeout (default: 5s)
    /// 画面キャプチャタイムアウト（デフォルト: 5秒）
    pub screen_capture: Duration,

    /// OCR operation timeout (default: 30s)
    /// OCR操作タイムアウト（デフォルト: 30秒）
    pub ocr: Duration,
}

impl Default for DefaultTimeouts {
    fn default() -> Self {
        Self {
            find: Duration::from_secs(3),
            wait: Duration::from_secs(3),
            exists: Duration::from_secs(0),
            script: Duration::from_secs(600),
            screen_capture: Duration::from_secs(5),
            ocr: Duration::from_secs(30),
        }
    }
}

impl DefaultTimeouts {
    /// Create new default timeouts
    /// 新しいデフォルトタイムアウトを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Get timeout for find operations
    /// find操作のタイムアウトを取得
    pub fn get_find(&self) -> Duration {
        self.find
    }

    /// Get timeout for wait operations
    /// wait操作のタイムアウトを取得
    pub fn get_wait(&self) -> Duration {
        self.wait
    }

    /// Get timeout for exists operations
    /// exists操作のタイムアウトを取得
    pub fn get_exists(&self) -> Duration {
        self.exists
    }

    /// Get timeout for script execution
    /// スクリプト実行のタイムアウトを取得
    pub fn get_script(&self) -> Duration {
        self.script
    }

    /// Get timeout for screen capture
    /// 画面キャプチャのタイムアウトを取得
    pub fn get_screen_capture(&self) -> Duration {
        self.screen_capture
    }

    /// Get timeout for OCR operations
    /// OCR操作のタイムアウトを取得
    pub fn get_ocr(&self) -> Duration {
        self.ocr
    }

    /// Builder method to set find timeout
    /// findタイムアウトを設定するビルダーメソッド
    pub fn with_find(mut self, duration: Duration) -> Self {
        self.find = duration;
        self
    }

    /// Builder method to set wait timeout
    /// waitタイムアウトを設定するビルダーメソッド
    pub fn with_wait(mut self, duration: Duration) -> Self {
        self.wait = duration;
        self
    }

    /// Builder method to set exists timeout
    /// existsタイムアウトを設定するビルダーメソッド
    pub fn with_exists(mut self, duration: Duration) -> Self {
        self.exists = duration;
        self
    }

    /// Builder method to set script timeout
    /// scriptタイムアウトを設定するビルダーメソッド
    pub fn with_script(mut self, duration: Duration) -> Self {
        self.script = duration;
        self
    }
}

/// Cancellation token for cancellable operations
/// キャンセル可能な操作用のキャンセルトークン
///
/// # Example / 例
///
/// ```
/// use sikulid::timeout::CancellationToken;
/// use std::thread;
/// use std::time::Duration;
///
/// let token = CancellationToken::new();
/// let token_clone = token.clone();
///
/// // Spawn a long-running operation
/// // 長時間実行操作を起動
/// let handle = thread::spawn(move || {
///     for i in 0..100 {
///         if token_clone.is_cancelled() {
///             println!("Operation cancelled at iteration {}", i);
///             return Err("Cancelled");
///         }
///         thread::sleep(Duration::from_millis(100));
///     }
///     Ok("Completed")
/// });
///
/// // Cancel after 1 second
/// // 1秒後にキャンセル
/// thread::sleep(Duration::from_secs(1));
/// token.cancel();
///
/// let result = handle.join().unwrap();
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    /// Create a new cancellation token
    /// 新しいキャンセルトークンを作成
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Cancel the operation
    /// 操作をキャンセル
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        log::debug!("Cancellation token triggered");
    }

    /// Check if the operation has been cancelled
    /// 操作がキャンセルされたかチェック
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Reset the cancellation state
    /// キャンセル状態をリセット
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a synchronous operation with timeout
/// 同期操作をタイムアウト付きで実行
///
/// # Arguments / 引数
///
/// * `timeout` - Maximum duration to wait / 最大待機時間
/// * `f` - The operation to execute / 実行する操作
///
/// # Returns / 戻り値
///
/// Returns Ok(T) if operation completes within timeout, or TimeoutError if timeout occurs
/// タイムアウト内に完了すればOk(T)を返し、タイムアウトが発生すればTimeoutErrorを返す
///
/// # Example / 例
///
/// ```
/// use sikulid::timeout::with_timeout;
/// use std::time::Duration;
/// use std::thread;
///
/// let result = with_timeout(Duration::from_secs(2), || {
///     thread::sleep(Duration::from_secs(1));
///     Ok(42)
/// });
///
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap(), 42);
///
/// let timeout_result = with_timeout(Duration::from_secs(1), || {
///     thread::sleep(Duration::from_secs(2));
///     Ok(42)
/// });
///
/// assert!(timeout_result.is_err());
/// ```
pub fn with_timeout<T, F>(timeout: Duration, f: F) -> Result<T>
where
    F: FnOnce() -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();

    // Spawn operation in a separate thread
    // 別スレッドで操作を起動
    thread::spawn(move || {
        let result = f();
        let _ = tx.send(result);
    });

    // Wait for result or timeout
    // 結果またはタイムアウトを待機
    match rx.recv_timeout(timeout) {
        Ok(result) => {
            log::debug!("Operation completed in {:?}", start.elapsed());
            result
        }
        Err(mpsc::RecvTimeoutError::Timeout) => {
            log::warn!("Operation timed out after {:?}", timeout);
            Err(SikulixError::FindFailed {
                pattern_name: "operation".to_string(),
                timeout_secs: timeout.as_secs_f64(),
            })
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            log::error!("Operation thread disconnected unexpectedly");
            Err(SikulixError::PlatformError(
                "Operation thread disconnected".to_string(),
            ))
        }
    }
}

/// Execute a synchronous operation with timeout and cancellation token
/// 同期操作をタイムアウトとキャンセルトークン付きで実行
///
/// # Arguments / 引数
///
/// * `timeout` - Maximum duration to wait / 最大待機時間
/// * `token` - Cancellation token / キャンセルトークン
/// * `f` - The operation to execute / 実行する操作
///
/// # Returns / 戻り値
///
/// Returns Ok(T) if operation completes, TimeoutError if timeout occurs, or error if cancelled
/// 完了すればOk(T)、タイムアウトならTimeoutError、キャンセルされればエラーを返す
pub fn with_timeout_and_cancel<T, F>(timeout: Duration, token: CancellationToken, f: F) -> Result<T>
where
    F: FnOnce(CancellationToken) -> Result<T> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    let start = Instant::now();
    let token_clone = token.clone();

    // Spawn operation in a separate thread
    // 別スレッドで操作を起動
    thread::spawn(move || {
        let result = f(token_clone);
        let _ = tx.send(result);
    });

    // Wait for result, timeout, or cancellation
    // 結果、タイムアウト、またはキャンセルを待機
    loop {
        if token.is_cancelled() {
            log::warn!("Operation cancelled by user");
            return Err(SikulixError::PlatformError(
                "Operation cancelled".to_string(),
            ));
        }

        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(result) => {
                log::debug!("Operation completed in {:?}", start.elapsed());
                return result;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                if start.elapsed() >= timeout {
                    log::warn!("Operation timed out after {:?}", timeout);
                    return Err(SikulixError::FindFailed {
                        pattern_name: "operation".to_string(),
                        timeout_secs: timeout.as_secs_f64(),
                    });
                }
                // Continue waiting
                // 待機を継続
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                log::error!("Operation thread disconnected unexpectedly");
                return Err(SikulixError::PlatformError(
                    "Operation thread disconnected".to_string(),
                ));
            }
        }
    }
}

/// Wait for a condition to become true with timeout
/// 条件が真になるまでタイムアウト付きで待機
///
/// # Arguments / 引数
///
/// * `timeout` - Maximum duration to wait / 最大待機時間
/// * `interval` - Check interval / チェック間隔
/// * `condition` - Condition function to check / チェックする条件関数
///
/// # Returns / 戻り値
///
/// Returns Ok(()) if condition becomes true, TimeoutError if timeout occurs
/// 条件が真になればOk(())、タイムアウトならTimeoutErrorを返す
pub fn wait_for_condition<F>(timeout: Duration, interval: Duration, mut condition: F) -> Result<()>
where
    F: FnMut() -> bool,
{
    let start = Instant::now();

    loop {
        if condition() {
            log::debug!("Condition met in {:?}", start.elapsed());
            return Ok(());
        }

        if start.elapsed() >= timeout {
            log::warn!("Condition not met after {:?}", timeout);
            return Err(SikulixError::FindFailed {
                pattern_name: "condition".to_string(),
                timeout_secs: timeout.as_secs_f64(),
            });
        }

        thread::sleep(interval);
    }
}

/// Wait for a condition to become true with timeout and cancellation
/// 条件が真になるまでタイムアウトとキャンセル付きで待機
///
/// # Arguments / 引数
///
/// * `timeout` - Maximum duration to wait / 最大待機時間
/// * `interval` - Check interval / チェック間隔
/// * `token` - Cancellation token / キャンセルトークン
/// * `condition` - Condition function to check / チェックする条件関数
pub fn wait_for_condition_with_cancel<F>(
    timeout: Duration,
    interval: Duration,
    token: &CancellationToken,
    mut condition: F,
) -> Result<()>
where
    F: FnMut() -> bool,
{
    let start = Instant::now();

    loop {
        if token.is_cancelled() {
            log::warn!("Wait cancelled by user");
            return Err(SikulixError::PlatformError("Wait cancelled".to_string()));
        }

        if condition() {
            log::debug!("Condition met in {:?}", start.elapsed());
            return Ok(());
        }

        if start.elapsed() >= timeout {
            log::warn!("Condition not met after {:?}", timeout);
            return Err(SikulixError::FindFailed {
                pattern_name: "condition".to_string(),
                timeout_secs: timeout.as_secs_f64(),
            });
        }

        thread::sleep(interval);
    }
}

/// Timeout guard for automatic timeout management
/// 自動タイムアウト管理用のタイムアウトガード
///
/// # Example / 例
///
/// ```
/// use sikulid::timeout::TimeoutGuard;
/// use std::time::Duration;
///
/// let guard = TimeoutGuard::new(Duration::from_secs(5));
///
/// loop {
///     if guard.is_expired() {
///         println!("Timeout!");
///         break;
///     }
///     // Do work...
///     // 作業を実行...
/// }
/// ```
pub struct TimeoutGuard {
    start: Instant,
    timeout: Duration,
}

impl TimeoutGuard {
    /// Create a new timeout guard
    /// 新しいタイムアウトガードを作成
    pub fn new(timeout: Duration) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }

    /// Check if timeout has expired
    /// タイムアウトが期限切れかチェック
    pub fn is_expired(&self) -> bool {
        self.start.elapsed() >= self.timeout
    }

    /// Get remaining time
    /// 残り時間を取得
    pub fn remaining(&self) -> Duration {
        self.timeout.saturating_sub(self.start.elapsed())
    }

    /// Get elapsed time
    /// 経過時間を取得
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Reset the timeout guard
    /// タイムアウトガードをリセット
    pub fn reset(&mut self) {
        self.start = Instant::now();
    }

    /// Check and return error if expired
    /// 期限切れならエラーを返す
    pub fn check(&self, operation_name: &str) -> Result<()> {
        if self.is_expired() {
            Err(SikulixError::FindFailed {
                pattern_name: operation_name.to_string(),
                timeout_secs: self.timeout.as_secs_f64(),
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(feature = "async")]
pub mod r#async {
    //! Async timeout utilities
    //! 非同期タイムアウトユーティリティ

    use crate::{Result, SikulixError};
    use std::future::Future;
    use std::time::Duration;

    /// Execute an async operation with timeout
    /// 非同期操作をタイムアウト付きで実行
    ///
    /// # Example / 例
    ///
    /// ```ignore
    /// use sikulid::timeout::async::with_timeout_async;
    /// use std::time::Duration;
    ///
    /// async fn example() {
    ///     let result = with_timeout_async(
    ///         Duration::from_secs(5),
    ///         async {
    ///             // Long running async operation
    ///             // 長時間実行非同期操作
    ///             Ok(42)
    ///         }
    ///     ).await;
    ///
    ///     assert!(result.is_ok());
    /// }
    /// ```
    pub async fn with_timeout_async<F, T>(timeout: Duration, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        match tokio::time::timeout(timeout, f).await {
            Ok(result) => result,
            Err(_) => {
                log::warn!("Async operation timed out after {:?}", timeout);
                Err(SikulixError::FindFailed {
                    pattern_name: "async_operation".to_string(),
                    timeout_secs: timeout.as_secs_f64(),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_timeouts() {
        let timeouts = DefaultTimeouts::new();
        assert_eq!(timeouts.find, Duration::from_secs(3));
        assert_eq!(timeouts.wait, Duration::from_secs(3));
        assert_eq!(timeouts.exists, Duration::from_secs(0));
        assert_eq!(timeouts.script, Duration::from_secs(600));
    }

    #[test]
    fn test_default_timeouts_builder() {
        let timeouts = DefaultTimeouts::new()
            .with_find(Duration::from_secs(5))
            .with_wait(Duration::from_secs(10));

        assert_eq!(timeouts.find, Duration::from_secs(5));
        assert_eq!(timeouts.wait, Duration::from_secs(10));
    }

    #[test]
    fn test_cancellation_token() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());

        token.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone() {
        let token1 = CancellationToken::new();
        let token2 = token1.clone();

        token1.cancel();
        assert!(token2.is_cancelled());
    }

    #[test]
    fn test_with_timeout_success() {
        let result = with_timeout(Duration::from_secs(2), || {
            thread::sleep(Duration::from_millis(100));
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_with_timeout_failure() {
        let result: Result<i32> = with_timeout(Duration::from_millis(100), || {
            thread::sleep(Duration::from_secs(1));
            Ok(42)
        });

        assert!(result.is_err());
    }

    #[test]
    fn test_with_timeout_and_cancel() {
        let token = CancellationToken::new();
        let token_clone = token.clone();

        // Spawn a thread to cancel after 200ms
        // 200ms後にキャンセルするスレッドを起動
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(200));
            token_clone.cancel();
        });

        let result: Result<i32> =
            with_timeout_and_cancel(Duration::from_secs(10), token, |token| {
                for _ in 0..100 {
                    if token.is_cancelled() {
                        return Err(SikulixError::PlatformError("Cancelled".to_string()));
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Ok(42)
            });

        assert!(result.is_err());
    }

    #[test]
    fn test_wait_for_condition_success() {
        let mut counter = 0;
        let result = wait_for_condition(Duration::from_secs(2), Duration::from_millis(50), || {
            counter += 1;
            counter >= 5
        });

        assert!(result.is_ok());
        assert!(counter >= 5);
    }

    #[test]
    fn test_wait_for_condition_timeout() {
        let result = wait_for_condition(
            Duration::from_millis(200),
            Duration::from_millis(50),
            || false,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_timeout_guard() {
        let guard = TimeoutGuard::new(Duration::from_millis(100));
        assert!(!guard.is_expired());

        thread::sleep(Duration::from_millis(150));
        assert!(guard.is_expired());
    }

    #[test]
    fn test_timeout_guard_remaining() {
        let guard = TimeoutGuard::new(Duration::from_secs(1));
        let remaining = guard.remaining();

        assert!(remaining > Duration::from_millis(900));
        assert!(remaining <= Duration::from_secs(1));
    }

    #[test]
    fn test_timeout_guard_reset() {
        let mut guard = TimeoutGuard::new(Duration::from_millis(100));
        thread::sleep(Duration::from_millis(150));
        assert!(guard.is_expired());

        guard.reset();
        assert!(!guard.is_expired());
    }

    #[test]
    fn test_timeout_guard_check() {
        let guard = TimeoutGuard::new(Duration::from_millis(100));
        assert!(guard.check("test_operation").is_ok());

        thread::sleep(Duration::from_millis(150));
        assert!(guard.check("test_operation").is_err());
    }
}
