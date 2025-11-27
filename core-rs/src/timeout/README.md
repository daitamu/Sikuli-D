# Timeout Handling Module
# タイムアウト処理モジュール

## Overview / 概要

This module provides comprehensive timeout management for SikuliX operations in the Sikuli-D project.

このモジュールは、Sikuli-D プロジェクトの SikuliX 操作のための包括的なタイムアウト管理を提供します。

## Features / 機能

### 1. Synchronous Timeout / 同期タイムアウト

Execute synchronous operations with timeout protection.

同期操作をタイムアウト保護付きで実行します。

```rust
use sikulix_core::timeout::with_timeout;
use std::time::Duration;

let result = with_timeout(Duration::from_secs(5), || {
    // Your operation here
    // ここに操作を記述
    Ok(42)
});
```

### 2. Async Timeout / 非同期タイムアウト

Execute async operations with timeout (requires `async` feature).

非同期操作をタイムアウト付きで実行します（`async`機能が必要）。

```rust
use sikulix_core::timeout::async::with_timeout_async;
use std::time::Duration;

let result = with_timeout_async(Duration::from_secs(5), async {
    // Your async operation here
    // ここに非同期操作を記述
    Ok(42)
}).await;
```

### 3. Cancellation Token / キャンセルトークン

Cancel long-running operations gracefully.

長時間実行操作を優雅にキャンセルします。

```rust
use sikulix_core::timeout::CancellationToken;
use std::thread;
use std::time::Duration;

let token = CancellationToken::new();
let token_clone = token.clone();

// Spawn operation
// 操作を起動
let handle = thread::spawn(move || {
    for i in 0..100 {
        if token_clone.is_cancelled() {
            return Err("Cancelled");
        }
        // Do work...
        // 作業を実行...
        thread::sleep(Duration::from_millis(100));
    }
    Ok("Completed")
});

// Cancel after 2 seconds
// 2秒後にキャンセル
thread::sleep(Duration::from_secs(2));
token.cancel();

let result = handle.join().unwrap();
```

### 4. Wait for Condition / 条件待機

Wait for a condition to become true with timeout.

条件が真になるまでタイムアウト付きで待機します。

```rust
use sikulix_core::timeout::wait_for_condition;
use std::time::Duration;

let mut counter = 0;
let result = wait_for_condition(
    Duration::from_secs(5),
    Duration::from_millis(100),
    || {
        counter += 1;
        counter >= 10
    }
);
```

### 5. Timeout Guard / タイムアウトガード

RAII-style timeout guard for automatic timeout checking.

自動タイムアウトチェック用のRAIIスタイルタイムアウトガード。

```rust
use sikulix_core::timeout::TimeoutGuard;
use std::time::Duration;

let guard = TimeoutGuard::new(Duration::from_secs(10));

loop {
    if guard.is_expired() {
        break;
    }
    // Do work...
    // 作業を実行...
}
```

### 6. Default Timeouts / デフォルトタイムアウト

Pre-configured timeout values for common operations.

一般的な操作のための事前設定済みタイムアウト値。

```rust
use sikulix_core::timeout::DefaultTimeouts;

let timeouts = DefaultTimeouts::new();
println!("Find timeout: {:?}", timeouts.get_find());
println!("Wait timeout: {:?}", timeouts.get_wait());
println!("Script timeout: {:?}", timeouts.get_script());
```

## Integration with ImageMatcher / ImageMatcherとの統合

The timeout module is integrated with `ImageMatcher` for cancellable image search operations.

タイムアウトモジュールは、キャンセル可能な画像検索操作のために `ImageMatcher` と統合されています。

```rust
use sikulix_core::{ImageMatcher, Pattern, Screen, CancellationToken};

let matcher = ImageMatcher::new();
let screen = Screen::primary();
let pattern = Pattern::from_file("button.png")?;
let token = CancellationToken::new();

// Wait with cancellation support
// キャンセルサポート付きで待機
let result = matcher.wait_with_cancel(&screen, &pattern, 5.0, &token);

// In another thread, you can cancel the operation
// 別のスレッドで操作をキャンセル可能
// token.cancel();
```

## Error Types / エラー型

The module introduces several timeout-related error types:

モジュールは複数のタイムアウト関連エラー型を導入します：

- `SikulixError::Timeout` - Generic operation timeout
  汎用操作タイムアウト
- `SikulixError::Cancelled` - Operation was cancelled
  操作がキャンセルされた
- `SikulixError::WaitTimeout` - Wait condition not met
  待機条件が満たされなかった
- `SikulixError::ScriptTimeout` - Script execution timeout
  スクリプト実行タイムアウト
- `SikulixError::FindFailed` - Pattern not found within timeout
  タイムアウト内にパターンが見つからなかった

## Default Timeout Values / デフォルトタイムアウト値

| Operation | Default | Description |
|-----------|---------|-------------|
| find | 3s | Pattern find timeout |
| wait | 3s | Pattern wait timeout |
| exists | 0s | Pattern exists check (immediate) |
| script | 10m | Script execution timeout |
| screen_capture | 5s | Screen capture timeout |
| ocr | 30s | OCR operation timeout |

## Usage Examples / 使用例

### Example 1: Basic Timeout

