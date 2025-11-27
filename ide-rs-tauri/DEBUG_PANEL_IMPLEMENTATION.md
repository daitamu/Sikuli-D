# Debug Panel Implementation
# デバッグパネル実装

**Wave 3 Task 3-3C: Debug Panel for ide-rs-tauri**
**Wave 3 タスク 3-3C: ide-rs-tauri用デバッグパネル**

**Date / 日付**: 2025-11-27
**Status / ステータス**: Implemented / 実装済み

---

## Overview / 概要

This document describes the implementation of the debug panel for the SikuliX IDE (ide-rs-tauri). The debug panel integrates with the core-rs debugger to provide a comprehensive debugging experience.

このドキュメントは、SikuliX IDE（ide-rs-tauri）用デバッグパネルの実装について説明します。デバッグパネルは、core-rsデバッガと統合し、包括的なデバッグ体験を提供します。

---

## Implementation Details / 実装詳細

### Files Created / 作成されたファイル

1. **`ide-rs-tauri/src/debug.rs`**
   - Rust backend implementation
   - Tauri commands for debug control
   - State management for debug sessions
   - Event forwarding from core-rs to frontend

2. **`ide-rs-tauri/types/debug.ts`**
   - TypeScript type definitions
   - Interface definitions for debug commands
   - Type-safe API for frontend

3. **`ide-rs-tauri/examples/DebugPanel.tsx`**
   - React component reference implementation
   - Complete UI with toolbar, call stack, variables, breakpoints
   - Example event handling

### Files Modified / 変更されたファイル

1. **`ide-rs-tauri/src/main.rs`**
   - Added `mod debug;` module declaration
   - Registered `DebugPanelState` in Tauri app state
   - Added 18 debug commands to invoke handler

---

## Architecture / アーキテクチャ

```
┌─────────────────────────────────────────────────────────────┐
│                  Frontend (React/TypeScript)                 │
│  ┌────────────────────────────────────────────────────────┐ │
│  │             DebugPanel Component                        │ │
│  │  • Toolbar (Resume, Pause, Step, Stop)                 │ │
│  │  • Call Stack Display                                  │ │
│  │  • Variable Inspector                                  │ │
│  │  • Breakpoint List                                     │ │
│  │  • Watch Expressions                                   │ │
│  └────────────────────────────────────────────────────────┘ │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Tauri IPC (invoke + events)
                         │
┌────────────────────────▼────────────────────────────────────┐
│              Tauri Backend (Rust)                            │
│  ┌────────────────────────────────────────────────────────┐ │
│  │           debug.rs - Tauri Commands                    │ │
│  │  • debug_init_session / debug_end_session             │ │
│  │  • debug_add/remove/toggle_breakpoint                 │ │
│  │  • debug_pause/resume/step_over/step_into/step_out    │ │
│  │  • debug_get_variables / debug_get_call_stack         │ │
│  │  • debug_evaluate_expression                          │ │
│  └────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────┐ │
│  │        DebugPanelState - State Management             │ │
│  │  • Debugger instance (Arc<Mutex<Option<Debugger>>>)  │ │
│  │  • Current script path                                │ │
│  │  • Event callback registration                        │ │
│  └────────────────────────────────────────────────────────┘ │
└────────────────────────┬────────────────────────────────────┘
                         │
                         │ Library calls
                         │
┌────────────────────────▼────────────────────────────────────┐
│                core-rs/src/debug/debugger.rs                 │
│  • Breakpoint management                                     │
│  • Execution control (pause, resume, step)                  │
│  • Variable inspection                                      │
│  • Call stack tracking                                      │
│  • Event notification                                       │
└─────────────────────────────────────────────────────────────┘
```

---

## API Reference / API リファレンス

### Tauri Commands / Tauriコマンド

#### Session Management / セッション管理

```rust
// Initialize debug session
// デバッグセッションを初期化
debug_init_session(scriptPath: string) -> Result<(), String>

// End debug session
// デバッグセッションを終了
debug_end_session() -> Result<(), String>
```

#### Breakpoint Management / ブレークポイント管理

