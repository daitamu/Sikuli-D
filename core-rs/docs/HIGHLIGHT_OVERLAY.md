# Highlight Overlay Implementation / ハイライトオーバーレイ実装

## Overview / 概要

The highlight overlay feature provides visual feedback by drawing colored borders around screen regions. This is essential for debugging and demonstrating GUI automation scripts.

ハイライトオーバーレイ機能は、画面領域の周りに色付きの境界線を描画することで視覚的なフィードバックを提供します。これは、GUI自動化スクリプトのデバッグとデモンストレーションに不可欠です。

## Implementation Status / 実装状況

| Platform | Status | Implementation Method |
|----------|--------|----------------------|
| Windows  | ✅ Complete | Layered window with WS_EX_LAYERED, WS_EX_TRANSPARENT |
| macOS    | 🚧 Stub | TODO: NSWindow with CALayer |
| Linux    | 🚧 Stub | TODO: X11 override-redirect window |

## Windows Implementation / Windows実装

### Architecture / アーキテクチャ

The Windows implementation uses the following Win32 APIs:

Windows実装は以下のWin32 APIを使用します：

1. **CreateWindowExW** - Creates a layered, topmost window
   レイヤード、最前面ウィンドウを作成

2. **WS_EX_LAYERED** - Enables alpha blending and transparency
   アルファブレンドと透明度を有効化

3. **WS_EX_TRANSPARENT** - Makes the window click-through
   ウィンドウをクリックスルー化

4. **WS_EX_TOPMOST** - Keeps the window on top of all others
   ウィンドウを常に最前面に保持

5. **WS_EX_TOOLWINDOW** - Hides the window from the taskbar
   タスクバーからウィンドウを非表示

6. **SetLayeredWindowAttributes** - Sets window alpha transparency
   ウィンドウのアルファ透明度を設定

7. **GDI DrawRectangle** - Draws the colored border
   色付きの境界線を描画

### Code Flow / コードフロー

```
show_highlight(region, config)
  ↓
RegisterClassW("SikuliDHighlightOverlay")
  ↓
CreateWindowExW with layered/transparent/topmost flags
  ↓
SetLayeredWindowAttributes(alpha=255)
  ↓
ShowWindow(SW_SHOWNOACTIVATE)
  ↓
GetDC + draw_border (CreatePen + Rectangle)
  ↓
ReleaseDC
  ↓
If duration > 0:
  spawn thread → sleep → DestroyWindow
```

### Key Features / 主な機能

#### 1. Click-Through Transparency / クリックスルー透明度

The overlay does not intercept mouse clicks or keyboard input. Users can interact with applications underneath the overlay normally.

オーバーレイはマウスクリックやキーボード入力を傍受しません。ユーザーはオーバーレイの下のアプリケーションと通常通りやり取りできます。

#### 2. Always on Top / 常に最前面

The highlight is displayed above all other windows, ensuring visibility even when other applications are active.

ハイライトは他のすべてのウィンドウの上に表示され、他のアプリケーションがアクティブでも可視性を確保します。

#### 3. No Taskbar Presence / タスクバーに表示されない

The overlay window is flagged as a tool window, so it doesn't appear in the taskbar or Alt+Tab switcher.

オーバーレイウィンドウはツールウィンドウとしてフラグが立てられるため、タスクバーやAlt+Tabスイッチャーに表示されません。

#### 4. Automatic Cleanup / 自動クリーンアップ

When a duration is specified, the overlay automatically destroys itself after the timeout without requiring manual intervention.

時間が指定されると、オーバーレイは手動介入なしでタイムアウト後に自動的に破棄されます。

#### 5. Multiple Simultaneous Overlays / 複数同時オーバーレイ

The implementation supports creating multiple overlays simultaneously, useful for highlighting several regions at once.

実装は複数のオーバーレイを同時に作成することをサポートし、複数の領域を一度にハイライトするのに便利です。

### Configuration Options / 設定オプション

```rust
pub struct HighlightConfig {
    pub color: (u8, u8, u8),      // RGB color (0-255 each)
    pub border_width: u32,         // Width in pixels (default: 3)
    pub duration_ms: u64,          // Duration in milliseconds (0 = manual close)
}
```

#### Default Values / デフォルト値

- Color: `(255, 0, 0)` - Red / 赤
- Border Width: `3` pixels / ピクセル
- Duration: `2000` ms (2 seconds) / ミリ秒（2秒）

## API Usage / API使用方法

### Basic Usage / 基本的な使用方法

```rust
use sikulix_core::{Region, Color};

let region = Region::new(100, 100, 300, 200);
let color = Color::rgb(255, 0, 0); // Red

// Show highlight for 2 seconds
sikulix_core::debug::highlight(&region, 2000, color)?;
```

