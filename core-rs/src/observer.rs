//! Background observation and monitoring module
//! バックグラウンド監視モジュール
//!
//! Provides event-driven monitoring of screen regions with callbacks.
//! Supports pattern appearance, vanishing, and visual change detection.
//!
//! 画面領域のイベント駆動型監視をコールバックで提供します。
//! パターンの出現、消失、視覚的変化の検出をサポートします。
//!
//! # Example / 使用例
//!
//! ```ignore
//! use sikulid::{Observer, Pattern, Region};
//!
//! let region = Region::new(0, 0, 800, 600);
//! let mut observer = Observer::new(region);
//!
//! // Register appearance handler
//! // 出現ハンドラーを登録
//! let pattern = Pattern::from_file("button.png")?;
//! observer.on_appear(pattern, |m| {
//!     println!("Button appeared at ({}, {})", m.get_x(), m.get_y());
//! });
//!
//! // Start observing in background
//! // バックグラウンドで監視開始
//! let handle = observer.observe_in_background();
//!
//! // Do other work...
//! // 他の処理...
//!
//! // Stop observation
//! // 監視停止
//! observer.stop();
//! handle.join().unwrap()?;
//! ```

use crate::image::ImageMatcher;
use crate::screen::Screen;
use crate::{Match, Pattern, Region, Result};
use image::DynamicImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// Default observation interval in milliseconds
/// デフォルト監視間隔（ミリ秒）
const DEFAULT_OBSERVATION_INTERVAL_MS: u64 = 500;

/// Minimum change threshold (0.0 - 1.0) for change detection
/// 変化検出のための最小変化閾値（0.0 - 1.0）
#[allow(dead_code)]
const DEFAULT_CHANGE_THRESHOLD: f64 = 0.05;

/// Event handler for pattern appearance
/// パターン出現用イベントハンドラー
type AppearHandler = Box<dyn Fn(&Match) + Send + 'static>;

/// Event handler for pattern vanishing
/// パターン消失用イベントハンドラー
type VanishHandler = Box<dyn Fn() + Send + 'static>;

/// Event handler for visual changes
/// 視覚的変化用イベントハンドラー
type ChangeHandler = Box<dyn Fn(f64) + Send + 'static>;

/// Observer for background region monitoring
/// バックグラウンド領域監視用オブザーバー
///
/// Monitors a screen region for pattern appearances, disappearances, and visual changes.
/// Runs in a background thread and invokes registered callbacks when events occur.
///
/// 画面領域でパターンの出現、消失、視覚的変化を監視します。
/// バックグラウンドスレッドで実行され、イベント発生時に登録されたコールバックを呼び出します。
///
/// # Thread Safety / スレッド安全性
///
/// Observer uses Arc<AtomicBool> for thread-safe stop signaling and Arc<Mutex<T>> for
/// shared state protection. All callbacks must be Send + 'static.
///
/// ObserverはスレッドセーフなストップシグナルにArc<AtomicBool>を使用し、
/// 共有状態の保護にArc<Mutex<T>>を使用します。全てのコールバックはSend + 'staticである必要があります。
pub struct Observer {
    /// Region to observe
    /// 監視対象の領域
    region: Region,

    /// Running flag (thread-safe)
    /// 実行フラグ（スレッドセーフ）
    running: Arc<AtomicBool>,

    /// Observation interval in milliseconds
    /// 監視間隔（ミリ秒）
    interval_ms: u64,

    /// Image matcher for pattern detection
    /// パターン検出用画像マッチャー
    matcher: ImageMatcher,

    /// Appearance handlers (pattern, handler)
    /// 出現ハンドラー（パターン、ハンドラー）
    appear_handlers: Arc<Mutex<Vec<(Pattern, AppearHandler)>>>,

    /// Vanish handlers (pattern, last_seen, handler)
    /// 消失ハンドラー（パターン、最終確認時刻、ハンドラー）
    vanish_handlers: Arc<Mutex<Vec<(Pattern, Option<Instant>, VanishHandler)>>>,

    /// Change handlers (threshold, last_image, handler)
    /// 変化ハンドラー（閾値、最終画像、ハンドラー）
    change_handlers: Arc<Mutex<Vec<(f64, Option<DynamicImage>, ChangeHandler)>>>,
}

