# マルチモニター・DPI対応 設計書

## 1. 概要

### 1.1 目的
- マルチモニター環境でのSikuliX完全互換を実現
- DPI/スケーリング対応（論理ピクセル基準）
- グローバル座標系（プライマリモニター原点）での統一操作

### 1.2 確定仕様
| 項目 | 決定 |
|------|------|
| 座標系 | グローバル座標（プライマリモニター原点基準） |
| Screen(0) | プライマリモニター |
| ピクセル | 論理ピクセルがマスターデータ |
| 変換方向 | 論理→物理のみ（逆変換なし、丸め誤差回避） |
| マッチング | キャプチャを論理サイズにリサイズ、論理座標で結果返却 |
| capture_region | indexパラメータ削除、グローバル座標で統一 |
| 互換性 | SikuliX完全互換 |

---

## 2. データ構造設計

### 2.1 MonitorInfo 拡張

```rust
// core-rs/src/screen/windows.rs

/// Monitor information structure
/// モニター情報構造体
#[derive(Debug, Clone)]
pub struct MonitorInfo {
    /// Monitor index (0 = primary) / モニターインデックス（0 = プライマリ）
    pub index: u32,
    /// X position (logical pixels) / X座標（論理ピクセル）
    pub x: i32,
    /// Y position (logical pixels) / Y座標（論理ピクセル）
    pub y: i32,
    /// Width (logical pixels) / 幅（論理ピクセル）
    pub width: u32,
    /// Height (logical pixels) / 高さ（論理ピクセル）
    pub height: u32,
    /// Is primary monitor / プライマリモニターかどうか
    pub is_primary: bool,
    /// DPI scale factor (1.0 = 100%, 1.5 = 150%, 2.0 = 200%)
    /// DPIスケールファクター
    pub scale_factor: f64,
}
```

### 2.2 座標変換ユーティリティ

**設計原則**: 論理ピクセルをマスターデータとして保持し、OSへの操作時のみ物理ピクセルに変換する。
逆変換（物理→論理）は丸め誤差の原因となるため提供しない。

```rust
// core-rs/src/screen/coordinates.rs (新規ファイル)

/// Coordinate conversion utilities
/// 座標変換ユーティリティ
///
/// Design principle: Logical pixels are the master data.
/// Convert to physical only when needed for OS operations.
/// No physical-to-logical conversion to avoid rounding errors.
///
/// 設計原則: 論理ピクセルがマスターデータ。
/// OS操作時のみ物理ピクセルに変換。
/// 丸め誤差回避のため、物理→論理変換は提供しない。

/// Convert logical to physical pixels
/// 論理ピクセルを物理ピクセルに変換
pub fn logical_to_physical(logical: i32, scale_factor: f64) -> i32 {
    (logical as f64 * scale_factor).round() as i32
}

/// Convert logical Region to physical Region
/// 論理Regionを物理Regionに変換
pub fn region_to_physical(region: &Region, scale_factor: f64) -> Region {
    Region::new(
        logical_to_physical(region.x, scale_factor),
        logical_to_physical(region.y, scale_factor),
        (region.width as f64 * scale_factor).round() as u32,
        (region.height as f64 * scale_factor).round() as u32,
    )
}

/// Resize image from physical to logical dimensions
/// 物理サイズの画像を論理サイズにリサイズ
pub fn resize_to_logical(image: &DynamicImage, scale_factor: f64) -> DynamicImage {
    if (scale_factor - 1.0).abs() < 0.001 {
        return image.clone(); // No scaling needed
    }
    let logical_width = (image.width() as f64 / scale_factor).round() as u32;
    let logical_height = (image.height() as f64 / scale_factor).round() as u32;
    image.resize_exact(logical_width, logical_height, image::imageops::FilterType::Lanczos3)
}
```

### 2.3 データフロー

```
┌─────────────────────────────────────────────────────────────────┐
│                    データフロー図                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  [ユーザーAPI]                                                   │
│      │                                                           │
│      ▼                                                           │
│  ┌────────────────────────────────────────┐                     │
│  │  論理座標 (マスターデータ)              │                     │
│  │  - Region(x, y, width, height)         │                     │
│  │  - Point(x, y)                         │                     │
│  │  - すべてのAPI入出力                    │                     │
│  └────────────────────────────────────────┘                     │
│      │                         │                                 │
│      │ OS操作時               │ 画像マッチング時                │
│      ▼                         ▼                                 │
│  ┌──────────────┐     ┌──────────────────────────────┐         │
│  │ 物理座標変換 │     │ キャプチャ画像を論理サイズに │         │
│  │ click, move  │     │ リサイズしてマッチング       │         │
│  │ capture_rect │     │ 結果は論理座標で返却         │         │
│  └──────────────┘     └──────────────────────────────┘         │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. API設計

### 3.1 Screen API 更新

```rust
// core-rs/src/screen/mod.rs