```rust
use sikulix_core::timeout::with_timeout;
use std::time::Duration;

fn expensive_operation() -> Result<String, String> {
    // Simulate work...
    // 作業をシミュレート...
    std::thread::sleep(Duration::from_secs(2));
    Ok("Done".to_string())
}

let result = with_timeout(Duration::from_secs(5), || {
    expensive_operation().map_err(|e|
        sikulix_core::SikulixError::PlatformError(e)
    )
});
```

### Example 2: Cancellable Long Operation

```rust
use sikulix_core::timeout::{with_timeout_and_cancel, CancellationToken};
use std::time::Duration;

let token = CancellationToken::new();

let result = with_timeout_and_cancel(
    Duration::from_secs(60),
    token.clone(),
    |token| {
        for i in 0..1000 {
            if token.is_cancelled() {
                return Err(sikulix_core::SikulixError::Cancelled(
                    "Operation cancelled".to_string()
                ));
            }
            // Do work...
            // 作業を実行...
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }
);
```

### Example 3: Image Search with Cancellation

```rust
use sikulix_core::{ImageMatcher, Pattern, Screen, CancellationToken};

let matcher = ImageMatcher::new();
let screen = Screen::primary();
let pattern = Pattern::from_file("button.png")?;
let token = CancellationToken::new();
let token_for_thread = token.clone();

// Spawn cancellation thread
// キャンセルスレッドを起動
std::thread::spawn(move || {
    std::thread::sleep(Duration::from_secs(3));
    token_for_thread.cancel();
    println!("Search cancelled!");
});

// Wait for pattern with cancellation
// キャンセル付きでパターンを待機
match matcher.wait_with_cancel(&screen, &pattern, 10.0, &token) {
    Ok(m) => println!("Found at {:?}", m.center()),
    Err(sikulix_core::SikulixError::Cancelled(_)) => {
        println!("Search was cancelled");
    }
    Err(e) => println!("Error: {}", e),
}
```

## Thread Safety / スレッドセーフティ

All timeout utilities are thread-safe:

すべてのタイムアウトユーティリティはスレッドセーフです：

- `CancellationToken` uses `Arc<AtomicBool>` for safe sharing across threads
  `CancellationToken`はスレッド間で安全に共有するために`Arc<AtomicBool>`を使用
- `TimeoutGuard` can be safely used in multi-threaded contexts
  `TimeoutGuard`はマルチスレッドコンテキストで安全に使用可能
- All timeout functions handle thread synchronization internally
  すべてのタイムアウト関数は内部でスレッド同期を処理

## Testing / テスト

Run the timeout module tests:

タイムアウトモジュールのテストを実行：

```bash
cargo test --lib timeout
```

Run specific tests:

特定のテストを実行：

```bash
cargo test --lib timeout::tests::test_cancellation_token
cargo test --lib timeout::tests::test_with_timeout_success
cargo test --lib timeout::tests::test_timeout_guard
```

## Performance Considerations / パフォーマンス考慮事項

1. **Thread Creation Overhead / スレッド作成オーバーヘッド**:
   - `with_timeout` spawns a new thread for each call
   - Use thread pools for high-frequency operations
   - `with_timeout`は各呼び出しで新しいスレッドを起動
   - 高頻度操作にはスレッドプールを使用

2. **Polling Interval / ポーリング間隔**:
   - Default polling interval is 100ms for cancellation checks
   - Adjust based on responsiveness requirements
   - キャンセルチェックのデフォルトポーリング間隔は100ms
   - 応答性の要件に基づいて調整

3. **Resource Cleanup / リソースクリーンアップ**:
   - Ensure proper cleanup on timeout or cancellation
   - Use RAII patterns where possible
   - タイムアウトまたはキャンセル時に適切なクリーンアップを確保
   - 可能な限りRAIIパターンを使用

## Best Practices / ベストプラクティス

1. **Always Set Reasonable Timeouts / 常に適切なタイムアウトを設定**:
   - Not too short (causes false failures)
   - Not too long (poor user experience)
   - 短すぎない（誤った失敗を引き起こす）
   - 長すぎない（悪いユーザー体験）

2. **Use Cancellation Tokens for User-Facing Operations / ユーザー向け操作にはキャンセルトークンを使用**:
   - Allow users to cancel long-running operations
   - Provide feedback on cancellation
   - ユーザーが長時間実行操作をキャンセルできるようにする
   - キャンセルに関するフィードバックを提供

3. **Handle Timeout Errors Gracefully / タイムアウトエラーを優雅に処理**:
   - Provide clear error messages
   - Suggest remedial actions
   - 明確なエラーメッセージを提供
   - 是正措置を提案

4. **Log Timeout Events / タイムアウトイベントをログに記録**:
   - Use `log::warn!` for timeout events
   - Include operation context in logs
   - タイムアウトイベントには`log::warn!`を使用
   - ログに操作コンテキストを含める

## Future Enhancements / 将来の拡張

- [ ] Adaptive timeout based on historical performance
      過去のパフォーマンスに基づく適応型タイムアウト
- [ ] Timeout statistics and monitoring
      タイムアウト統計と監視
- [ ] Configurable timeout policies
      設定可能なタイムアウトポリシー
- [ ] Timeout budget for composite operations
      複合操作のためのタイムアウト予算

## See Also / 関連項目

- [ImageMatcher Documentation](../image/matcher.rs)
- [Error Recovery Strategy](../../../../.local/doc/spec/ERROR-RECOVERY-SPEC.md)
- [Runtime-RS Design](../../../../.local/doc/spec/RUNTIME-RS-DESIGN.md)