impl Observer {
    /// Create a new observer for the specified region
    /// 指定された領域の新しいオブザーバーを作成
    ///
    /// # Arguments / 引数
    ///
    /// * `region` - Screen region to monitor / 監視する画面領域
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::{Observer, Region};
    ///
    /// let region = Region::new(0, 0, 800, 600);
    /// let observer = Observer::new(region);
    /// ```
    pub fn new(region: Region) -> Self {
        Self {
            region,
            running: Arc::new(AtomicBool::new(false)),
            interval_ms: DEFAULT_OBSERVATION_INTERVAL_MS,
            matcher: ImageMatcher::new(),
            appear_handlers: Arc::new(Mutex::new(Vec::new())),
            vanish_handlers: Arc::new(Mutex::new(Vec::new())),
            change_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set the observation interval
    /// 監視間隔を設定
    ///
    /// Controls how frequently the screen is checked for changes.
    /// Lower values provide faster detection but use more CPU.
    ///
    /// 画面変化のチェック頻度を制御します。
    /// 低い値は高速検出を提供しますが、より多くのCPUを使用します。
    ///
    /// # Arguments / 引数
    ///
    /// * `interval_ms` - Observation interval in milliseconds / 監視間隔（ミリ秒）
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::{Observer, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 100, 100));
    /// observer.set_interval(200); // Check every 200ms
    /// ```
    pub fn set_interval(&mut self, interval_ms: u64) {
        self.interval_ms = interval_ms.max(10); // Minimum 10ms
    }

    /// Set minimum similarity for pattern matching
    /// パターンマッチングの最小類似度を設定
    ///
    /// # Arguments / 引数
    ///
    /// * `similarity` - Similarity threshold (0.0 - 1.0) / 類似度閾値（0.0 - 1.0）
    pub fn set_min_similarity(&mut self, similarity: f64) {
        self.matcher = self.matcher.with_min_similarity(similarity);
    }

    /// Register a callback for when a pattern appears
    /// パターンが出現した時のコールバックを登録
    ///
    /// The callback is invoked when the pattern is found in the observed region.
    /// Multiple handlers can be registered for different patterns.
    ///
    /// コールバックは監視領域でパターンが見つかった時に呼び出されます。
    /// 異なるパターンに対して複数のハンドラーを登録できます。
    ///
    /// # Arguments / 引数
    ///
    /// * `pattern` - Pattern to watch for / 監視するパターン
    /// * `callback` - Handler invoked when pattern appears / パターン出現時に呼び出されるハンドラー
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Pattern, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 800, 600));
    /// let pattern = Pattern::from_file("button.png")?;
    ///
    /// observer.on_appear(pattern, |m| {
    ///     println!("Button found at ({}, {})", m.get_x(), m.get_y());
    /// });
    /// ```
    pub fn on_appear<F>(&mut self, pattern: Pattern, callback: F)
    where
        F: Fn(&Match) + Send + 'static,
    {
        let mut handlers = self.appear_handlers.lock().unwrap();
        handlers.push((pattern, Box::new(callback)));
    }

    /// Register a callback for when a pattern vanishes
    /// パターンが消失した時のコールバックを登録
    ///
    /// The callback is invoked when a previously visible pattern is no longer found.
    /// The pattern must be found at least once before vanish detection works.
    ///
    /// コールバックは以前表示されていたパターンが見つからなくなった時に呼び出されます。
    /// 消失検出が機能するには、パターンが少なくとも一度は見つかっている必要があります。
    ///
    /// # Arguments / 引数
    ///
    /// * `pattern` - Pattern to watch for vanishing / 消失を監視するパターン
    /// * `callback` - Handler invoked when pattern vanishes / パターン消失時に呼び出されるハンドラー
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Pattern, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 800, 600));
    /// let pattern = Pattern::from_file("popup.png")?;
    ///
    /// observer.on_vanish(pattern, || {
    ///     println!("Popup disappeared!");
    /// });
    /// ```
    pub fn on_vanish<F>(&mut self, pattern: Pattern, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        let mut handlers = self.vanish_handlers.lock().unwrap();
        handlers.push((pattern, None, Box::new(callback)));
    }

    /// Register a callback for visual changes in the region
    /// 領域の視覚的変化のコールバックを登録
    ///
    /// The callback is invoked when the region's visual content changes by more than
    /// the specified threshold. Uses normalized cross-correlation for change detection.
    ///
    /// コールバックは領域の視覚的内容が指定された閾値以上変化した時に呼び出されます。
    /// 変化検出には正規化相互相関を使用します。
    ///
    /// # Arguments / 引数
    ///
    /// * `threshold` - Change threshold (0.0 - 1.0), higher = less sensitive / 変化閾値、高いほど感度が低い
    /// * `callback` - Handler invoked with change amount (0.0 - 1.0) / 変化量と共に呼び出されるハンドラー
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 800, 600));
    ///
    /// observer.on_change(0.1, |change_amount| {
    ///     println!("Region changed by {:.1}%", change_amount * 100.0);
    /// });
    /// ```
    pub fn on_change<F>(&mut self, threshold: f64, callback: F)
    where
        F: Fn(f64) + Send + 'static,
    {
        let threshold = threshold.clamp(0.0, 1.0);
        let mut handlers = self.change_handlers.lock().unwrap();
        handlers.push((threshold, None, Box::new(callback)));
    }

    /// Start observing with a timeout (blocking)
    /// タイムアウト付きで監視開始（ブロッキング）
    ///
    /// Observes the region until the timeout expires or stop() is called.
    /// This method blocks the current thread.
    ///
    /// タイムアウトが切れるかstop()が呼ばれるまで領域を監視します。
    /// このメソッドは現在のスレッドをブロックします。
    ///
    /// # Arguments / 引数
    ///
    /// * `timeout_secs` - Maximum observation time in seconds (0.0 = infinite) / 最大監視時間（秒）、0.0は無限
    ///
    /// # Returns / 戻り値
    ///
    /// Ok(()) when observation completes normally or is stopped.
    /// 監視が正常に完了または停止された時にOk(())を返します。
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Pattern, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 800, 600));
    /// let pattern = Pattern::from_file("button.png")?;
    ///
    /// observer.on_appear(pattern, |m| {
    ///     println!("Found!");
    /// });
    ///
    /// // Observe for 10 seconds
    /// // 10秒間監視
    /// observer.observe(10.0)?;
    /// ```
    pub fn observe(&self, timeout_secs: f64) -> Result<()> {
        self.running.store(true, Ordering::SeqCst);
        let start = Instant::now();
        let timeout = if timeout_secs > 0.0 {
            Some(Duration::from_secs_f64(timeout_secs))
        } else {
            None
        };

        log::info!(
            "Observer started for region ({}, {}, {}, {}), timeout: {}s",
            self.region.x,
            self.region.y,
            self.region.width,
            self.region.height,
            timeout_secs
        );

        while self.running.load(Ordering::SeqCst) {
            // Check timeout
            // タイムアウトチェック
            if let Some(timeout) = timeout {
                if start.elapsed() >= timeout {
                    log::debug!("Observer timeout reached");
                    break;
                }
            }

            // Process one observation cycle
            // 1監視サイクルを処理
            self.process_observation()?;

            // Sleep until next observation
            // 次の監視まで待機
            thread::sleep(Duration::from_millis(self.interval_ms));
        }

        self.running.store(false, Ordering::SeqCst);
        log::info!("Observer stopped");
        Ok(())
    }

    /// Start observing in a background thread
    /// バックグラウンドスレッドで監視開始
    ///
    /// Spawns a new thread that continuously observes the region until stop() is called.
    /// Returns a JoinHandle that can be used to wait for the thread to complete.
    ///
    /// stop()が呼ばれるまで領域を継続的に監視する新しいスレッドを生成します。
    /// スレッドの完了を待つために使用できるJoinHandleを返します。
    ///
    /// # Returns / 戻り値
    ///
    /// JoinHandle for the background thread
    /// バックグラウンドスレッドのJoinHandle
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Pattern, Region};
    ///
    /// let mut observer = Observer::new(Region::new(0, 0, 800, 600));
    /// let pattern = Pattern::from_file("button.png")?;
    ///
    /// observer.on_appear(pattern, |m| {
    ///     println!("Found!");
    /// });
    ///
    /// // Start in background
    /// // バックグラウンドで開始
    /// let handle = observer.observe_in_background();
    ///
    /// // Do other work...
    /// // 他の作業...
    ///
    /// // Stop and wait for completion
    /// // 停止して完了を待つ
    /// observer.stop();
    /// handle.join().unwrap()?;
    /// ```
    pub fn observe_in_background(&self) -> JoinHandle<Result<()>> {
        let running = Arc::clone(&self.running);
        let interval_ms = self.interval_ms;
        let region = self.region;
        let matcher = self.matcher.clone();
        let appear_handlers = Arc::clone(&self.appear_handlers);
        let vanish_handlers = Arc::clone(&self.vanish_handlers);
        let change_handlers = Arc::clone(&self.change_handlers);

        running.store(true, Ordering::SeqCst);

        thread::spawn(move || {
            log::info!(
                "Observer background thread started for region ({}, {}, {}, {})",
                region.x,
                region.y,
                region.width,
                region.height
            );

            while running.load(Ordering::SeqCst) {
                // Capture screen
                // 画面をキャプチャ
                let screen = Screen::primary();
                let screenshot = match screen.capture_region(&region) {
                    Ok(img) => img,
                    Err(e) => {
                        log::error!("Failed to capture screen: {}", e);
                        thread::sleep(Duration::from_millis(interval_ms));
                        continue;
                    }
                };

                // Process appearance handlers
                // 出現ハンドラーを処理
                if let Ok(handlers) = appear_handlers.lock() {
                    for (pattern, callback) in handlers.iter() {
                        if let Ok(Some(m)) = matcher.find(&screenshot, pattern) {
                            callback(&m);
                        }
                    }
                }

                // Process vanish handlers
                // 消失ハンドラーを処理
                if let Ok(mut handlers) = vanish_handlers.lock() {
                    for (pattern, last_seen, callback) in handlers.iter_mut() {
                        match matcher.find(&screenshot, pattern) {
                            Ok(Some(_)) => {
                                // Pattern found, update last_seen
                                // パターンが見つかった、last_seenを更新
                                *last_seen = Some(Instant::now());
                            }
                            Ok(None) => {
                                // Pattern not found
                                // パターンが見つからない
                                if last_seen.is_some() {
                                    // Pattern vanished! Trigger callback
                                    // パターンが消失！コールバックを起動
                                    callback();
                                    *last_seen = None;
                                }
                            }
                            Err(e) => {
                                log::warn!("Vanish detection error: {}", e);
                            }
                        }
                    }
                }

                // Process change handlers
                // 変化ハンドラーを処理
                if let Ok(mut handlers) = change_handlers.lock() {
                    for (threshold, last_image, callback) in handlers.iter_mut() {
                        if let Some(ref last) = last_image {
                            // Calculate change amount
                            // 変化量を計算
                            let change = calculate_image_difference(last, &screenshot);
                            if change >= *threshold {
                                callback(change);
                                *last_image = Some(screenshot.clone());
                            }
                        } else {
                            // First capture, store baseline
                            // 最初のキャプチャ、ベースラインを保存
                            *last_image = Some(screenshot.clone());
                        }
                    }
                }

                thread::sleep(Duration::from_millis(interval_ms));
            }

            running.store(false, Ordering::SeqCst);
            log::info!("Observer background thread stopped");
            Ok(())
        })
    }

    /// Stop observation
    /// 監視を停止
    ///
    /// Signals the observer to stop. For background observations, call join() on the
    /// returned JoinHandle to wait for the thread to complete.
    ///
    /// オブザーバーに停止を通知します。バックグラウンド監視の場合、
    /// 返されたJoinHandleでjoin()を呼んでスレッドの完了を待ってください。
    ///
    /// # Example / 使用例
    ///
    /// ```ignore
    /// use sikulid::{Observer, Region};
    ///
    /// let observer = Observer::new(Region::new(0, 0, 100, 100));
    /// let handle = observer.observe_in_background();
    ///
    /// // ... later ...
    /// observer.stop();
    /// handle.join().unwrap()?;
    /// ```
    pub fn stop(&self) {
        log::debug!("Observer stop requested");
        self.running.store(false, Ordering::SeqCst);
    }

    /// Check if the observer is currently running
    /// オブザーバーが現在実行中か確認
    ///
    /// # Returns / 戻り値
    ///
    /// true if observing, false otherwise
    /// 監視中ならtrue、それ以外はfalse
    ///
    /// # Example / 使用例
    ///
    /// ```
    /// use sikulid::{Observer, Region};
    ///
    /// let observer = Observer::new(Region::new(0, 0, 100, 100));
    /// assert!(!observer.is_running());
    ///
    /// let handle = observer.observe_in_background();
    /// std::thread::sleep(std::time::Duration::from_millis(10));
    /// assert!(observer.is_running());
    ///
    /// observer.stop();
    /// handle.join().unwrap().unwrap();
    /// assert!(!observer.is_running());
    /// ```
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Process one observation cycle (internal)
    /// 1監視サイクルを処理（内部用）
    fn process_observation(&self) -> Result<()> {
        // Capture screen
        // 画面をキャプチャ
        let screen = Screen::primary();
        let screenshot = screen.capture_region(&self.region)?;

        // Process appearance handlers
        // 出現ハンドラーを処理
        if let Ok(handlers) = self.appear_handlers.lock() {
            for (pattern, callback) in handlers.iter() {
                if let Ok(Some(m)) = self.matcher.find(&screenshot, pattern) {
                    callback(&m);
                }
            }
        }

        // Process vanish handlers
        // 消失ハンドラーを処理
        if let Ok(mut handlers) = self.vanish_handlers.lock() {
            for (pattern, last_seen, callback) in handlers.iter_mut() {
                match self.matcher.find(&screenshot, pattern) {
                    Ok(Some(_)) => {
                        *last_seen = Some(Instant::now());
                    }
                    Ok(None) => {
                        if last_seen.is_some() {
                            callback();
                            *last_seen = None;
                        }
                    }
                    Err(e) => {
                        log::warn!("Vanish detection error: {}", e);
                    }
                }
            }
        }

        // Process change handlers
        // 変化ハンドラーを処理
        if let Ok(mut handlers) = self.change_handlers.lock() {
            for (threshold, last_image, callback) in handlers.iter_mut() {
                if let Some(ref last) = last_image {
                    let change = calculate_image_difference(last, &screenshot);
                    if change >= *threshold {
                        callback(change);
                        *last_image = Some(screenshot.clone());
                    }
                } else {
                    *last_image = Some(screenshot.clone());
                }
            }
        }

        Ok(())
    }
}