impl Screen {
    /// Get all connected screens
    /// 接続されているすべてのスクリーンを取得
    pub fn all() -> Vec<Screen> {
        let count = Self::get_number_screens();
        (0..count).map(Screen::new).collect()
    }

    /// Get monitor info for this screen
    /// このスクリーンのモニター情報を取得
    pub fn get_monitor_info(&self) -> Option<MonitorInfo> {
        Self::get_monitor_info_impl(self.index)
    }

    /// Get DPI scale factor for this screen
    /// このスクリーンのDPIスケールファクターを取得
    pub fn get_scale_factor(&self) -> f64 {
        self.get_monitor_info()
            .map(|m| m.scale_factor)
            .unwrap_or(1.0)
    }
}
```

### 3.2 capture_region 修正

```rust
// Before (現状):
pub fn capture_region(_index: u32, region: &Region) -> Result<DynamicImage>

// After (修正後):
pub fn capture_region(region: &Region) -> Result<DynamicImage>
```

**影響範囲**:
- `core-rs/src/screen/mod.rs`: `capture_region_impl` 呼び出し
- `core-rs/src/screen/windows.rs`: 関数シグネチャ
- `core-rs/src/screen/macos.rs`: 関数シグネチャ
- `core-rs/src/screen/linux.rs`: 関数シグネチャ

---

## 4. Windows DPI取得実装

```rust
// core-rs/src/screen/windows.rs

use windows::Win32::UI::HiDpi::*;

