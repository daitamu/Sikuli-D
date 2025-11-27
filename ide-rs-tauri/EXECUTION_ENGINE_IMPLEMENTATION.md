# Script Execution Engine Implementation
# スクリプト実行エンジン実装

**Date / 日付**: 2025-11-27
**Task**: Wave 3 Task 3-3A
**Status / ステータス**: Completed / 完了

---

## Overview / 概要

This implementation adds a comprehensive script execution engine to ide-rs-tauri that calls the sikulix CLI (runtime-rs) as a subprocess. Following the architectural principle that the IDE does NOT directly use core-rs for execution.

この実装では、sikulix CLI (runtime-rs) をサブプロセスとして呼び出す包括的なスクリプト実行エンジンをide-rs-tauriに追加しました。IDEは実行のためにcore-rsを直接使用しないというアーキテクチャ原則に従っています。

---

## Files Created / 作成されたファイル

### 1. `src/execution.rs` (577 lines)

**Purpose / 目的:**
- Script execution engine module
- Manages subprocess execution of sikulix CLI
- Provides real-time output streaming
- Process lifecycle management

**Key Components / 主要コンポーネント:**

#### Data Structures / データ構造

```rust
// Script execution options
pub struct ScriptRunOptions {
    pub working_dir: Option<String>,
    pub args: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub debug: bool,
    pub timeout_secs: Option<u64>,
}

// Execution result
pub struct ScriptRunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub success: bool,
}

// Process state management
pub struct ScriptProcessState {
    processes: Arc<AsyncMutex<HashMap<String, RunningProcess>>>,
}

// Script executor
pub struct ScriptExecutor {
    sikulix_path: PathBuf,
}
```

#### Tauri Commands / Tauriコマンド

1. **`run_script`** - Execute and wait for completion
   - スクリプトを実行して完了を待つ
   - Supports timeout / タイムアウト対応
   - Returns full output / 完全な出力を返す

2. **`run_script_streaming`** - Execute with real-time output
   - リアルタイム出力付きで実行
   - Events: `script-stdout`, `script-stderr`, `script-complete`
   - Non-blocking execution / ノンブロッキング実行

3. **`stop_script`** - Stop running script by ID
   - IDによって実行中のスクリプトを停止
   - Graceful process termination / グレースフルなプロセス終了

4. **`get_running_processes`** - List running scripts
   - 実行中のスクリプトをリスト

5. **`stop_all_scripts`** - Stop all running scripts
   - すべての実行中のスクリプトを停止

---

## Files Modified / 変更されたファイル

### 1. `src/main.rs`

**Changes / 変更内容:**

```rust
// Added module import
mod execution;

// Added state management
.manage(execution::ScriptProcessState::new())

// Added command handlers
execution::run_script,
execution::run_script_streaming,
execution::stop_script,
execution::get_running_processes,
execution::stop_all_scripts,
```

### 2. `Cargo.toml`

**Dependencies Added / 追加された依存関係:**

```toml
# UUID for process IDs
uuid = { version = "1", features = ["v4"] }

# Extended tokio features
tokio = { version = "1", features = [
    "rt-multi-thread",
    "macros",
    "time",
    "process",      # Added for subprocess
    "io-util"       # Added for async I/O
] }
```

---

## Architecture / アーキテクチャ

### Design Principle / 設計原則

**IDE delegates execution to runtime-rs, NOT core-rs directly**
**IDEは実行をruntime-rsに委譲し、core-rsを直接使用しない**

```
┌──────────────────┐
│  IDE (Tauri)     │
│                  │
│  User clicks     │
│  "Run Script"    │
└────────┬─────────┘
         │
         │  subprocess: sikulix run script.py
         │  サブプロセス
         ▼
┌──────────────────┐
│  runtime-rs      │
│  (sikulix CLI)   │
└────────┬─────────┘
         │
         │  library call
         ▼
┌──────────────────┐
│  core-rs         │
│  (execution)     │
└──────────────────┘
```

### Execution Flow / 実行フロー

#### Non-Streaming Mode / 非ストリーミングモード