/// Calculate normalized difference between two images
/// 2つの画像間の正規化された差分を計算
///
/// Returns a value between 0.0 (identical) and 1.0 (completely different).
/// Uses mean squared error normalized by image size.
///
/// 0.0（同一）から1.0（完全に異なる）の値を返します。
/// 画像サイズで正規化された平均二乗誤差を使用します。
fn calculate_image_difference(img1: &DynamicImage, img2: &DynamicImage) -> f64 {
    let gray1 = img1.to_luma8();
    let gray2 = img2.to_luma8();

    // Images must have same dimensions
    // 画像は同じ寸法である必要がある
    if gray1.dimensions() != gray2.dimensions() {
        return 1.0; // Consider completely different
    }

    let (width, height) = gray1.dimensions();
    let total_pixels = (width * height) as f64;

    // Calculate mean squared error
    // 平均二乗誤差を計算
    let sum_squared_diff: f64 = gray1
        .pixels()
        .zip(gray2.pixels())
        .map(|(p1, p2)| {
            let diff = p1[0] as f64 - p2[0] as f64;
            diff * diff
        })
        .sum();

    // Normalize to 0.0 - 1.0 range
    // 0.0 - 1.0 の範囲に正規化
    // Max possible difference is 255^2 per pixel
    // 最大可能差分はピクセルあたり255^2
    let mse = sum_squared_diff / total_pixels;
    let max_mse = 255.0 * 255.0;
    (mse / max_mse).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Arc;

    #[test]
    fn test_observer_new() {
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);
        assert_eq!(observer.region, region);
        assert!(!observer.is_running());
    }

    #[test]
    fn test_observer_set_interval() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        observer.set_interval(200);
        assert_eq!(observer.interval_ms, 200);

        // Test minimum clamp
        // 最小値クランプをテスト
        observer.set_interval(5);
        assert_eq!(observer.interval_ms, 10); // Clamped to minimum
    }

    #[test]
    fn test_observer_is_running() {
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);
        assert!(!observer.is_running());

        // Start background observation
        // バックグラウンド監視を開始
        let handle = observer.observe_in_background();
        thread::sleep(Duration::from_millis(50)); // Give thread time to start
        assert!(observer.is_running());

        // Stop observation
        // 監視を停止
        observer.stop();
        handle.join().unwrap().unwrap();
        assert!(!observer.is_running());
    }

    #[test]
    fn test_observer_stop() {
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);

        let handle = observer.observe_in_background();
        thread::sleep(Duration::from_millis(50));
        assert!(observer.is_running());

        observer.stop();
        handle.join().unwrap().unwrap();
        assert!(!observer.is_running());
    }

    #[test]
    fn test_observer_observe_timeout() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);
        observer.set_interval(10); // Use short interval for timing test

        let start = Instant::now();
        observer.observe(0.1).unwrap(); // 100ms timeout
        let elapsed = start.elapsed();

        // Should complete after approximately 100ms
        // 約100ms後に完了するべき
        assert!(elapsed.as_millis() >= 90);
        assert!(elapsed.as_millis() < 300);
    }

    #[test]
    fn test_observer_on_appear_callback() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        // Create a counter to verify callback is registered
        // コールバックが登録されているか確認するためのカウンターを作成
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let pattern = Pattern::new(vec![1, 2, 3]);
        observer.on_appear(pattern, move |_m| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Verify handler was registered
        // ハンドラーが登録されたことを確認
        let handlers = observer.appear_handlers.lock().unwrap();
        assert_eq!(handlers.len(), 1);
    }

    #[test]
    fn test_observer_on_vanish_callback() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let pattern = Pattern::new(vec![1, 2, 3]);
        observer.on_vanish(pattern, move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Verify handler was registered
        // ハンドラーが登録されたことを確認
        let handlers = observer.vanish_handlers.lock().unwrap();
        assert_eq!(handlers.len(), 1);
    }

    #[test]
    fn test_observer_on_change_callback() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        observer.on_change(0.1, move |_change| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Verify handler was registered
        // ハンドラーが登録されたことを確認
        let handlers = observer.change_handlers.lock().unwrap();
        assert_eq!(handlers.len(), 1);
    }

    #[test]
    fn test_observer_multiple_handlers() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        // Register multiple appear handlers
        // 複数の出現ハンドラーを登録
        let pattern1 = Pattern::new(vec![1, 2, 3]);
        let pattern2 = Pattern::new(vec![4, 5, 6]);

        observer.on_appear(pattern1, |_| {});
        observer.on_appear(pattern2, |_| {});

        let handlers = observer.appear_handlers.lock().unwrap();
        assert_eq!(handlers.len(), 2);
    }

    #[test]
    fn test_calculate_image_difference_identical() {
        // Create two identical images
        // 2つの同一画像を作成
        use image::{GrayImage, Luma};

        let img = GrayImage::from_pixel(10, 10, Luma([128u8]));
        let img1 = DynamicImage::ImageLuma8(img.clone());
        let img2 = DynamicImage::ImageLuma8(img);

        let diff = calculate_image_difference(&img1, &img2);
        assert!(diff < 0.01, "Identical images should have ~0 difference");
    }

    #[test]
    fn test_calculate_image_difference_different() {
        // Create two different images
        // 2つの異なる画像を作成
        use image::{GrayImage, Luma};

        let img1 = DynamicImage::ImageLuma8(GrayImage::from_pixel(10, 10, Luma([0u8])));
        let img2 = DynamicImage::ImageLuma8(GrayImage::from_pixel(10, 10, Luma([255u8])));

        let diff = calculate_image_difference(&img1, &img2);
        assert!(
            diff > 0.9,
            "Completely different images should have high difference"
        );
    }

    #[test]
    fn test_calculate_image_difference_size_mismatch() {
        // Create images with different sizes
        // 異なるサイズの画像を作成
        use image::{GrayImage, Luma};

        let img1 = DynamicImage::ImageLuma8(GrayImage::from_pixel(10, 10, Luma([128u8])));
        let img2 = DynamicImage::ImageLuma8(GrayImage::from_pixel(20, 20, Luma([128u8])));

        let diff = calculate_image_difference(&img1, &img2);
        assert_eq!(diff, 1.0, "Different sized images should return 1.0");
    }

    #[test]
    fn test_observer_set_min_similarity() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        // Should not panic
        // パニックしないべき
        observer.set_min_similarity(0.9);
        observer.set_min_similarity(0.5);
    }

    #[test]
    fn test_observer_change_threshold_clamp() {
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        // Test threshold clamping
        // 閾値のクランプをテスト
        observer.on_change(-0.5, |_| {}); // Should be clamped to 0.0
        observer.on_change(1.5, |_| {}); // Should be clamped to 1.0

        let handlers = observer.change_handlers.lock().unwrap();
        assert_eq!(handlers.len(), 2);

        // Check that thresholds are clamped
        // 閾値がクランプされていることを確認
        assert_eq!(handlers[0].0, 0.0);
        assert_eq!(handlers[1].0, 1.0);
    }

    #[test]
    fn test_observer_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let region = Region::new(0, 0, 100, 100);
        let observer = Arc::new(Observer::new(region));

        // Clone for thread
        // スレッド用にクローン
        let observer_clone = Arc::clone(&observer);

        let handle = thread::spawn(move || {
            let handle = observer_clone.observe_in_background();
            thread::sleep(Duration::from_millis(100));
            observer_clone.stop();
            handle.join().unwrap()
        });

        handle.join().unwrap().unwrap();
    }

    // ========================================================================
    // Negative Tests / 異常系テスト
    // ========================================================================

    #[test]
    fn test_observer_stop_without_start() {
        // Stopping without starting should be safe
        // 開始せずに停止しても安全であるべき
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);
        observer.stop(); // Should not panic
        assert!(!observer.is_running());
    }

    #[test]
    fn test_observer_multiple_stops() {
        // Multiple stops should be safe
        // 複数回の停止は安全であるべき
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);

        let handle = observer.observe_in_background();
        thread::sleep(Duration::from_millis(50));

        observer.stop();
        observer.stop(); // Second stop should be safe
        observer.stop(); // Third stop should be safe

        handle.join().unwrap().unwrap();
        assert!(!observer.is_running());
    }

    #[test]
    fn test_observer_zero_timeout() {
        // Zero timeout means infinite observation
        // ゼロタイムアウトは無限監視を意味する
        let region = Region::new(0, 0, 100, 100);
        let observer = Observer::new(region);

        let handle = thread::spawn(move || {
            observer.observe(0.0) // Infinite timeout
        });

        thread::sleep(Duration::from_millis(100));

        // Observer should still be running
        // オブザーバーはまだ実行中のはず
        // (We can't directly check since we moved observer, but thread should still be alive)
        assert!(!handle.is_finished());

        // Note: This will leak the thread, but it's just for testing
        // 注意: これはスレッドをリークしますが、テスト用のみです
    }

    #[test]
    fn test_observer_very_short_interval() {
        // Very short interval should be clamped to minimum
        // 非常に短い間隔は最小値にクランプされるべき
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);

        observer.set_interval(1); // 1ms
        assert_eq!(observer.interval_ms, 10); // Should be clamped to 10ms minimum
    }

    #[test]
    fn test_observer_empty_handlers() {
        // Observer with no handlers should not crash
        // ハンドラーなしのオブザーバーはクラッシュしないべき
        let region = Region::new(0, 0, 100, 100);
        let mut observer = Observer::new(region);
        observer.set_interval(10); // Use short interval for timing test

        let start = Instant::now();
        observer.observe(0.05).unwrap(); // 50ms timeout
        let elapsed = start.elapsed();

        assert!(elapsed.as_millis() >= 40);
        // Allow more slack for CI/CD environments under load
        // CI/CD環境での負荷を考慮して余裕を持たせる
        assert!(elapsed.as_millis() < 500);
    }
}
