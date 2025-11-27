//! Template matching implementation
//! テンプレートマッチング実装
//!
//! Performance optimizations:
//! パフォーマンス最適化:
//!
//! - Parallel processing using rayon for multi-core utilization
//! - Pre-computed template statistics for faster NCC calculation
//!
//! - rayonによる並列処理でマルチコア活用
//! - NCC計算高速化のためのテンプレート統計事前計算

use crate::{Match, Pattern, Region, Result, SikulixError};
use image::{DynamicImage, GrayImage};
use rayon::prelude::*;
use std::time::{Duration, Instant};

/// Default timeout for wait operations in seconds
/// wait操作のデフォルトタイムアウト（秒）
pub const DEFAULT_WAIT_TIMEOUT: f64 = 3.0;

/// Default scan interval for wait operations in milliseconds
/// wait操作のデフォルトスキャン間隔（ミリ秒）
pub const DEFAULT_SCAN_INTERVAL_MS: u64 = 50;

/// Pre-computed template statistics for faster matching
/// マッチング高速化のための事前計算済みテンプレート統計
struct TemplateStats {
    gray: GrayImage,
    width: u32,
    height: u32,
    sum_t2: f64,
}

impl TemplateStats {
    fn new(template: &DynamicImage) -> Self {
        let gray = template.to_luma8();
        let (width, height) = gray.dimensions();
        let sum_t2: f64 = gray
            .pixels()
            .map(|p| {
                let t = p[0] as f64;
                t * t
            })
            .sum();
        Self {
            gray,
            width,
            height,
            sum_t2,
        }
    }
}

/// Image matcher for template matching
/// テンプレートマッチング用画像マッチャー
///
/// Provides find(), wait(), and exists() operations for image-based automation.
/// SikuliX-compatible API for legacy script support.
///
/// 画像ベース自動化のための find(), wait(), exists() 操作を提供します。
/// レガシースクリプトサポートのための SikuliX 互換 API。
///
/// # Example
/// 使用例
///
/// ```ignore
/// use sikulix_core::{ImageMatcher, Pattern, Screen};
///
/// let matcher = ImageMatcher::new().with_min_similarity(0.9);
/// let screen = Screen::primary();
/// let pattern = Pattern::from_file("button.png")?;
///
/// // Find immediately (no retry)
/// // 即時検索（リトライなし）
/// let result = matcher.find(&screen.capture()?, &pattern)?;
///
/// // Wait with timeout (retries until found or timeout)
/// // タイムアウト付き待機（見つかるかタイムアウトまでリトライ）
/// let result = matcher.wait(&screen, &pattern, 3.0)?;
///
/// // Check existence (returns Option, no error on not found)
/// // 存在確認（Option を返す、見つからなくてもエラーにならない）
/// if let Some(m) = matcher.exists(&screen, &pattern, 1.0)? {
///     println!("Found at {:?}", m.center());
/// }
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ImageMatcher {
    /// Minimum similarity threshold
    /// 最小類似度閾値
    min_similarity: f64,

    /// Scan interval for wait/exists operations in milliseconds
    /// wait/exists 操作のスキャン間隔（ミリ秒）
    scan_interval_ms: u64,
}

impl Default for ImageMatcher {
    fn default() -> Self {
        Self {
            min_similarity: 0.7,
            scan_interval_ms: DEFAULT_SCAN_INTERVAL_MS,
        }
    }
}