```rust
// Add breakpoint
// ブレークポイントを追加
debug_add_breakpoint(file: string, line: number) -> Result<(), String>

// Remove breakpoint
// ブレークポイントを削除
debug_remove_breakpoint(file: string, line: number) -> Result<(), String>

// Toggle breakpoint (returns true if now set)
// ブレークポイントを切り替え（設定された場合trueを返す）
debug_toggle_breakpoint(file: string, line: number) -> Result<bool, String>

// List all breakpoints
// すべてのブレークポイントをリスト
debug_list_breakpoints() -> Result<Vec<BreakpointInfo>, String>

// Clear all breakpoints
// すべてのブレークポイントをクリア
debug_clear_breakpoints() -> Result<(), String>
```

#### Execution Control / 実行制御

```rust
// Pause execution
// 実行を一時停止
debug_pause() -> Result<(), String>

// Resume execution
// 実行を再開
debug_resume() -> Result<(), String>

// Step over current line
// 現在の行をステップオーバー
debug_step_over() -> Result<(), String>

// Step into function
// 関数にステップイン
debug_step_into() -> Result<(), String>

// Step out of current function
// 現在の関数からステップアウト
debug_step_out() -> Result<(), String>

// Stop execution
// 実行を停止
debug_stop() -> Result<(), String>
```

#### State Inspection / 状態検査

```rust
// Get current debug state
// 現在のデバッグ状態を取得
debug_get_state() -> Result<DebugState, String>

// Get variables in scope (local/global/all)
// スコープ内の変数を取得（local/global/all）
debug_get_variables(scope?: string) -> Result<Vec<VariableInfo>, String>

// Get call stack
// コールスタックを取得
debug_get_call_stack() -> Result<Vec<CallFrame>, String>

// Get current execution position
// 現在の実行位置を取得
debug_get_current_position() -> Result<Option<(String, u32)>, String>

// Evaluate expression
// 式を評価
debug_evaluate_expression(expr: string) -> Result<String, String>
```

### Events / イベント

The debugger emits events via Tauri's event system on the `debug-event` channel:

デバッガは、Tauriのイベントシステムを介して`debug-event`チャネルでイベントを発行します：

```typescript
type DebugEvent =
  | { type: 'breakpointHit'; file: string; line: number; hitCount: number }
  | { type: 'paused'; file: string; line: number }
  | { type: 'resumed' }
  | { type: 'stepCompleted'; file: string; line: number }
  | { type: 'stopped' }
  | { type: 'error'; message: string }
  | { type: 'variableChanged'; name: string; value: string };
```

#### Listening to Events / イベントのリッスン

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<DebugEvent>('debug-event', (event) => {
  console.log('Debug event:', event.payload);

  switch (event.payload.type) {
    case 'breakpointHit':
      // Handle breakpoint hit
      // ブレークポイントヒットを処理
      highlightLine(event.payload.file, event.payload.line);
      break;

    case 'paused':
      // Handle pause
      // 一時停止を処理
      updateUI('paused');
      break;

    // ... handle other events
  }
});

// Clean up when component unmounts
// コンポーネントアンマウント時にクリーンアップ
return () => unlisten();
```

---

## Usage Example / 使用例

### Basic Integration / 基本的な統合

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

// 1. Initialize debug session
// デバッグセッションを初期化
await invoke('debug_init_session', {
  scriptPath: '/path/to/script.py'
});

// 2. Add breakpoints
// ブレークポイントを追加
await invoke('debug_add_breakpoint', {
  file: '/path/to/script.py',
  line: 42
});

// 3. Listen for events
// イベントをリッスン
const unlisten = await listen('debug-event', (event) => {
  handleDebugEvent(event.payload);
});

// 4. Control execution
// 実行を制御
await invoke('debug_resume'); // Start running
await invoke('debug_pause');  // Pause
await invoke('debug_step_over'); // Step

// 5. Inspect state
// 状態を検査
const variables = await invoke('debug_get_variables', { scope: 'local' });
const callStack = await invoke('debug_get_call_stack');

// 6. Clean up
// クリーンアップ
await invoke('debug_end_session');
unlisten();
```

### Full Component Example / 完全なコンポーネント例