### Custom Configuration / カスタム設定

```rust
use sikulix_core::debug::HighlightConfig;

let region = Region::new(100, 100, 300, 200);
let config = HighlightConfig::new()
    .with_color(0, 255, 0)        // Green
    .with_border_width(5)          // Thicker border
    .with_duration_ms(3000);       // 3 seconds

sikulix_core::debug::show_highlight_with_config(&region, &config)?;
```

### Highlighting Match Results / マッチ結果のハイライト

```rust
use sikulix_core::{ImageMatcher, Pattern, Screen};

let matcher = ImageMatcher::new();
let screen = Screen::primary().capture()?;
let pattern = Pattern::from_file("button.png")?;

if let Some(m) = matcher.find(&screen, &pattern)? {
    // Highlight the match for 2 seconds
    sikulix_core::debug::highlight_match(&m, 2000)?;
}
```

### Integration with Existing Highlight Module / 既存のHighlightモジュールとの統合

The existing `Highlight` struct in `src/highlight.rs` has been updated to use platform-specific overlays when available:

`src/highlight.rs`の既存の`Highlight`構造体は、利用可能な場合にプラットフォーム固有のオーバーレイを使用するように更新されました：

```rust
use sikulix_core::{Highlight, Region};

let region = Region::new(100, 100, 200, 150);
let highlight = Highlight::new(region);

// This will use platform-specific overlay on Windows
// Windowsではプラットフォーム固有のオーバーレイを使用します
highlight.show_for(2.0); // 2 seconds
```

## Technical Details / 技術詳細

### Window Procedure / ウィンドウプロシージャ

The overlay window has a minimal window procedure that handles:

オーバーレイウィンドウには以下を処理する最小限のウィンドウプロシージャがあります：

- **WM_PAINT**: Ignored (drawing is done once at creation)
  無視（描画は作成時に一度実行）

- **WM_DESTROY**: Clean up and exit
  クリーンアップと終了

- **WM_CLOSE**: Destroy the window
  ウィンドウを破棄

- **Default**: Passed to DefWindowProcW
  デフォルト：DefWindowProcWに渡す

### Drawing Process / 描画プロセス

1. Get device context with `GetDC`
   `GetDC`でデバイスコンテキストを取得

2. Create a pen with specified color and width using `CreatePen`
   `CreatePen`で指定された色と幅のペンを作成

3. Select null brush (transparent fill) with `GetStockObject(NULL_BRUSH)`
   `GetStockObject(NULL_BRUSH)`でヌルブラシを選択（透明塗りつぶし）

4. Draw rectangle border with `Rectangle`
   `Rectangle`で矩形境界線を描画

5. Restore old pen and brush
   古いペンとブラシを復元

6. Delete created pen
   作成したペンを削除

7. Release device context with `ReleaseDC`
   `ReleaseDC`でデバイスコンテキストを解放

### Thread Safety / スレッド安全性

The implementation spawns a separate thread for auto-closing overlays:

実装は、自動クローズオーバーレイ用に別のスレッドを生成します：

```rust
if config.duration_ms > 0 {
    let duration = Duration::from_millis(config.duration_ms);
    thread::spawn(move || {
        thread::sleep(duration);
        unsafe {
            DestroyWindow(hwnd);
        }
    });
}
```

This approach ensures the main thread is not blocked and can continue processing.

このアプローチにより、メインスレッドがブロックされず、処理を続行できます。

## Testing / テスト

### Unit Tests / ユニットテスト

```bash
cargo test --lib debug::highlight
```

Unit tests cover:
ユニットテストのカバー範囲：

- Configuration builder pattern
  設定ビルダーパターン
- Color conversion from Color struct
  Color構造体からの色変換
- API function signatures
  API関数シグネチャ

### Integration Tests / 統合テスト

```bash
# Requires GUI environment / GUI環境が必要
cargo test --test highlight_windows -- --ignored
```

Integration tests (marked with `#[ignore]`) require a GUI environment and test:

統合テスト（`#[ignore]`でマーク）はGUI環境を必要とし、以下をテストします：

- Actual window creation on Windows
  Windowsでの実際のウィンドウ作成
- Overlay visibility
  オーバーレイの可視性
- Auto-close behavior
  自動クローズ動作

### Example Demo / デモ例

```bash
cargo run --example highlight_demo
```

The demo showcases:
デモは以下を示します：

1. Basic red highlight (2 seconds)
   基本的な赤のハイライト（2秒）

2. Green highlight with custom duration
   カスタム時間の緑のハイライト

3. Blue highlight with thick border
   太い境界線の青のハイライト

4. Multiple simultaneous highlights
   複数の同時ハイライト

