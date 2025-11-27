# macOS Highlight Overlay Implementation Notes
# macOS ハイライトオーバーレイ実装ノート

**Date / 日付:** 2025-11-27
**Task:** Wave 1 Task 3-1D: macOS Highlight Overlay Implementation
**Status / ステータス:** ✅ Completed / 完了

---

## Summary / 概要

Implemented visual highlight overlay functionality for macOS using NSWindow with floating window level and CALayer for border rendering.
フローティングウィンドウレベルのNSWindowとボーダー描画用のCALayerを使用して、macOS用の視覚的ハイライトオーバーレイ機能を実装しました。

---

## Files Created / 作成されたファイル

### 1. `core-rs/src/highlight/mod.rs`
- Main highlight module with platform abstraction
  プラットフォーム抽象化を備えたメインハイライトモジュール
- High-level `Highlight` struct with `show_for()` method
  `show_for()`メソッドを備えた高レベル`Highlight`構造体
- Platform-specific module imports and re-exports
  プラットフォーム固有モジュールのインポートと再エクスポート

### 2. `core-rs/src/highlight/macos_overlay.rs`
- Full macOS implementation using Objective-C runtime
  Objective-Cランタイムを使用した完全なmacOS実装
- Key features / 主な機能:
  - NSWindow creation with borderless style / ボーダーレススタイルでのNSWindow作成
  - Floating window level (always on top) / フローティングウィンドウレベル（常に最前面）
  - Click-through support (ignores mouse events) / クリックスルーサポート（マウスイベントを無視）
  - CALayer-based border rendering / CALayerベースのボーダー描画
  - Customizable color and border width / カスタマイズ可能な色とボーダー幅
  - Automatic timeout and cleanup / 自動タイムアウトとクリーンアップ

### 3. `core-rs/src/highlight/windows_overlay.rs`
- Stub implementation for Windows
  Windows用のスタブ実装
- Placeholder for future GDI+ layered window implementation
  将来のGDI+レイヤードウィンドウ実装用のプレースホルダー

### 4. `core-rs/src/highlight/linux_overlay.rs`
- Stub implementation for Linux
  Linux用のスタブ実装
- Placeholder for future X11 override-redirect window implementation
  将来のX11オーバーライドリダイレクトウィンドウ実装用のプレースホルダー

---

## Implementation Details / 実装詳細

### macOS NSWindow Configuration / macOS NSWindowの設定

```rust
// Window properties / ウィンドウプロパティ
- Style: NS_WINDOW_STYLE_MASK_BORDERLESS (0) - No title bar or border
- Backing: NS_BACKING_STORE_BUFFERED (2) - Buffered backing store
- Level: NS_FLOATING_WINDOW_LEVEL (3) - Always on top
- Background: clearColor - Transparent background
- Opaque: false - Allows transparency
- IgnoresMouseEvents: true - Click-through enabled
- HasShadow: false - No shadow
```

### CALayer Border Configuration / CALayerボーダー設定

```rust
// Layer properties / レイヤープロパティ
- BorderWidth: Configurable (default 3.0 pixels)
- BorderColor: CGColor from NSColor (RGBA)
- BackgroundColor: Semi-transparent (default 20% opacity)
- CornerRadius: 4.0 pixels (rounded corners)
```

### Coordinate System Conversion / 座標系変換

macOS uses bottom-left origin coordinate system, while SikuliX uses top-left origin.
macOSは左下原点の座標系を使用しますが、SikuliXは左上原点を使用します。

```rust
// Convert from SikuliX (top-left) to macOS (bottom-left)
// SikuliX（左上）からmacOS（左下）に変換
let ns_y = screen_height - (region.y as f64) - (region.height as f64);
```

---

## API Design / API設計

### Public Functions / 公開関数

#### `highlight(region: &Region, duration_ms: u64, color: Color) -> Result<()>`
Display a highlight overlay with specified color and duration.
指定された色と時間でハイライトオーバーレイを表示します。

#### `highlight_with_config(region: &Region, config: &HighlightConfig) -> Result<()>`
Display a highlight overlay with custom configuration.
カスタム設定でハイライトオーバーレイを表示します。

#### `highlight_match(match_region: &Region, duration_ms: u64) -> Result<()>`
Convenience function for highlighting match results (red color).
マッチ結果をハイライトする便利関数（赤色）。

### Configuration Structs / 設定構造体

#### `Color`
```rust
pub struct Color {
    pub r: f64, // 0.0 - 1.0
    pub g: f64, // 0.0 - 1.0
    pub b: f64, // 0.0 - 1.0
    pub a: f64, // 0.0 - 1.0
}
```

#### `HighlightConfig`
```rust
pub struct HighlightConfig {
    pub color: Color,
    pub border_width: f64,
    pub duration_ms: u64,
    pub background_opacity: f64,
}
```

---

## Usage Example / 使用例

```rust
use sikulix_core::{Highlight, Region};

// Basic usage / 基本的な使い方
let region = Region::new(100, 100, 200, 150);
let highlight = Highlight::new(region);
highlight.show_for(2.0)?; // Show for 2 seconds

// Advanced usage with custom configuration / カスタム設定での高度な使用
use sikulix_core::highlight::{highlight_with_config, Color, HighlightConfig};

let config = HighlightConfig {
    color: Color::new(0.0, 1.0, 0.0, 1.0), // Green
    border_width: 5.0,
    duration_ms: 3000,
    background_opacity: 0.3,
};

highlight_with_config(&region, &config)?;
```

---

## Technical Challenges & Solutions / 技術的課題と解決策