See `ide-rs-tauri/examples/DebugPanel.tsx` for a complete React component implementation.

完全なReactコンポーネント実装については、`ide-rs-tauri/examples/DebugPanel.tsx`を参照してください。

---

## Data Types / データ型

### DebugState / デバッグ状態

```typescript
enum DebugState {
  NotStarted = 'notStarted',
  Running = 'running',
  Paused = 'paused',
  StepOver = 'stepOver',
  StepInto = 'stepInto',
  StepOut = 'stepOut',
  Stopped = 'stopped',
  Error = 'error',
}
```

### VariableInfo / 変数情報

```typescript
interface VariableInfo {
  name: string;       // Variable name
  value: string;      // String representation of value
  typeName: string;   // Type name (int, str, Region, etc.)
  scope: 'local' | 'global' | 'all';
}
```

### CallFrame / コールフレーム

```typescript
interface CallFrame {
  depth: number;      // 0 = current frame
  function: string;   // Function name
  file: string;       // File path
  line: number;       // Line number
}
```

### BreakpointInfo / ブレークポイント情報

```typescript
interface BreakpointInfo {
  file: string;       // File path
  line: number;       // Line number
}
```

---

## Features / 機能

### Implemented / 実装済み

- ✅ Breakpoint management (add, remove, toggle, list, clear)
  ブレークポイント管理（追加、削除、切り替え、リスト、クリア）

- ✅ Execution control (pause, resume, step over, step into, step out, stop)
  実行制御（一時停止、再開、ステップオーバー、ステップイン、ステップアウト、停止）

- ✅ Variable inspection (local, global, all scopes)
  変数インスペクション（ローカル、グローバル、すべてのスコープ）

- ✅ Call stack tracking
  コールスタック追跡

- ✅ Expression evaluation
  式評価

- ✅ Event-driven architecture with Tauri events
  Tauriイベントを使用したイベント駆動アーキテクチャ

- ✅ Type-safe TypeScript API
  型安全なTypeScript API

- ✅ Session management (init/end)
  セッション管理（初期化/終了）

### UI Components (Example) / UIコンポーネント（例）

The example DebugPanel.tsx includes:
例のDebugPanel.tsxには以下が含まれます：

- ✅ Debug toolbar with controls
  制御付きデバッグツールバー

- ✅ Call stack viewer
  コールスタックビューア

- ✅ Variable inspector (table view)
  変数インスペクター（テーブルビュー）

- ✅ Breakpoint list
  ブレークポイントリスト

- ✅ Watch expression evaluator
  ウォッチ式エバリュエーター

- ✅ Current position indicator
  現在位置インジケーター

---

## Integration with IDE / IDEとの統合

### Editor Integration / エディタ統合

The debug panel should be integrated with your code editor to:

デバッグパネルは、以下のためにコードエディタと統合する必要があります：

1. **Breakpoint Markers / ブレークポイントマーカー**
   - Display breakpoint indicators in the gutter
   - Allow clicking line numbers to toggle breakpoints
   - 行番号の余白にブレークポイントインジケーターを表示
   - 行番号をクリックしてブレークポイントを切り替え可能に

2. **Current Line Highlighting / 現在行のハイライト**
   - Highlight the current execution line
   - Scroll to current line when paused
   - 現在の実行行をハイライト
   - 一時停止時に現在行までスクロール

3. **Variable Hover / 変数ホバー**
   - Show variable values on hover (future enhancement)
   - ホバー時に変数値を表示（将来の拡張）

### Layout Suggestions / レイアウト提案