impl ImageMatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set minimum similarity threshold
    /// 最小類似度閾値を設定
    pub fn with_min_similarity(mut self, similarity: f64) -> Self {
        self.min_similarity = similarity.clamp(0.0, 1.0);
        self
    }

    /// Set scan interval for wait/exists operations
    /// wait/exists 操作のスキャン間隔を設定
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `interval_ms` - Scan interval in milliseconds
    ///   スキャン間隔（ミリ秒）
    pub fn with_scan_interval(mut self, interval_ms: u64) -> Self {
        self.scan_interval_ms = interval_ms.max(10); // Minimum 10ms
        self
    }

    /// Find a pattern in the screen image (parallelized)
    /// 画面画像内でパターンを検索（並列処理）
    ///
    /// Uses normalized cross-correlation for template matching
    /// テンプレートマッチングに正規化相互相関を使用
    pub fn find(&self, screen: &DynamicImage, pattern: &Pattern) -> Result<Option<Match>> {
        let template = super::load_image_from_bytes(&pattern.image_data)?;
        let template_stats = TemplateStats::new(&template);

        let screen_gray = screen.to_luma8();
        let (sw, sh) = screen_gray.dimensions();
        let (tw, th) = (template_stats.width, template_stats.height);

        if tw > sw || th > sh {
            return Ok(None);
        }

        let threshold = pattern.similarity.max(self.min_similarity);

        // Parallel sliding window template matching
        // 並列スライディングウィンドウテンプレートマッチング
        let search_height = sh - th + 1;
        let results: Vec<(f64, u32, u32)> = (0..search_height)
            .into_par_iter()
            .map(|y| {
                let mut row_best_score = 0.0f64;
                let mut row_best_x = 0u32;

                for x in 0..=(sw - tw) {
                    let score = calculate_ncc_with_stats(&screen_gray, &template_stats, x, y);
                    if score > row_best_score {
                        row_best_score = score;
                        row_best_x = x;
                    }
                }
                (row_best_score, row_best_x, y)
            })
            .collect();

        // Find global best from per-row results
        // 各行の結果からグローバル最良を検索
        let (best_score, best_x, best_y) = results
            .into_iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap_or((0.0, 0, 0));

        if best_score >= threshold {
            let region = Region::new(best_x as i32, best_y as i32, tw, th);
            Ok(Some(Match::new(region, best_score)))
        } else {
            Ok(None)
        }
    }

    /// Find all occurrences of a pattern (parallelized)
    /// パターンの全出現箇所を検索（並列処理）
    pub fn find_all(&self, screen: &DynamicImage, pattern: &Pattern) -> Result<Vec<Match>> {
        let template = super::load_image_from_bytes(&pattern.image_data)?;
        let template_stats = TemplateStats::new(&template);

        let screen_gray = screen.to_luma8();
        let (sw, sh) = screen_gray.dimensions();
        let (tw, th) = (template_stats.width, template_stats.height);

        if tw > sw || th > sh {
            return Ok(vec![]);
        }

        let threshold = pattern.similarity.max(self.min_similarity);

        // Parallel sliding window with threshold check
        // 閾値チェック付き並列スライディングウィンドウ
        let search_height = sh - th + 1;
        let matches: Vec<Match> = (0..search_height)
            .into_par_iter()
            .flat_map(|y| {
                let mut row_matches = Vec::new();
                for x in 0..=(sw - tw) {
                    let score = calculate_ncc_with_stats(&screen_gray, &template_stats, x, y);
                    if score >= threshold {
                        let region = Region::new(x as i32, y as i32, tw, th);
                        row_matches.push(Match::new(region, score));
                    }
                }
                row_matches
            })
            .collect();

        // Non-maximum suppression
        Ok(self.non_maximum_suppression(matches, tw, th))
    }

    /// Non-maximum suppression to remove overlapping matches
    /// 重複するマッチを除去する非最大値抑制
    ///
    /// Optimized to reduce memory allocations and improve cache performance.
    /// メモリ割り当てを削減し、キャッシュ性能を向上させるように最適化。
    fn non_maximum_suppression(
        &self,
        mut matches: Vec<Match>,
        _template_width: u32,
        _template_height: u32,
    ) -> Vec<Match> {
        // Early return for small inputs
        // 小さい入力の場合は早期リターン
        if matches.len() <= 1 {
            return matches;
        }

        // Sort by score descending (unstable is faster)
        // スコア降順でソート（不安定ソートの方が高速）
        matches.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Pre-allocate suppression flags
        // 抑制フラグを事前割り当て
        let mut suppressed = vec![false; matches.len()];

        for i in 0..matches.len() {
            if suppressed[i] {
                continue;
            }

            // Move instead of clone to avoid allocation
            // クローンではなく移動することで割り当てを回避
            let current_region = &matches[i].region;

            // Suppress overlapping matches
            // Early termination if most matches are suppressed
            // 重複するマッチを抑制
            // ほとんどのマッチが抑制されている場合は早期終了
            for j in (i + 1)..matches.len() {
                if suppressed[j] {
                    continue;
                }

                let overlap = calculate_overlap_fast(current_region, &matches[j].region);

                // Suppress if overlap > 50%
                if overlap > 0.5 {
                    suppressed[j] = true;
                }
            }
        }

        // Collect non-suppressed matches by moving
        // 抑制されていないマッチを移動して収集
        matches
            .into_iter()
            .enumerate()
            .filter_map(|(i, m)| if !suppressed[i] { Some(m) } else { None })
            .collect()
    }

    /// Wait for a pattern to appear on screen with timeout
    /// タイムアウト付きでパターンが画面に表示されるのを待機
    ///
    /// Repeatedly captures the screen and searches for the pattern until found or timeout.
    /// Returns FindFailed error if the pattern is not found within the timeout period.
    ///
    /// 画面をキャプチャしてパターンを繰り返し検索し、見つかるかタイムアウトになるまで待機します。
    /// タイムアウト時間内にパターンが見つからない場合は FindFailed エラーを返します。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `screen` - Screen to capture from
    ///   キャプチャ元のスクリーン
    /// * `pattern` - Pattern to search for
    ///   検索するパターン
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///   最大待機時間（秒）
    ///
    /// # Returns
    /// 戻り値
    ///
    /// - `Ok(Match)` - Pattern was found
    ///   パターンが見つかった場合
    /// - `Err(FindFailed)` - Pattern not found within timeout
    ///   タイムアウト内にパターンが見つからなかった場合
    ///
    /// # Example
    /// 使用例
    ///
    /// ```ignore
    /// let matcher = ImageMatcher::new();
    /// let screen = Screen::primary();
    /// let pattern = Pattern::from_file("button.png")?;
    ///
    /// // Wait up to 5 seconds for pattern
    /// // 最大5秒間パターンを待機
    /// match matcher.wait(&screen, &pattern, 5.0) {
    ///     Ok(m) => println!("Found at {:?}", m.center()),
    ///     Err(e) => println!("Not found: {}", e),
    /// }
    /// ```
    pub fn wait(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<Match> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Try to find pattern
            // パターンを検索
            if let Some(m) = self.find(&screen_image, pattern)? {
                return Ok(m);
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Err(SikulixError::FindFailed {
                    pattern_name: format!("Pattern({}bytes)", pattern.image_data.len()),
                    timeout_secs,
                });
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for a pattern with default timeout
    /// デフォルトタイムアウトでパターンを待機
    ///
    /// Same as `wait()` but uses DEFAULT_WAIT_TIMEOUT (3.0 seconds).
    /// `wait()` と同じですが、DEFAULT_WAIT_TIMEOUT（3.0秒）を使用します。
    pub fn wait_default(&self, screen: &crate::screen::Screen, pattern: &Pattern) -> Result<Match> {
        self.wait(screen, pattern, DEFAULT_WAIT_TIMEOUT)
    }

    /// Check if a pattern exists on screen with optional timeout
    /// オプションのタイムアウト付きでパターンが画面上に存在するか確認
    ///
    /// Similar to wait() but returns Option instead of raising FindFailed error.
    /// This is the non-throwing variant for conditional checks.
    ///
    /// wait() と似ていますが、FindFailed エラーを発生させる代わりに Option を返します。
    /// これは条件分岐チェック用の例外を発生させないバリアントです。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `screen` - Screen to capture from
    ///   キャプチャ元のスクリーン
    /// * `pattern` - Pattern to search for
    ///   検索するパターン
    /// * `timeout_secs` - Maximum time to wait in seconds (use 0.0 for immediate check)
    ///   最大待機時間（秒）（即時チェックには 0.0 を使用）
    ///
    /// # Returns
    /// 戻り値
    ///
    /// - `Ok(Some(Match))` - Pattern was found
    ///   パターンが見つかった場合
    /// - `Ok(None)` - Pattern not found within timeout
    ///   タイムアウト内にパターンが見つからなかった場合
    /// - `Err(_)` - Screen capture or other error occurred
    ///   スクリーンキャプチャまたはその他のエラーが発生した場合
    ///
    /// # Example
    /// 使用例
    ///
    /// ```ignore
    /// let matcher = ImageMatcher::new();
    /// let screen = Screen::primary();
    /// let pattern = Pattern::from_file("button.png")?;
    ///
    /// // Check if pattern exists (immediate)
    /// // パターンが存在するか確認（即時）
    /// if let Some(m) = matcher.exists(&screen, &pattern, 0.0)? {
    ///     println!("Button found at {:?}", m.center());
    /// } else {
    ///     println!("Button not found");
    /// }
    ///
    /// // Check with 2 second timeout
    /// // 2秒のタイムアウトで確認
    /// if matcher.exists(&screen, &pattern, 2.0)?.is_some() {
    ///     // Do something when pattern appears
    ///     // パターンが表示された時に何かを実行
    /// }
    /// ```
    pub fn exists(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<Option<Match>> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Try to find pattern
            // パターンを検索
            if let Some(m) = self.find(&screen_image, pattern)? {
                return Ok(Some(m));
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Ok(None);
            }

            // Don't sleep on last iteration if timeout is 0
            // タイムアウトが0の場合、最後のイテレーションではスリープしない
            if timeout.is_zero() {
                return Ok(None);
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Check if pattern exists immediately (no waiting)
    /// パターンが即時存在するか確認（待機なし）
    ///
    /// Convenience method that calls exists() with 0 timeout.
    /// 0タイムアウトで exists() を呼び出す便利メソッドです。
    pub fn exists_now(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
    ) -> Result<Option<Match>> {
        self.exists(screen, pattern, 0.0)
    }

    /// Find pattern within a specific region
    /// 特定の領域内でパターンを検索
    ///
    /// Captures only the specified region and searches within it.
    /// Useful for optimizing search in known areas.
    ///
    /// 指定された領域のみをキャプチャし、その中で検索します。
    /// 既知のエリア内での検索を最適化するのに便利です。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `screen` - Screen to capture from
    ///   キャプチャ元のスクリーン
    /// * `region` - Region to search within
    ///   検索する領域
    /// * `pattern` - Pattern to search for
    ///   検索するパターン
    ///
    /// # Returns
    /// 戻り値
    ///
    /// Match result with coordinates relative to the full screen (not the region).
    /// 全画面を基準とした座標でのマッチ結果（領域基準ではない）。
    pub fn find_in_region(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        pattern: &Pattern,
    ) -> Result<Option<Match>> {
        // Capture the region
        // 領域をキャプチャ
        let region_image = screen.capture_region(region)?;

        // Find pattern in region
        // 領域内でパターンを検索
        if let Some(mut m) = self.find(&region_image, pattern)? {
            // Adjust coordinates to full screen
            // 座標を全画面基準に調整
            m.region.x += region.x;
            m.region.y += region.y;
            Ok(Some(m))
        } else {
            Ok(None)
        }
    }

    /// Wait for pattern within a specific region
    /// 特定の領域内でパターンを待機
    pub fn wait_in_region(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<Match> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            if let Some(m) = self.find_in_region(screen, region, pattern)? {
                return Ok(m);
            }

            if start.elapsed() >= timeout {
                return Err(SikulixError::FindFailed {
                    pattern_name: format!("Pattern({}bytes)", pattern.image_data.len()),
                    timeout_secs,
                });
            }

            std::thread::sleep(scan_interval);
        }
    }

    /// Check if pattern exists within a specific region
    /// 特定の領域内でパターンが存在するか確認
    pub fn exists_in_region(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<Option<Match>> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            if let Some(m) = self.find_in_region(screen, region, pattern)? {
                return Ok(Some(m));
            }

            if start.elapsed() >= timeout {
                return Ok(None);
            }

            if timeout.is_zero() {
                return Ok(None);
            }

            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for a pattern to vanish from screen
    /// パターンが画面から消えるのを待機
    ///
    /// Repeatedly checks if the pattern exists and returns when it's no longer found.
    /// Returns true if pattern vanished, false if timeout reached while pattern still visible.
    ///
    /// パターンが存在するかを繰り返しチェックし、見つからなくなったら戻ります。
    /// パターンが消えた場合はtrueを、パターンがまだ見える状態でタイムアウトした場合はfalseを返します。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `screen` - Screen to capture from
    ///   キャプチャ元のスクリーン
    /// * `pattern` - Pattern to wait for vanishing
    ///   消えるのを待つパターン
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///   最大待機時間（秒）
    ///
    /// # Returns
    /// 戻り値
    ///
    /// - `Ok(true)` - Pattern vanished within timeout
    ///   タイムアウト内にパターンが消えた場合
    /// - `Ok(false)` - Timeout reached, pattern still visible
    ///   タイムアウトに達し、パターンがまだ見える場合
    /// - `Err(_)` - Screen capture or other error occurred
    ///   スクリーンキャプチャまたはその他のエラーが発生した場合
    pub fn wait_vanish(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<bool> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Check if pattern is NOT found
            // パターンが見つからないかチェック
            if self.find(&screen_image, pattern)?.is_none() {
                return Ok(true); // Vanished
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Ok(false); // Still visible
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for a pattern to vanish with default timeout
    /// デフォルトタイムアウトでパターンが消えるのを待機
    pub fn wait_vanish_default(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
    ) -> Result<bool> {
        self.wait_vanish(screen, pattern, DEFAULT_WAIT_TIMEOUT)
    }

    /// Wait for pattern to vanish within a specific region
    /// 特定の領域内でパターンが消えるのを待機
    pub fn wait_vanish_in_region(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        pattern: &Pattern,
        timeout_secs: f64,
    ) -> Result<bool> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            if self.find_in_region(screen, region, pattern)?.is_none() {
                return Ok(true);
            }

            if start.elapsed() >= timeout {
                return Ok(false);
            }

            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for visual change in a region
    /// 領域内の視覚的変化を待機
    ///
    /// Captures the region, then repeatedly checks for pixel differences.
    /// Returns when significant change is detected or timeout occurs.
    ///
    /// 領域をキャプチャし、ピクセルの差分を繰り返しチェックします。
    /// 大きな変化が検出されるかタイムアウトになると戻ります。
    ///
    /// # Arguments
    /// 引数
    ///
    /// * `screen` - Screen to capture from
    ///   キャプチャ元のスクリーン
    /// * `region` - Region to monitor for changes
    ///   変化を監視する領域
    /// * `timeout_secs` - Maximum time to wait in seconds
    ///   最大待機時間（秒）
    /// * `min_change_percent` - Minimum percentage of changed pixels to detect (0.0-1.0, e.g., 0.05 = 5%)
    ///   検出する変化ピクセルの最小割合（0.0-1.0、例: 0.05 = 5%）
    ///
    /// # Returns
    /// 戻り値
    ///
    /// - `Ok(true)` - Significant change was detected within timeout
    ///   タイムアウト内に大きな変化が検出された場合
    /// - `Ok(false)` - Timeout reached without detecting change
    ///   変化が検出されずにタイムアウトに達した場合
    /// - `Err(_)` - Screen capture or other error occurred
    ///   スクリーンキャプチャまたはその他のエラーが発生した場合
    ///
    /// # Example
    /// 使用例
    ///
    /// ```ignore
    /// use sikulix_core::{ImageMatcher, Region, Screen};
    ///
    /// let matcher = ImageMatcher::new();
    /// let screen = Screen::primary();
    /// let region = Region::new(100, 100, 200, 200);
    ///
    /// // Wait up to 5 seconds for 5% of pixels to change
    /// // 最大5秒間、5%のピクセルが変化するのを待機
    /// if matcher.on_change(&screen, &region, 5.0, 0.05)? {
    ///     println!("Visual change detected!");
    /// } else {
    ///     println!("No change detected within timeout");
    /// }
    /// ```
    pub fn on_change(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        timeout_secs: f64,
        min_change_percent: f64,
    ) -> Result<bool> {
        // Capture initial reference image
        // 初期参照画像をキャプチャ
        let reference_image = screen.capture_region(region)?;
        let reference_gray = reference_image.to_luma8();

        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Check timeout before waiting
            // 待機前にタイムアウトをチェック
            if start.elapsed() >= timeout {
                return Ok(false);
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);

            // Capture current image
            // 現在の画像をキャプチャ
            let current_image = screen.capture_region(region)?;
            let current_gray = current_image.to_luma8();

            // Calculate change percentage
            // 変化の割合を計算
            let change_percent = calculate_change_percent(&reference_gray, &current_gray);

            // Check if change exceeds threshold
            // 変化が閾値を超えているかチェック
            if change_percent >= min_change_percent {
                return Ok(true);
            }
        }
    }

    /// Wait for visual change in a region with default threshold (5%)
    /// デフォルト閾値（5%）で領域内の視覚的変化を待機
    ///
    /// Convenience method that calls on_change() with 5% threshold.
    /// 5%閾値でon_change()を呼び出す便利メソッドです。
    pub fn on_change_default(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        timeout_secs: f64,
    ) -> Result<bool> {
        self.on_change(screen, region, timeout_secs, 0.05)
    }

    /// Wait for a pattern to appear with cancellation support
    /// キャンセルサポート付きでパターンが表示されるのを待機
    ///
    /// Similar to wait() but can be cancelled via CancellationToken.
    /// wait()と似ていますが、CancellationTokenでキャンセル可能です。
    ///
    /// # Arguments / 引数
    ///
    /// * `screen` - Screen to capture from / キャプチャ元のスクリーン
    /// * `pattern` - Pattern to search for / 検索するパターン
    /// * `timeout_secs` - Maximum time to wait in seconds / 最大待機時間（秒）
    /// * `token` - Cancellation token / キャンセルトークン
    ///
    /// # Returns / 戻り値
    ///
    /// - `Ok(Match)` - Pattern was found / パターンが見つかった場合
    /// - `Err(Cancelled)` - Operation was cancelled / 操作がキャンセルされた場合
    /// - `Err(FindFailed)` - Pattern not found within timeout / タイムアウト内にパターンが見つからなかった場合
    pub fn wait_with_cancel(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
        token: &crate::timeout::CancellationToken,
    ) -> Result<Match> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Check cancellation first
            // 最初にキャンセルをチェック
            if token.is_cancelled() {
                return Err(SikulixError::Cancelled(
                    "Pattern search cancelled by user".to_string(),
                ));
            }

            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Try to find pattern
            // パターンを検索
            if let Some(m) = self.find(&screen_image, pattern)? {
                return Ok(m);
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Err(SikulixError::FindFailed {
                    pattern_name: format!("Pattern({}bytes)", pattern.image_data.len()),
                    timeout_secs,
                });
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Check if a pattern exists with cancellation support
    /// キャンセルサポート付きでパターンが存在するか確認
    ///
    /// Similar to exists() but can be cancelled via CancellationToken.
    /// exists()と似ていますが、CancellationTokenでキャンセル可能です。
    ///
    /// # Arguments / 引数
    ///
    /// * `screen` - Screen to capture from / キャプチャ元のスクリーン
    /// * `pattern` - Pattern to search for / 検索するパターン
    /// * `timeout_secs` - Maximum time to wait in seconds / 最大待機時間（秒）
    /// * `token` - Cancellation token / キャンセルトークン
    ///
    /// # Returns / 戻り値
    ///
    /// - `Ok(Some(Match))` - Pattern was found / パターンが見つかった場合
    /// - `Ok(None)` - Pattern not found within timeout / タイムアウト内にパターンが見つからなかった場合
    /// - `Err(Cancelled)` - Operation was cancelled / 操作がキャンセルされた場合
    pub fn exists_with_cancel(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
        token: &crate::timeout::CancellationToken,
    ) -> Result<Option<Match>> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Check cancellation first
            // 最初にキャンセルをチェック
            if token.is_cancelled() {
                return Err(SikulixError::Cancelled(
                    "Pattern search cancelled by user".to_string(),
                ));
            }

            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Try to find pattern
            // パターンを検索
            if let Some(m) = self.find(&screen_image, pattern)? {
                return Ok(Some(m));
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Ok(None);
            }

            // Don't sleep on last iteration if timeout is 0
            // タイムアウトが0の場合、最後のイテレーションではスリープしない
            if timeout.is_zero() {
                return Ok(None);
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for a pattern to vanish with cancellation support
    /// キャンセルサポート付きでパターンが消えるのを待機
    ///
    /// Similar to wait_vanish() but can be cancelled via CancellationToken.
    /// wait_vanish()と似ていますが、CancellationTokenでキャンセル可能です。
    pub fn wait_vanish_with_cancel(
        &self,
        screen: &crate::screen::Screen,
        pattern: &Pattern,
        timeout_secs: f64,
        token: &crate::timeout::CancellationToken,
    ) -> Result<bool> {
        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Check cancellation first
            // 最初にキャンセルをチェック
            if token.is_cancelled() {
                return Err(SikulixError::Cancelled(
                    "Wait vanish cancelled by user".to_string(),
                ));
            }

            // Capture screen
            // 画面をキャプチャ
            let screen_image = screen.capture()?;

            // Check if pattern is NOT found
            // パターンが見つからないかチェック
            if self.find(&screen_image, pattern)?.is_none() {
                return Ok(true); // Vanished
            }

            // Check timeout
            // タイムアウトチェック
            if start.elapsed() >= timeout {
                return Ok(false); // Still visible
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);
        }
    }

    /// Wait for visual change with cancellation support
    /// キャンセルサポート付きで視覚的変化を待機
    ///
    /// Similar to on_change() but can be cancelled via CancellationToken.
    /// on_change()と似ていますが、CancellationTokenでキャンセル可能です。
    pub fn on_change_with_cancel(
        &self,
        screen: &crate::screen::Screen,
        region: &Region,
        timeout_secs: f64,
        min_change_percent: f64,
        token: &crate::timeout::CancellationToken,
    ) -> Result<bool> {
        // Capture initial reference image
        // 初期参照画像をキャプチャ
        let reference_image = screen.capture_region(region)?;
        let reference_gray = reference_image.to_luma8();

        let timeout = Duration::from_secs_f64(timeout_secs.max(0.0));
        let scan_interval = Duration::from_millis(self.scan_interval_ms);
        let start = Instant::now();

        loop {
            // Check cancellation first
            // 最初にキャンセルをチェック
            if token.is_cancelled() {
                return Err(SikulixError::Cancelled(
                    "Change detection cancelled by user".to_string(),
                ));
            }

            // Check timeout before waiting
            // 待機前にタイムアウトをチェック
            if start.elapsed() >= timeout {
                return Ok(false);
            }

            // Sleep before next scan
            // 次のスキャンまでスリープ
            std::thread::sleep(scan_interval);

            // Capture current image
            // 現在の画像をキャプチャ
            let current_image = screen.capture_region(region)?;
            let current_gray = current_image.to_luma8();

            // Calculate change percentage
            // 変化の割合を計算
            let change_percent = calculate_change_percent(&reference_gray, &current_gray);

            // Check if change exceeds threshold
            // 変化が閾値を超えているかチェック
            if change_percent >= min_change_percent {
                return Ok(true);
            }
        }
    }
}

/// Calculate Normalized Cross-Correlation with pre-computed template stats
/// 事前計算済みテンプレート統計を使用した正規化相互相関の計算
///
/// Optimized version with improved memory access patterns for better auto-vectorization.
/// メモリアクセスパターンを改善し、自動ベクトル化を促進する最適化版。
fn calculate_ncc_with_stats(
    screen: &GrayImage,
    template: &TemplateStats,
    offset_x: u32,
    offset_y: u32,
) -> f64 {
    let (tw, th) = (template.width, template.height);

    // Bounds check to prevent panic
    // パニックを防ぐための境界チェック
    if offset_x + tw > screen.width() || offset_y + th > screen.height() {
        return 0.0;
    }

    let mut sum_st = 0.0f64;
    let mut sum_s2 = 0.0f64;

    // Row-major access pattern for better cache locality
    // キャッシュ局所性向上のための行優先アクセスパターン
    for ty in 0..th {
        let screen_row_offset = offset_y + ty;

        // Access pixels in contiguous memory for auto-vectorization
        // 自動ベクトル化のための連続メモリアクセス
        for tx in 0..tw {
            // Bounds are checked above
            // 上記で境界チェック済み
            let s = screen.get_pixel(offset_x + tx, screen_row_offset)[0] as f64;
            let t = template.gray.get_pixel(tx, ty)[0] as f64;

            sum_st += s * t;
            sum_s2 += s * s;
        }
    }

    let denominator = (sum_s2 * template.sum_t2).sqrt();
    if denominator < f64::EPSILON {
        0.0
    } else {
        sum_st / denominator
    }
}

/// Calculate overlap ratio between two regions (IoU) - fast inline version
/// 2つの領域間の重複率を計算（IoU）- 高速インライン版
#[inline(always)]
fn calculate_overlap_fast(a: &Region, b: &Region) -> f64 {
    // Early exit if regions don't overlap at all
    // 領域が全く重ならない場合は早期終了
    if a.x + a.width as i32 <= b.x || b.x + b.width as i32 <= a.x
        || a.y + a.height as i32 <= b.y || b.y + b.height as i32 <= a.y {
        return 0.0;
    }

    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width as i32).min(b.x + b.width as i32);
    let y2 = (a.y + a.height as i32).min(b.y + b.height as i32);

    let intersection = ((x2 - x1) * (y2 - y1)) as f64;
    let area_a = (a.width * a.height) as f64;
    let area_b = (b.width * b.height) as f64;
    let union = area_a + area_b - intersection;

    intersection / union
}

/// Calculate overlap ratio between two regions (IoU)
/// 2つの領域間の重複率を計算（IoU）
#[inline]
#[allow(dead_code)]
fn calculate_overlap(a: &Region, b: &Region) -> f64 {
    calculate_overlap_fast(a, b)
}

/// Calculate percentage of changed pixels between two grayscale images
/// 2つのグレースケール画像間の変化ピクセルの割合を計算
///
/// Compares images pixel by pixel and counts pixels with significant difference.
/// Returns the ratio of changed pixels to total pixels (0.0-1.0).
///
/// 画像をピクセル単位で比較し、大きな差のあるピクセルをカウントします。
/// 全ピクセルに対する変化したピクセルの割合を返します（0.0-1.0）。
///
/// # Arguments
/// 引数
///
/// * `img1` - First grayscale image
///   最初のグレースケール画像
/// * `img2` - Second grayscale image
///   2番目のグレースケール画像
///
/// # Returns
/// 戻り値
///
/// Percentage of pixels that changed (0.0 = no change, 1.0 = all pixels changed)
/// 変化したピクセルの割合（0.0 = 変化なし、1.0 = すべてのピクセルが変化）
///
/// # Note
/// 注記
///
/// Uses a threshold of 20 for pixel difference to filter out minor noise.
/// A pixel is considered changed if the absolute difference exceeds 20.
///
/// ピクセル差分の閾値として20を使用し、軽微なノイズをフィルタリングします。
/// 絶対差分が20を超える場合、ピクセルは変化したと見なされます。
fn calculate_change_percent(img1: &GrayImage, img2: &GrayImage) -> f64 {
    // Check if dimensions match
    // 画像サイズが一致するかチェック
    if img1.dimensions() != img2.dimensions() {
        return 1.0; // Consider as full change if dimensions differ
                    // サイズが異なる場合は完全な変化と見なす
    }

    let total_pixels = (img1.width() * img1.height()) as f64;
    if total_pixels < f64::EPSILON {
        return 0.0; // Avoid division by zero
                    // ゼロ除算を回避
    }

    // Count pixels with significant difference (threshold = 20)
    // 大きな差分があるピクセルをカウント（閾値 = 20）
    let changed_pixels = img1
        .pixels()
        .zip(img2.pixels())
        .filter(|(p1, p2)| {
            let diff = (p1[0] as i32 - p2[0] as i32).abs();
            diff > 20 // Threshold to ignore minor noise
                      // 軽微なノイズを無視するための閾値
        })
        .count();

    changed_pixels as f64 / total_pixels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_creation() {
        let matcher = ImageMatcher::new().with_min_similarity(0.8);
        assert!((matcher.min_similarity - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn test_matcher_default_values() {
        let matcher = ImageMatcher::default();
        assert!((matcher.min_similarity - 0.7).abs() < f64::EPSILON);
        assert_eq!(matcher.scan_interval_ms, DEFAULT_SCAN_INTERVAL_MS);
    }

    #[test]
    fn test_matcher_with_scan_interval() {
        let matcher = ImageMatcher::new().with_scan_interval(100);
        assert_eq!(matcher.scan_interval_ms, 100);
    }

    #[test]
    fn test_matcher_scan_interval_minimum() {
        // Scan interval should be at least 10ms
        let matcher = ImageMatcher::new().with_scan_interval(5);
        assert_eq!(matcher.scan_interval_ms, 10);
    }

    #[test]
    fn test_matcher_builder_chain() {
        let matcher = ImageMatcher::new()
            .with_min_similarity(0.95)
            .with_scan_interval(75);
        assert!((matcher.min_similarity - 0.95).abs() < f64::EPSILON);
        assert_eq!(matcher.scan_interval_ms, 75);
    }

    #[test]
    fn test_default_constants() {
        assert!((DEFAULT_WAIT_TIMEOUT - 3.0).abs() < f64::EPSILON);
        assert_eq!(DEFAULT_SCAN_INTERVAL_MS, 50);
    }

    #[test]
    fn test_calculate_overlap_no_overlap() {
        let a = Region::new(0, 0, 100, 100);
        let b = Region::new(200, 200, 100, 100);
        assert!((calculate_overlap(&a, &b) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_overlap_full_overlap() {
        let a = Region::new(0, 0, 100, 100);
        let b = Region::new(0, 0, 100, 100);
        assert!((calculate_overlap(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_overlap_partial() {
        let a = Region::new(0, 0, 100, 100);
        let b = Region::new(50, 50, 100, 100);
        // Intersection is 50x50 = 2500
        // Union is 100*100 + 100*100 - 2500 = 17500
        // IoU = 2500/17500 ≈ 0.143
        let overlap = calculate_overlap(&a, &b);
        assert!(overlap > 0.1 && overlap < 0.2);
    }

    #[test]
    fn test_template_stats_creation() {
        // Create a simple 10x10 image
        let img = DynamicImage::new_rgba8(10, 10);
        let stats = TemplateStats::new(&img);
        assert_eq!(stats.width, 10);
        assert_eq!(stats.height, 10);
    }

    // ========================================================================
    // onChange Tests / onChange テスト
    // ========================================================================

    #[test]
    fn test_calculate_change_percent_identical_images() {
        // Test that identical images have 0% change
        // 同一画像の変化率が0%であることをテスト
        use image::GrayImage;

        let img1 = GrayImage::from_raw(10, 10, vec![128u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(10, 10, vec![128u8; 100]).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            change < f64::EPSILON,
            "Identical images should have 0% change"
        );
    }

    #[test]
    fn test_calculate_change_percent_completely_different() {
        // Test that completely different images have high change percentage
        // 完全に異なる画像の変化率が高いことをテスト
        use image::GrayImage;

        let img1 = GrayImage::from_raw(10, 10, vec![0u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(10, 10, vec![255u8; 100]).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            (change - 1.0).abs() < f64::EPSILON,
            "Completely different images should have 100% change"
        );
    }

    #[test]
    fn test_calculate_change_percent_partial_change() {
        // Test partial change detection
        // 部分的な変化検出のテスト
        use image::GrayImage;

        // Create two images where half the pixels differ significantly
        // 半分のピクセルが大きく異なる2つの画像を作成
        let data1 = vec![0u8; 100];
        let mut data2 = vec![0u8; 100];
        for i in 0..50 {
            data2[i] = 255; // Change first 50 pixels
        }

        let img1 = GrayImage::from_raw(10, 10, data1).unwrap();
        let img2 = GrayImage::from_raw(10, 10, data2).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            (change - 0.5).abs() < 0.01,
            "Half changed pixels should result in ~50% change, got {}",
            change
        );
    }

    #[test]
    fn test_calculate_change_percent_minor_noise() {
        // Test that minor noise is filtered out (threshold = 20)
        // 軽微なノイズがフィルタリングされることをテスト（閾値 = 20）
        use image::GrayImage;

        // Create images with minor differences (below threshold)
        // 閾値未満の軽微な差分がある画像を作成
        let img1 = GrayImage::from_raw(10, 10, vec![128u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(10, 10, vec![138u8; 100]).unwrap(); // +10 difference

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            change < f64::EPSILON,
            "Minor noise (diff=10) should be filtered out, got {}",
            change
        );
    }

    #[test]
    fn test_calculate_change_percent_threshold_boundary() {
        // Test the threshold boundary (20)
        // 閾値境界（20）のテスト
        use image::GrayImage;

        // Difference of exactly 20 should NOT trigger change
        // ちょうど20の差分は変化とみなされない
        let img1 = GrayImage::from_raw(10, 10, vec![100u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(10, 10, vec![120u8; 100]).unwrap(); // +20 difference

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            change < f64::EPSILON,
            "Difference of exactly 20 should not trigger change"
        );

        // Difference of 21 should trigger change
        // 21の差分は変化とみなされる
        let img3 = GrayImage::from_raw(10, 10, vec![121u8; 100]).unwrap(); // +21 difference
        let change2 = calculate_change_percent(&img1, &img3);
        assert!(
            (change2 - 1.0).abs() < f64::EPSILON,
            "Difference of 21 should trigger change for all pixels"
        );
    }

    #[test]
    fn test_calculate_change_percent_different_dimensions() {
        // Test that different dimensions are treated as full change
        // 異なるサイズは完全な変化として扱われることをテスト
        use image::GrayImage;

        let img1 = GrayImage::from_raw(10, 10, vec![128u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(20, 20, vec![128u8; 400]).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            (change - 1.0).abs() < f64::EPSILON,
            "Different dimensions should result in 100% change"
        );
    }

    #[test]
    fn test_calculate_change_percent_empty_images() {
        // Test handling of empty images
        // 空画像の処理をテスト
        use image::GrayImage;

        let img1 = GrayImage::from_raw(0, 0, vec![]).unwrap();
        let img2 = GrayImage::from_raw(0, 0, vec![]).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            change < f64::EPSILON,
            "Empty images should have 0% change"
        );
    }

    #[test]
    fn test_calculate_change_percent_single_pixel_change() {
        // Test detection of a single pixel change
        // 1ピクセルだけの変化検出をテスト
        use image::GrayImage;

        let data1 = vec![128u8; 100];
        let mut data2 = vec![128u8; 100];
        data2[50] = 255; // Change one pixel significantly

        let img1 = GrayImage::from_raw(10, 10, data1).unwrap();
        let img2 = GrayImage::from_raw(10, 10, data2).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            (change - 0.01).abs() < 0.001,
            "Single pixel change in 100-pixel image should be ~1%, got {}",
            change
        );
    }

    #[test]
    fn test_calculate_change_percent_negative_difference() {
        // Test that negative differences are handled correctly (absolute value)
        // 負の差分が正しく処理されることをテスト（絶対値）
        use image::GrayImage;

        let img1 = GrayImage::from_raw(10, 10, vec![255u8; 100]).unwrap();
        let img2 = GrayImage::from_raw(10, 10, vec![0u8; 100]).unwrap();

        let change = calculate_change_percent(&img1, &img2);
        assert!(
            (change - 1.0).abs() < f64::EPSILON,
            "Negative difference should be treated as positive"
        );
    }
}