```
Frontend                Tauri Backend              sikulix CLI
   │                          │                         │
   │─invoke('run_script')────→│                         │
   │                          │─spawn subprocess───────→│
   │                          │                         │
   │                          │                    [executing]
   │                          │                         │
   │                          │←──exit + output────────│
   │←─ScriptRunResult─────────│                         │
   │                          │                         │
```

#### Streaming Mode / ストリーミングモード

```
Frontend                Tauri Backend              sikulix CLI
   │                          │                         │
   │─invoke('run_script_streaming')→│                   │
   │←─process_id──────────────│                         │
   │                          │─spawn subprocess───────→│
   │                          │                         │
   │                          │                    [executing]
   │                          │                         │
   │←─event: script-stdout────│←─stdout line───────────│
   │←─event: script-stdout────│←─stdout line───────────│
   │←─event: script-stderr────│←─stderr line───────────│
   │                          │                         │
   │←─event: script-complete──│←─exit──────────────────│
   │                          │                         │
```

### Process Management / プロセス管理

```rust
// Each spawned process is tracked with:
struct RunningProcess {
    child: Child,           // Tokio async child process
    start_time: Instant,    // For duration tracking
    script_path: String,    // For logging
}

// Global state stores all running processes
ScriptProcessState {
    processes: HashMap<UUID, RunningProcess>
}
```

---

## Command Details / コマンド詳細

### `run_script`

**Usage / 使用方法:**

```typescript
const result = await invoke('run_script', {
  scriptPath: '/path/to/script.py',
  options: {
    working_dir: '/path/to/workdir',
    args: ['arg1', 'arg2'],
    env_vars: { 'KEY': 'value' },
    debug: true,
    timeout_secs: 300  // 5 minutes
  }
});

// Result structure
{
  exit_code: 0,
  stdout: "Script output...",
  stderr: "",
  duration_ms: 1234,
  success: true
}
```

**Features / 機能:**
- Synchronous execution / 同期実行
- Optional timeout / オプションのタイムアウト
- Full output buffering / 完全な出力バッファリング
- Environment variable support / 環境変数サポート
- Debug mode / デバッグモード

---

### `run_script_streaming`

**Usage / 使用方法:**

```typescript
// Start script
const processId = await invoke('run_script_streaming', {
  scriptPath: '/path/to/script.py',
  options: { debug: true }
});

// Listen for output
await listen('script-stdout', (event) => {
  const { process_id, line } = event.payload;
  console.log(`[${process_id}] ${line}`);
});

await listen('script-stderr', (event) => {
  const { process_id, line } = event.payload;
  console.error(`[${process_id}] ${line}`);
});

await listen('script-complete', (event) => {
  const { process_id, exit_code } = event.payload;
  console.log(`[${process_id}] Completed: ${exit_code}`);
});
```

**Features / 機能:**
- Asynchronous execution / 非同期実行
- Real-time output streaming / リアルタイム出力ストリーミング
- Separate stdout/stderr streams / stdout/stderrの分離
- Non-blocking / ノンブロッキング
- Process ID tracking / プロセスID追跡

**Events / イベント:**

```rust
// Stdout line
{
  "process_id": "uuid",
  "line": "output text"
}

// Stderr line
{
  "process_id": "uuid",
  "line": "error text"
}

// Completion
{
  "process_id": "uuid",
  "exit_code": 0 | null
}

// Error
{
  "process_id": "uuid",
  "error": "error message"
}
```

---

### `stop_script`

**Usage / 使用方法:**

```typescript
await invoke('stop_script', {
  processId: processId
});
```

**Behavior / 動作:**
- Terminates running process / 実行中のプロセスを終了
- Removes from process map / プロセスマップから削除
- Emits completion event / 完了イベントを送信
- Error if process not found / プロセスが見つからない場合はエラー

---

### `get_running_processes`

**Usage / 使用方法:**

```typescript
const processes = await invoke('get_running_processes');
// Returns: ["uuid1", "uuid2", ...]
```

---

### `stop_all_scripts`

**Usage / 使用方法:**

```typescript
await invoke('stop_all_scripts');
```

**Behavior / 動作:**
- Stops all tracked processes / すべての追跡プロセスを停止
- Clears process map / プロセスマップをクリア
- Logs each termination / 各終了をログ

---

## Testing / テスト

### Unit Tests / ユニットテスト