```
┌─────────────────────────────────────────────────────────────┐
│  Toolbar: [Run] [Debug] [Stop] | [Capture] [OCR]           │
├──────────────┬──────────────────────────────────┬───────────┤
│              │                                  │           │
│  Project     │        Code Editor               │  Debug    │
│  Explorer    │        (with breakpoints)        │  Panel    │
│              │                                  │           │
│  Files:      │  10 ● def helper():              │  [▶️][⏸️] │
│  □ script.py │  11     region = Region(...)     │  [⏭️][⬇️] │
│  □ utils.py  │  12 ►   match = find("btn.png")  │  [⬆️][⏹️] │
│              │  13     if match:                │           │
│              │  14       click(match)           │  Stack:   │
│              │                                  │  ▶ helper │
│              │                                  │    main   │
│              │                                  │           │
│              │                                  │  Vars:    │
│              │                                  │  region=  │
│              │                                  │  match=   │
├──────────────┴──────────────────────────────────┴───────────┤
│  Output Console                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Testing / テスト

### Unit Tests / ユニットテスト

The debug.rs module includes unit tests:

debug.rsモジュールにはユニットテストが含まれています：

```bash
cargo test --lib debug
```

Tests cover:
テストカバー範囲：

- DebugState enum conversion
- DebugPanelState creation and initialization
- BreakpointInfo serialization

### Integration Testing / 統合テスト

To test the full integration:

完全な統合をテストするには：

1. Build the IDE:
   ```bash
   cd ide-rs-tauri
   npm install  # Install frontend dependencies
   npm run tauri build
   ```

2. Run the IDE and test debug features:
   - Open a Python script
   - Click line numbers to set breakpoints
   - Click "Debug" button to start
   - Use toolbar controls (pause, step, resume)
   - Inspect variables and call stack

---

## Future Enhancements / 将来の拡張

### Planned Features / 計画機能

- 🔲 **Conditional Breakpoints / 条件付きブレークポイント**
  - Set breakpoints with conditions (e.g., `x > 10`)
  - 条件付きブレークポイントの設定（例：`x > 10`）

- 🔲 **Hit Count Breakpoints / ヒットカウントブレークポイント**
  - Break after N hits
  - N回ヒット後にブレーク

- 🔲 **Log Points / ログポイント**
  - Log expressions without stopping
  - 停止せずに式をログ

- 🔲 **Variable Editing / 変数編集**
  - Modify variable values during debugging
  - デバッグ中に変数値を変更

- 🔲 **Visual Variable Inspection / ビジュアル変数インスペクション**
  - Show Region/Match objects visually
  - Region/Matchオブジェクトをビジュアル表示

- 🔲 **Screenshot on Break / ブレーク時のスクリーンショット**
  - Capture screen state at breakpoint
  - ブレークポイントで画面状態をキャプチャ

- 🔲 **Debug Console / デバッグコンソール**
  - REPL for evaluating expressions
  - 式評価用REPL

---

## Troubleshooting / トラブルシューティング

### Common Issues / よくある問題

**1. "Debugger not initialized" error**
**"デバッガが初期化されていません"エラー**

Solution: Call `debug_init_session` before using other debug commands.
解決策：他のデバッグコマンドを使用する前に`debug_init_session`を呼び出してください。

**2. Events not received**
**イベントが受信されない**

Solution: Ensure event listener is registered before starting debug session.
解決策：デバッグセッション開始前にイベントリスナーが登録されていることを確認してください。

**3. Breakpoints not working**
**ブレークポイントが機能しない**

Solution: Verify that breakpoints are set before running the script with `sikulix run --debug`.
解決策：`sikulix run --debug`でスクリプトを実行する前にブレークポイントが設定されていることを確認してください。

---

## Dependencies / 依存関係

### Rust Dependencies / Rust依存関係

- `sikulix-core`: Core debugger implementation
- `tauri`: Tauri framework for desktop app
- `serde`: Serialization/deserialization
- `log`: Logging

### TypeScript Dependencies / TypeScript依存関係

- `@tauri-apps/api`: Tauri JavaScript bindings
- React (for example component)

---

## References / 参照

- Core debugger implementation: `core-rs/src/debug/debugger.rs`
- Design specification: `.local/doc/spec/IDE-RS-TAURI-DESIGN.md`
- Project rules: `.claude/CLAUDE.md`

---

## Changelog / 変更履歴

### 2025-11-27 - Initial Implementation / 初期実装

- ✅ Created `debug.rs` with 18 Tauri commands
- ✅ Integrated with core-rs debugger
- ✅ Event-driven architecture with Tauri events
- ✅ TypeScript type definitions
- ✅ React component example
- ✅ Comprehensive documentation

---

**END OF DOCUMENT / ドキュメント終了**