/// Get DPI scale factor for a monitor
/// モニターのDPIスケールファクターを取得
#[cfg(target_os = "windows")]
fn get_monitor_dpi(hmonitor: HMONITOR) -> f64 {
    unsafe {
        let mut dpi_x: u32 = 96;
        let mut dpi_y: u32 = 96;

        // Try to get per-monitor DPI (Windows 8.1+)
        if GetDpiForMonitor(hmonitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_ok() {
            return dpi_x as f64 / 96.0;
        }

        // Fallback to system DPI
        let hdc = GetDC(HWND::default());
        if !hdc.is_invalid() {
            let dpi = GetDeviceCaps(hdc, LOGPIXELSX);
            ReleaseDC(HWND::default(), hdc);
            return dpi as f64 / 96.0;
        }

        1.0 // Default scale factor
    }
}
```

---

## 5. テスト設計

### 5.1 ユニットテスト

```rust
// core-rs/src/screen/tests/coordinates_tests.rs

#[cfg(test)]
mod tests {
    use super::*;

    // === 座標変換テスト (論理→物理のみ) ===

    #[test]
    fn test_logical_to_physical_100_percent() {
        assert_eq!(logical_to_physical(100, 1.0), 100);
        assert_eq!(logical_to_physical(0, 1.0), 0);
        assert_eq!(logical_to_physical(-100, 1.0), -100);
    }

    #[test]
    fn test_logical_to_physical_150_percent() {
        assert_eq!(logical_to_physical(100, 1.5), 150);
        assert_eq!(logical_to_physical(200, 1.5), 300);
        assert_eq!(logical_to_physical(-100, 1.5), -150);
    }

    #[test]
    fn test_logical_to_physical_200_percent() {
        assert_eq!(logical_to_physical(100, 2.0), 200);
        assert_eq!(logical_to_physical(50, 2.0), 100);
    }

    #[test]
    fn test_region_to_physical() {
        let region = Region::new(100, 200, 300, 400);
        let physical = region_to_physical(&region, 1.5);
        assert_eq!(physical.x, 150);
        assert_eq!(physical.y, 300);
        assert_eq!(physical.width, 450);
        assert_eq!(physical.height, 600);
    }

    #[test]
    fn test_negative_coordinates() {
        // 左側モニターのテスト（負座標の変換）
        let x = -1920;
        let physical = logical_to_physical(x, 1.0);
        assert_eq!(physical, -1920);

        // 150%スケーリング
        let physical_150 = logical_to_physical(x, 1.5);
        assert_eq!(physical_150, -2880);
    }

    // === 画像リサイズテスト ===

    #[test]
    fn test_resize_to_logical_no_scaling() {
        // scale_factor 1.0 の場合は画像サイズ変更なし
        let img = create_test_image(1920, 1080);
        let result = resize_to_logical(&img, 1.0);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }

    #[test]
    fn test_resize_to_logical_150_percent() {
        // 物理 3840x2160 → 論理 2560x1440 (150%)
        let img = create_test_image(3840, 2160);
        let result = resize_to_logical(&img, 1.5);
        assert_eq!(result.width(), 2560);
        assert_eq!(result.height(), 1440);
    }

    #[test]
    fn test_resize_to_logical_200_percent() {
        // 物理 3840x2160 → 論理 1920x1080 (200%)
        let img = create_test_image(3840, 2160);
        let result = resize_to_logical(&img, 2.0);
        assert_eq!(result.width(), 1920);
        assert_eq!(result.height(), 1080);
    }

    // === モニター情報テスト ===

    #[test]
    fn test_monitor_info_scale_factor_default() {
        let info = MonitorInfo {
            index: 0,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
            scale_factor: 1.0,
        };
        assert_eq!(info.scale_factor, 1.0);
    }

    #[test]
    fn test_monitor_info_with_scaling() {
        let info = MonitorInfo {
            index: 0,
            x: 0,
            y: 0,
            width: 2560,  // 論理: 2560 (4K @ 150%)
            height: 1440,
            is_primary: true,
            scale_factor: 1.5,
        };
        // 物理解像度は 3840x2160
        let physical_width = (info.width as f64 * info.scale_factor) as u32;
        assert_eq!(physical_width, 3840);
    }

    // === Screen API テスト ===

    #[test]
    fn test_screen_get_number_screens() {
        let count = Screen::get_number_screens();
        assert!(count >= 1, "At least one monitor should be connected");
    }

    #[test]
    fn test_screen_all() {
        let screens = Screen::all();
        let count = Screen::get_number_screens();
        assert_eq!(screens.len(), count as usize);
    }

    #[test]
    fn test_screen_primary_is_index_zero() {
        let primary = Screen::primary();
        assert_eq!(primary.index, 0);
    }

    // === ヘルパー関数 ===

    fn create_test_image(width: u32, height: u32) -> DynamicImage {
        DynamicImage::new_rgb8(width, height)
    }
}
```

### 5.2 統合テスト

```rust
// core-rs/tests/multimonitor_integration.rs

#[test]
#[ignore = "Requires multi-monitor setup"]
fn integration_test_multimonitor_capture() {
    let screens = Screen::all();

    for screen in &screens {
        let capture = screen.capture();
        assert!(capture.is_ok(), "Capture should succeed for screen {}", screen.index);

        if let Ok(img) = capture {
            assert!(img.width() > 0);
            assert!(img.height() > 0);
        }
    }
}

#[test]
#[ignore = "Requires actual monitor"]
fn integration_test_dpi_detection() {
    let screen = Screen::primary();
    let scale = screen.get_scale_factor();

    // スケールファクターは0.5〜4.0の範囲
    assert!(scale >= 0.5 && scale <= 4.0,
            "Scale factor {} is out of expected range", scale);
}
```

---

## 6. 実装フェーズ

### Phase 1: MonitorInfo拡張とDPI取得
1. `MonitorInfo` に `scale_factor` フィールド追加
2. Windows DPI取得API実装
3. ユニットテスト追加

### Phase 2: 座標変換
1. `coordinates.rs` モジュール作成
2. 変換関数実装
3. ユニットテスト追加

### Phase 3: capture_region修正
1. シグネチャ変更（index削除）
2. 各プラットフォーム対応
3. 既存コードの呼び出し修正

### Phase 4: Screen API更新
1. `Screen::all()` 実装
2. `Screen::get_monitor_info()` 実装
3. `Screen::get_scale_factor()` 実装

### Phase 5: 統合テスト
1. マルチモニター環境テスト
2. DPI検出テスト
3. SikuliX互換性テスト

---

## 7. 検証項目

### 7.1 自動テスト（cargo test）
- [ ] `logical_to_physical`: 100%, 150%, 200%スケーリング
- [ ] `logical_to_physical`: 負座標（左側モニター）
- [ ] `region_to_physical`: 領域全体の変換
- [ ] `resize_to_logical`: 画像リサイズ（100%, 150%, 200%）
- [ ] MonitorInfoのscale_factor取得
- [ ] Screen APIの基本動作（all, get_number_screens, primary）

### 7.2 統合テスト（マルチモニター環境）
- [ ] 各モニターでのキャプチャ成功
- [ ] DPI検出（0.5〜4.0範囲）
- [ ] 異なるDPIモニター間での画像マッチング

### 7.3 手動テスト（ユーザー確認）
- [ ] 異なるDPI設定でのスクリプト動作
- [ ] SikuliXスクリプトの互換性

---

## 8. 影響分析

### 変更ファイル
| ファイル | 変更内容 |
|---------|---------|
| `core-rs/src/screen/windows.rs` | MonitorInfo拡張、DPI取得、capture_region修正 |
| `core-rs/src/screen/macos.rs` | capture_region修正 |
| `core-rs/src/screen/linux.rs` | capture_region修正 |
| `core-rs/src/screen/mod.rs` | Screen API更新 |
| `core-rs/src/screen/coordinates.rs` | 新規: 座標変換 |

### 後方互換性
- `capture_region` のシグネチャ変更は破壊的変更
- 内部APIのため外部影響は限定的
- runtime-rs, ide-rs-tauri の呼び出し箇所を同時修正