The execution module includes comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    // Data structure tests
    test_script_run_options_default()
    test_script_run_options_serialization()

    // Executor tests
    test_script_executor_creation()
    test_script_process_state_default()

    // Command builder tests
    test_build_command_basic()
    test_build_command_with_debug()
    test_build_command_with_args()
}
```

**Run tests / テスト実行:**

```bash
cargo test --package sikulix-ide-tauri --lib execution::tests
```

---

## Integration / 統合

### Frontend Integration / フロントエンド統合

**Example React Component:**

```typescript
import { invoke, listen } from '@tauri-apps/api';
import { useState, useEffect } from 'react';

function ScriptRunner({ scriptPath }) {
  const [output, setOutput] = useState([]);
  const [processId, setProcessId] = useState(null);
  const [running, setRunning] = useState(false);

  useEffect(() => {
    const unlisten1 = listen('script-stdout', (event) => {
      setOutput(prev => [...prev, { type: 'stdout', text: event.payload.line }]);
    });

    const unlisten2 = listen('script-stderr', (event) => {
      setOutput(prev => [...prev, { type: 'stderr', text: event.payload.line }]);
    });

    const unlisten3 = listen('script-complete', (event) => {
      setRunning(false);
      console.log('Script completed:', event.payload.exit_code);
    });

    return () => {
      unlisten1.then(fn => fn());
      unlisten2.then(fn => fn());
      unlisten3.then(fn => fn());
    };
  }, []);

  const runScript = async () => {
    try {
      setRunning(true);
      setOutput([]);
      const id = await invoke('run_script_streaming', {
        scriptPath,
        options: { debug: false }
      });
      setProcessId(id);
    } catch (err) {
      console.error('Failed to run script:', err);
      setRunning(false);
    }
  };

  const stopScript = async () => {
    if (processId) {
      await invoke('stop_script', { processId });
    }
  };

  return (
    <div>
      <button onClick={runScript} disabled={running}>Run</button>
      <button onClick={stopScript} disabled={!running}>Stop</button>

      <pre>
        {output.map((line, i) => (
          <div key={i} className={line.type}>
            {line.text}
          </div>
        ))}
      </pre>
    </div>
  );
}
```

---

## Error Handling / エラーハンドリング

### Common Errors / 一般的なエラー

1. **sikulix binary not found**
   ```rust
   Error: "Failed to spawn process: No such file or directory"
   ```
   **Solution / 解決策:** Ensure sikulix is in PATH or configure `sikulix_path`

2. **Script timeout**
   ```rust
   Error: "Script execution timed out after 300 seconds"
   ```
   **Solution / 解決策:** Increase `timeout_secs` or optimize script

3. **Process not found**
   ```rust
   Error: "Process {uuid} not found"
   ```
   **Solution / 解決策:** Process already completed or ID is invalid

4. **Permission denied**
   ```rust
   Error: "Failed to execute script: Permission denied"
   ```
   **Solution / 解決策:** Check script file permissions

---

## Performance Considerations / パフォーマンス考慮事項

### Memory Usage / メモリ使用量

- **Non-streaming mode:** Buffers entire output in memory
  - 非ストリーミングモード：出力全体をメモリにバッファ
  - Use for short scripts only / 短いスクリプトのみに使用

- **Streaming mode:** Line-by-line processing
  - ストリーミングモード：行単位の処理
  - Recommended for long-running scripts / 長時間実行スクリプトに推奨

### Process Limits / プロセス制限

- No hard limit on concurrent processes / 同時プロセス数に明確な制限なし
- Limited by system resources / システムリソースによって制限
- Consider implementing max concurrent process limit / 最大同時プロセス数の制限実装を検討

### Cleanup / クリーンアップ

- Processes are automatically removed when complete / プロセスは完了時に自動削除
- Killed processes are removed immediately / キルされたプロセスは即座に削除
- No zombie processes / ゾンビプロセスなし

---

## Security Considerations / セキュリティ考慮事項

### Input Validation / 入力検証

- **Script path:** No validation currently implemented
  - スクリプトパス：現在検証は実装されていません
  - **TODO:** Add path traversal prevention / パストラバーサル防止を追加

- **Arguments:** Passed directly to subprocess
  - 引数：サブプロセスに直接渡される
  - **TODO:** Sanitize user-provided arguments / ユーザー提供の引数をサニタイズ

### Environment Variables / 環境変数

- User can set arbitrary environment variables / ユーザーは任意の環境変数を設定可能
- **Risk:** Potential for sensitive data exposure / 機密データ漏洩の可能性
- **Mitigation:** Validate and filter environment variables / 環境変数を検証・フィルター

### Process Isolation / プロセス分離

- Subprocess runs with IDE's permissions / サブプロセスはIDEの権限で実行
- **Risk:** Script can access IDE resources / スクリプトはIDEリソースにアクセス可能
- **Mitigation:** Consider sandboxing / サンドボックス化を検討

---

## Future Enhancements / 将来の拡張

### Planned Features / 計画中の機能

1. **Progress tracking**
   - 進捗追跡
   - Percentage completion / 完了パーセンテージ
   - ETA calculation / 完了予想時間計算

2. **Script pause/resume**
   - スクリプトの一時停止・再開
   - SIGSTOP/SIGCONT signals / SIGSTOP/SIGCONTシグナル

3. **Output filtering**
   - 出力フィルタリング
   - Regex-based line filtering / 正規表現ベースの行フィルタリング
   - Log level detection / ログレベル検出

4. **Resource monitoring**
   - リソース監視
   - CPU usage tracking / CPU使用率追跡
   - Memory usage tracking / メモリ使用量追跡

5. **Script queuing**
   - スクリプトキューイング
   - Max concurrent process limit / 最大同時プロセス制限
   - Priority-based execution / 優先度ベースの実行

6. **Execution history**
   - 実行履歴
   - Save past execution results / 過去の実行結果を保存
   - Execution statistics / 実行統計

---

## Dependencies / 依存関係

### Runtime Dependencies / ランタイム依存関係

- **sikulix CLI (runtime-rs):** Must be in PATH
  - PATH内に存在する必要があります
  - Version compatibility: Any / バージョン互換性：任意

### Rust Crates / Rustクレート

```toml
tokio = { version = "1", features = ["process", "io-util"] }
uuid = { version = "1", features = ["v4"] }
log = "0.4"
serde = { version = "1", features = ["derive"] }
tauri = "2"
```

---

## Known Limitations / 既知の制限事項

1. **Windows path handling**
   - Windows環境でのパス処理
   - May need special handling for UNC paths / UNCパスに特別な処理が必要な場合あり

2. **Large output buffering**
   - 大量の出力バッファリング
   - Non-streaming mode may cause OOM for very large outputs / 非ストリーミングモードは巨大な出力でOOMの可能性

3. **Process orphaning**
   - プロセスの孤立
   - If IDE crashes, subprocesses may continue / IDEがクラッシュするとサブプロセスが継続する可能性

4. **No stdin support**
   - stdin サポートなし
   - Cannot send input to running scripts / 実行中のスクリプトに入力を送信不可

---

## Changelog / 変更履歴

### v0.1.0 (2025-11-27)

- Initial implementation / 初期実装
- Added `run_script` command / `run_script`コマンド追加
- Added `run_script_streaming` command / `run_script_streaming`コマンド追加
- Added `stop_script` command / `stop_script`コマンド追加
- Added `get_running_processes` command / `get_running_processes`コマンド追加
- Added `stop_all_scripts` command / `stop_all_scripts`コマンド追加
- Process state management / プロセス状態管理
- Real-time output streaming / リアルタイム出力ストリーミング
- Timeout support / タイムアウトサポート
- Comprehensive unit tests / 包括的なユニットテスト

---

## References / 参考資料

### Related Documentation / 関連ドキュメント

- [IDE-RS-TAURI-DESIGN.md](C:\VSCode\Sikuli-D\.local\doc\spec\IDE-RS-TAURI-DESIGN.md)
- [ARCHITECTURE.md](C:\VSCode\Sikuli-D\.local\doc\spec\ARCHITECTURE.md)

### Design Specifications / 設計仕様

- Script Execution Integration: Lines 1242-1288 (IDE-RS-TAURI-DESIGN.md)
- Command Design: Lines 153-428 (IDE-RS-TAURI-DESIGN.md)

---

**END OF DOCUMENTATION / ドキュメント終了**
