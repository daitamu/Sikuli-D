//! Global hotkey module for script interruption
//! スクリプト中断用のグローバルホットキーモジュール
//!
//! Registers Shift+Alt+C as a global hotkey to stop running scripts.
//! Shift+Alt+Cをグローバルホットキーとして登録し、実行中のスクリプトを停止します。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

/// Stop signal shared between hotkey handler and script execution
/// ホットキーハンドラとスクリプト実行間で共有される停止シグナル
pub struct StopSignal {
    /// Whether stop was requested / 停止が要求されたかどうか
    stop_requested: AtomicBool,
    /// Whether running from IDE (IDE handles stop) / IDEから実行中か（IDEが停止を処理）
    from_ide: AtomicBool,
}

impl StopSignal {
    /// Create a new stop signal
    pub fn new() -> Self {
        Self {
            stop_requested: AtomicBool::new(false),
            from_ide: AtomicBool::new(false),
        }
    }

    /// Check if stop was requested
    /// 停止が要求されたかチェック
    pub fn is_stop_requested(&self) -> bool {
        self.stop_requested.load(Ordering::SeqCst)
    }

    /// Request stop
    /// 停止を要求
    pub fn request_stop(&self) {
        self.stop_requested.store(true, Ordering::SeqCst);
    }

    /// Set whether running from IDE
    /// IDEから実行中かを設定
    pub fn set_from_ide(&self, from_ide: bool) {
        self.from_ide.store(from_ide, Ordering::SeqCst);
    }

    /// Check if running from IDE
    /// IDEから実行中かチェック
    #[allow(dead_code)]
    pub fn is_from_ide(&self) -> bool {
        self.from_ide.load(Ordering::SeqCst)
    }
}

impl Default for StopSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Global hotkey manager for Shift+Alt+C
/// Shift+Alt+C用のグローバルホットキーマネージャ
pub struct HotkeyManager {
    manager: GlobalHotKeyManager,
    hotkey: HotKey,
    #[allow(dead_code)]
    stop_signal: Arc<StopSignal>,
}

impl HotkeyManager {
    /// Create and register the Shift+Alt+C hotkey
    /// Shift+Alt+Cホットキーを作成して登録
    pub fn new(stop_signal: Arc<StopSignal>) -> anyhow::Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| anyhow::anyhow!("Failed to create hotkey manager: {}", e))?;

        // Shift+Alt+C
        let hotkey = HotKey::new(Some(Modifiers::SHIFT | Modifiers::ALT), Code::KeyC);

        manager
            .register(hotkey)
            .map_err(|e| anyhow::anyhow!("Failed to register Shift+Alt+C hotkey: {}", e))?;

        log::info!("Registered Shift+Alt+C hotkey for script interruption");

        Ok(Self {
            manager,
            hotkey,
            stop_signal,
        })
    }

    /// Process hotkey events (call this in a loop or event handler)
    /// ホットキーイベントを処理（ループまたはイベントハンドラで呼び出す）
    #[allow(dead_code)]
    pub fn process_events(&self) {
        if let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.id == self.hotkey.id() {
                log::warn!("Shift+Alt+C pressed - requesting script stop");
                self.stop_signal.request_stop();

                // If not running from IDE, print message directly
                // IDEから実行していない場合は直接メッセージを表示
                if !self.stop_signal.is_from_ide() {
                    eprintln!("\n=== Script interrupted by Shift+Alt+C ===");
                    eprintln!("=== Shift+Alt+Cによりスクリプトが中断されました ===\n");
                }
            }
        }
    }

    /// Unregister the hotkey
    /// ホットキーを登録解除
    pub fn unregister(&self) {
        if let Err(e) = self.manager.unregister(self.hotkey) {
            log::warn!("Failed to unregister hotkey: {}", e);
        } else {
            log::debug!("Unregistered Shift+Alt+C hotkey");
        }
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        self.unregister();
    }
}

/// Check if the SIKULID_FROM_IDE environment variable is set
/// SIKULID_FROM_IDE環境変数が設定されているかチェック
pub fn is_running_from_ide() -> bool {
    std::env::var("SIKULID_FROM_IDE").is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_signal() {
        let signal = StopSignal::new();
        assert!(!signal.is_stop_requested());
        signal.request_stop();
        assert!(signal.is_stop_requested());
    }

    #[test]
    fn test_from_ide_flag() {
        let signal = StopSignal::new();
        assert!(!signal.is_from_ide());
        signal.set_from_ide(true);
        assert!(signal.is_from_ide());
    }
}
