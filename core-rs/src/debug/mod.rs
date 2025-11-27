//! Debug infrastructure for script debugging and visual highlighting
//! スクリプトデバッグとビジュアルハイライト用デバッグ基盤
//!
//! This module provides comprehensive debugging capabilities including:
//! このモジュールは以下を含む包括的なデバッグ機能を提供します:
//! - Breakpoint management with conditions / 条件付きブレークポイント管理
//! - Execution control (pause, resume, step) / 実行制御（一時停止、再開、ステップ）
//! - Variable inspection and evaluation / 変数インスペクションと評価
//! - Call stack tracking / コールスタック追跡
//! - Event notification system / イベント通知システム
//! - Visual highlight overlays for screen regions / 画面領域のビジュアルハイライトオーバーレイ

mod debugger;
pub mod highlight;
pub mod highlight_linux;

// Re-export debugger types
// デバッガ型を再エクスポート
pub use debugger::{
    CallFrame, DebugEvent, DebugState, Debugger, Scope, VariableInfo, VariableValue,
};

// Re-export highlight functionality
// ハイライト機能を再エクスポート
pub use highlight::{
    highlight, highlight_match, show_highlight_with_config, HighlightConfig,
};


#[cfg(test)]
mod tests;