### Challenge 1: Objective-C Interop / 課題1: Objective-C相互運用
**Problem:** Direct FFI to AppKit/Cocoa APIs requires careful handling of Objective-C runtime.
**問題:** AppKit/Cocoa APIへの直接FFIにはObjective-Cランタイムの慎重な処理が必要です。

**Solution:** Used `objc` crate with `msg_send!` macro for safe Objective-C message passing.
**解決策:** 安全なObjective-Cメッセージパッシングのために`objc`クレートと`msg_send!`マクロを使用しました。

### Challenge 2: Memory Management / 課題2: メモリ管理
**Problem:** NSWindow and NSView require proper retain/release for memory safety.
**問題:** NSWindowとNSViewはメモリ安全性のための適切なretain/releaseが必要です。

**Solution:** Implemented `HighlightHandle` with `Drop` trait for automatic cleanup.
**解決策:** 自動クリーンアップのための`Drop`トレイトを備えた`HighlightHandle`を実装しました。

### Challenge 3: Coordinate System / 課題3: 座標系
**Problem:** macOS uses bottom-left origin while SikuliX uses top-left origin.
**問題:** macOSは左下原点を使用しますが、SikuliXは左上原点を使用します。

**Solution:** Implemented `get_screen_height()` and coordinate conversion in `create_highlight_window()`.
**解決策:** `get_screen_height()`と`create_highlight_window()`での座標変換を実装しました。

### Challenge 4: NSRect Structure / 課題4: NSRect構造体
**Problem:** Need proper C-compatible struct layout for NSRect passing to Objective-C.
**問題:** Objective-CへのNSRect渡しに適切なC互換構造体レイアウトが必要です。

**Solution:** Defined `#[repr(C)]` structs for NSRect, NSPoint, and NSSize.
**解決策:** NSRect、NSPoint、NSSize用の`#[repr(C)]`構造体を定義しました。

---

## Dependencies Added / 追加された依存関係

Updated `Cargo.toml` with:
`Cargo.toml`を以下で更新しました：

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-graphics = "0.24"
core-foundation = "0.10"
cocoa = "0.25"
objc = "0.2"
objc-foundation = "0.1"
```

---

## Testing / テスト

### Unit Tests / ユニットテスト
- Color creation and clamping / 色の作成とクランプ
- Color preset functions (red, green, blue, yellow) / 色プリセット関数（赤、緑、青、黄）
- Configuration defaults / 設定のデフォルト値

### Integration Tests / 統合テスト
Actual window creation tests require macOS environment with GUI support and are marked with `#[ignore]`.
実際のウィンドウ作成テストにはGUIサポート付きmacOS環境が必要で、`#[ignore]`でマークされています。

---

## Future Enhancements / 将来の拡張

### Windows Implementation / Windows実装
- Layered window with GDI+ / GDI+付きレイヤードウィンドウ
- Transparency support with UpdateLayeredWindow / UpdateLayeredWindowでの透明度サポート
- Hardware acceleration / ハードウェアアクセラレーション

### Linux Implementation / Linux実装
- X11 override-redirect window / X11オーバーライドリダイレクトウィンドウ
- XShape extension for transparency / 透明度用のXShape拡張
- Wayland support (compositor-dependent) / Waylandサポート（コンポジター依存）

### Additional Features / 追加機能
- Animation effects (fade in/out) / アニメーション効果（フェードイン/アウト）
- Multiple simultaneous highlights / 複数同時ハイライト
- Custom shapes (circle, rounded rectangle) / カスタム形状（円、角丸矩形）
- Highlight tracking (follow moving windows) / ハイライト追跡（移動するウィンドウを追従）

---

## References / 参考文献

- **L4-PLATFORM-SPEC.md**: Section 8 - Visual Highlight Overlay / セクション8 - ビジュアルハイライトオーバーレイ
- **Apple Documentation**: NSWindow, NSView, CALayer APIs
- **objc crate**: https://docs.rs/objc/
- **SikuliX Original Implementation**: Java AWT/Swing overlay

---

## Checklist / チェックリスト

- [x] macOS overlay implementation with NSWindow / NSWindowでのmacOSオーバーレイ実装
- [x] CALayer border rendering / CALayerボーダー描画
- [x] Click-through support / クリックスルーサポート
- [x] Customizable color and duration / カスタマイズ可能な色と時間
- [x] Automatic cleanup and memory management / 自動クリーンアップとメモリ管理
- [x] Coordinate system conversion / 座標系変換
- [x] Unit tests for configuration structs / 設定構造体のユニットテスト
- [x] Windows stub implementation / Windowsスタブ実装
- [x] Linux stub implementation / Linuxスタブ実装
- [x] Integration with highlight module / highlightモジュールとの統合
- [x] Documentation and code comments / ドキュメントとコードコメント

---

## Conclusion / 結論

The macOS highlight overlay implementation is complete and ready for integration testing.
The design follows the L4-PLATFORM-SPEC.md specification and provides a solid foundation
for future platform implementations.

macOSハイライトオーバーレイ実装が完了し、統合テストの準備が整いました。
設計はL4-PLATFORM-SPEC.md仕様に従い、将来のプラットフォーム実装の
堅固な基盤を提供します。

**Next Steps / 次のステップ:**
1. Build and test on actual macOS environment / 実際のmacOS環境でビルドしてテスト
2. Implement Windows overlay using GDI+ / GDI+を使用してWindowsオーバーレイを実装
3. Implement Linux overlay using X11 / X11を使用してLinuxオーバーレイを実装
4. Add integration tests with screen capture / スクリーンキャプチャとの統合テストを追加