5. Match result highlighting
   マッチ結果のハイライト

## Future Enhancements / 将来の拡張

### macOS Implementation / macOS実装

Use NSWindow with borderless style and CALayer for border drawing:

境界線描画に境界なしスタイルとCALayerを使用したNSWindowを使用：

```rust
// Pseudocode
let window = NSWindow::alloc()
    .initWithContentRect(rect, NSWindowStyleMask::Borderless, ...)
    .setBackgroundColor(NSColor::clearColor())
    .setOpaque(false)
    .setLevel(NSFloatingWindowLevel)
    .setIgnoresMouseEvents(true);

let layer = view.layer();
layer.setBorderWidth(border_width);
layer.setBorderColor(cgColor);
```

### Linux Implementation / Linux実装

Use X11 override-redirect window with shape extension:

シェイプ拡張を使用したX11オーバーライドリダイレクトウィンドウを使用：

```rust
// Pseudocode
let window = conn.generate_id();
let values = CreateWindowAux::new()
    .override_redirect(1)
    .background_pixel(screen.black_pixel)
    .border_pixel(border_color);

conn.create_window(
    screen.root_depth,
    window,
    screen.root,
    x, y, width, height,
    border_width,
    WindowClass::INPUT_OUTPUT,
    screen.root_visual,
    &values,
)?;
```

### Additional Features / 追加機能

- [ ] Animated borders (blinking, pulsing)
      アニメーション境界線（点滅、パルス）

- [ ] Rounded corner support
      角丸サポート

- [ ] Shadow effects
      影効果

- [ ] Text labels on overlays
      オーバーレイ上のテキストラベル

- [ ] Screenshot capture with highlights
      ハイライト付きスクリーンショットキャプチャ

## Troubleshooting / トラブルシューティング

### Overlay Not Visible / オーバーレイが見えない

1. Check that the region coordinates are within screen bounds
   領域座標が画面境界内にあることを確認

2. Verify that the application has sufficient privileges
   アプリケーションに十分な権限があることを確認

3. On Windows, check if another always-on-top window is blocking
   Windowsでは、別の常に最前面ウィンドウがブロックしていないか確認

### Overlay Stays After Duration / オーバーレイが時間後も残る

1. Check that duration_ms > 0 (0 means manual close)
   duration_ms > 0であることを確認（0は手動クローズを意味）

2. Verify that the cleanup thread is running properly
   クリーンアップスレッドが適切に実行されていることを確認

### Multiple Overlays Interfering / 複数のオーバーレイが干渉

1. Add small delays between creating overlays
   オーバーレイの作成間に小さな遅延を追加

2. Ensure each overlay has unique window coordinates
   各オーバーレイが一意のウィンドウ座標を持つことを確認

## Performance Considerations / パフォーマンスの考慮事項

### Memory Usage / メモリ使用量

Each overlay window consumes:
各オーバーレイウィンドウが消費するもの：

- ~16KB for window structure
  ウィンドウ構造に約16KB
- Minimal GDI resources (1 pen)
  最小限のGDIリソース（1ペン）
- Thread stack space for auto-close (~1MB)
  自動クローズ用スレッドスタック空間（約1MB）

### CPU Usage / CPU使用量

- Initial creation: <1ms on modern hardware
  初期作成：最新ハードウェアで<1ms
- Drawing: <1ms (one-time)
  描画：<1ms（一回のみ）
- Auto-close thread: Negligible (sleeping)
  自動クローズスレッド：無視できる（スリープ中）

### Recommendations / 推奨事項

- Limit simultaneous overlays to <10 for best performance
  最高のパフォーマンスのために同時オーバーレイを<10に制限
- Use reasonable durations (avoid very long or very short)
  合理的な時間を使用（非常に長いまたは短いものを避ける）
- Clean up manually if creating many temporary overlays
  多くの一時的なオーバーレイを作成する場合は手動でクリーンアップ

## References / 参考文献

### Windows API Documentation

- [Layered Windows](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-features#layered-windows)
- [SetLayeredWindowAttributes](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setlayeredwindowattributes)
- [CreateWindowExW](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-createwindowexw)
- [Window Styles](https://docs.microsoft.com/en-us/windows/win32/winmsg/window-styles)
- [Extended Window Styles](https://docs.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)

### Related Documents

- [L4-PLATFORM-SPEC.md](../../.local/doc/spec/L4-PLATFORM-SPEC.md) - Platform abstraction layer specification
- [L1-L2-API-SPEC.md](../../.local/doc/spec/L1-L2-API-SPEC.md) - Public API specification

---

**Document Version / ドキュメントバージョン:** 1.0
**Date / 日付:** 2025-11-27
**Author / 著者:** Claude Code
