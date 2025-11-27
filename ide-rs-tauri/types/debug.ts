/**
 * Debug panel TypeScript types
 * デバッグパネルTypeScript型定義
 *
 * These types mirror the Rust data structures in ide-rs-tauri/src/debug.rs
 * これらの型はide-rs-tauri/src/debug.rsのRustデータ構造を反映しています
 */

/**
 * Debug state for frontend
 * フロントエンド用デバッグ状態
 */
export enum DebugState {
  NotStarted = 'notStarted',
  Running = 'running',
  Paused = 'paused',
  StepOver = 'stepOver',
  StepInto = 'stepInto',
  StepOut = 'stepOut',
  Stopped = 'stopped',
  Error = 'error',
}

/**
 * Variable scope
 * 変数スコープ
 */
export type VariableScope = 'local' | 'global' | 'all';

/**
 * Variable information for debugging
 * デバッグ用変数情報
 */
export interface VariableInfo {
  /** Variable name / 変数名 */
  name: string;
  /** Variable value (as string) / 変数値（文字列） */
  value: string;
  /** Type name / 型名 */
  typeName: string;
  /** Scope / スコープ */
  scope: VariableScope;
}

/**
 * Call stack frame
 * コールスタックフレーム
 */
export interface CallFrame {
  /** Frame depth (0 = current) / フレーム深度（0 = 現在） */
  depth: number;
  /** Function or method name / 関数またはメソッド名 */
  function: string;
  /** File path / ファイルパス */
  file: string;
  /** Line number / 行番号 */
  line: number;
}

/**
 * Breakpoint information
 * ブレークポイント情報
 */
export interface BreakpointInfo {
  /** File path / ファイルパス */
  file: string;
  /** Line number / 行番号 */
  line: number;
}

/**
 * Debug event types
 * デバッグイベント型
 */
export type DebugEvent =
  | {
      type: 'breakpointHit';
      file: string;
      line: number;
      hitCount: number;
    }
  | {
      type: 'paused';
      file: string;
      line: number;
    }
  | {
      type: 'resumed';
    }
  | {
      type: 'stepCompleted';
      file: string;
      line: number;
    }
  | {
      type: 'stopped';
    }
  | {
      type: 'error';
      message: string;
    }
  | {
      type: 'variableChanged';
      name: string;
      value: string;
    };

/**
 * Debug commands interface
 * デバッグコマンドインターフェース
 *
 * These functions correspond to the Tauri commands in debug.rs
 * これらの関数はdebug.rsのTauriコマンドに対応しています
 */
export interface DebugCommands {
  /**
   * Initialize debug session
   * デバッグセッションを初期化
   */
  debugInitSession(scriptPath: string): Promise<void>;

  /**
   * End debug session
   * デバッグセッションを終了
   */
  debugEndSession(): Promise<void>;

  /**
   * Add a breakpoint
   * ブレークポイントを追加
   */
  debugAddBreakpoint(file: string, line: number): Promise<void>;

  /**
   * Remove a breakpoint
   * ブレークポイントを削除
   */
  debugRemoveBreakpoint(file: string, line: number): Promise<void>;

  /**
   * Toggle breakpoint (add if not exists, remove if exists)
   * ブレークポイントを切り替え（存在しない場合は追加、存在する場合は削除）
   */
  debugToggleBreakpoint(file: string, line: number): Promise<boolean>;

  /**
   * List all breakpoints
   * すべてのブレークポイントをリスト
   */
  debugListBreakpoints(): Promise<BreakpointInfo[]>;

  /**
   * Clear all breakpoints
   * すべてのブレークポイントをクリア
   */
  debugClearBreakpoints(): Promise<void>;

  /**
   * Pause execution
   * 実行を一時停止
   */
  debugPause(): Promise<void>;

  /**
   * Resume execution
   * 実行を再開
   */
  debugResume(): Promise<void>;

  /**
   * Step over current line
   * 現在の行をステップオーバー
   */
  debugStepOver(): Promise<void>;

  /**
   * Step into function
   * 関数にステップイン
   */
  debugStepInto(): Promise<void>;

  /**
   * Step out of current function
   * 現在の関数からステップアウト
   */
  debugStepOut(): Promise<void>;

  /**
   * Stop execution
   * 実行を停止
   */
  debugStop(): Promise<void>;

  /**
   * Get current debug state
   * 現在のデバッグ状態を取得
   */
  debugGetState(): Promise<DebugState>;

  /**
   * Get variables in the specified scope
   * 指定されたスコープの変数を取得
   */
  debugGetVariables(scope?: VariableScope): Promise<VariableInfo[]>;

  /**
   * Get call stack
   * コールスタックを取得
   */
  debugGetCallStack(): Promise<CallFrame[]>;

  /**
   * Get current execution position
   * 現在の実行位置を取得
   */
  debugGetCurrentPosition(): Promise<[string, number] | null>;

  /**
   * Evaluate expression in current context
   * 現在のコンテキストで式を評価
   */
  debugEvaluateExpression(expr: string): Promise<string>;
}

/**
 * Debug event listener callback type
 * デバッグイベントリスナーコールバック型
 */
export type DebugEventCallback = (event: DebugEvent) => void;
