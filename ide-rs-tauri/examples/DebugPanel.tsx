/**
 * Debug Panel Component Example
 * デバッグパネルコンポーネント例
 *
 * This is a reference implementation of a debug panel for the SikuliX IDE.
 * SikuliX IDE用デバッグパネルのリファレンス実装です。
 *
 * Usage in your IDE:
 * 1. Import this component
 * 2. Add it to your IDE layout
 * 3. Connect to debug events from Tauri
 */

import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type {
  DebugState,
  VariableInfo,
  CallFrame,
  BreakpointInfo,
  DebugEvent,
} from '../types/debug';

/**
 * Debug Panel Props
 * デバッグパネルプロップス
 */
interface DebugPanelProps {
  /** Current script path / 現在のスクリプトパス */
  scriptPath: string | null;
  /** Whether debugging is active / デバッグが有効かどうか */
  isActive: boolean;
  /** Callback when line should be highlighted / 行をハイライトすべき時のコールバック */
  onHighlightLine?: (file: string, line: number) => void;
}

/**
 * Debug Panel Component
 * デバッグパネルコンポーネント
 */
export function DebugPanel({
  scriptPath,
  isActive,
  onHighlightLine,
}: DebugPanelProps) {
  // State management / 状態管理
  const [debugState, setDebugState] = useState<DebugState>('notStarted' as DebugState);
  const [breakpoints, setBreakpoints] = useState<BreakpointInfo[]>([]);
  const [variables, setVariables] = useState<VariableInfo[]>([]);
  const [callStack, setCallStack] = useState<CallFrame[]>([]);
  const [currentPosition, setCurrentPosition] = useState<[string, number] | null>(null);
  const [watchExpression, setWatchExpression] = useState('');
  const [watchResult, setWatchResult] = useState('');

  // Listen to debug events / デバッグイベントをリッスン
  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    const setupListener = async () => {
      unlisten = await listen<DebugEvent>('debug-event', (event) => {
        handleDebugEvent(event.payload);
      });
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []);

  // Initialize debug session when active / アクティブ時にデバッグセッションを初期化
  useEffect(() => {
    if (isActive && scriptPath) {
      initializeDebugSession();
    } else if (!isActive) {
      endDebugSession();
    }
  }, [isActive, scriptPath]);

  /**
   * Initialize debug session
   * デバッグセッションを初期化
   */
  const initializeDebugSession = async () => {
    if (!scriptPath) return;

    try {
      await invoke('debug_init_session', { scriptPath });
      await refreshDebugInfo();
    } catch (error) {
      console.error('Failed to initialize debug session:', error);
    }
  };

  /**
   * End debug session
   * デバッグセッションを終了
   */
  const endDebugSession = async () => {
    try {
      await invoke('debug_end_session');
      setDebugState('notStarted' as DebugState);
      setBreakpoints([]);
      setVariables([]);
      setCallStack([]);
      setCurrentPosition(null);
    } catch (error) {
      console.error('Failed to end debug session:', error);
    }
  };

  /**
   * Handle debug events
   * デバッグイベントを処理
   */
  const handleDebugEvent = (event: DebugEvent) => {
    console.log('Debug event:', event);

    switch (event.type) {
      case 'breakpointHit':
      case 'paused':
      case 'stepCompleted':
        setCurrentPosition([event.file, event.line]);
        if (onHighlightLine) {
          onHighlightLine(event.file, event.line);
        }
        refreshDebugInfo();
        break;

      case 'resumed':
        setCurrentPosition(null);
        break;

      case 'stopped':
        setDebugState('stopped' as DebugState);
        setCurrentPosition(null);
        break;

      case 'error':
        console.error('Debug error:', event.message);
        alert(`Debug error: ${event.message}`);
        break;

      case 'variableChanged':
        refreshVariables();
        break;
    }
  };

  /**
   * Refresh all debug information
   * すべてのデバッグ情報を更新
   */
  const refreshDebugInfo = async () => {
    await Promise.all([
      refreshState(),
      refreshBreakpoints(),
      refreshVariables(),
      refreshCallStack(),
      refreshPosition(),
    ]);
  };

  const refreshState = async () => {
    try {
      const state = await invoke<DebugState>('debug_get_state');
      setDebugState(state);
    } catch (error) {
      console.error('Failed to get debug state:', error);
    }
  };

  const refreshBreakpoints = async () => {
    try {
      const bps = await invoke<BreakpointInfo[]>('debug_list_breakpoints');
      setBreakpoints(bps);
    } catch (error) {
      console.error('Failed to get breakpoints:', error);
    }
  };

  const refreshVariables = async () => {
    try {
      const vars = await invoke<VariableInfo[]>('debug_get_variables', {
        scope: 'all',
      });
      setVariables(vars);
    } catch (error) {
      console.error('Failed to get variables:', error);
    }
  };

  const refreshCallStack = async () => {
    try {
      const stack = await invoke<CallFrame[]>('debug_get_call_stack');
      setCallStack(stack);
    } catch (error) {
      console.error('Failed to get call stack:', error);
    }
  };

  const refreshPosition = async () => {
    try {
      const pos = await invoke<[string, number] | null>('debug_get_current_position');
      setCurrentPosition(pos);
    } catch (error) {
      console.error('Failed to get current position:', error);
    }
  };

  /**
   * Toggle breakpoint at current line
   * 現在の行でブレークポイントを切り替え
   */
  const handleToggleBreakpoint = async (file: string, line: number) => {
    try {
      const isSet = await invoke<boolean>('debug_toggle_breakpoint', {
        file,
        line,
      });
      console.log(`Breakpoint ${isSet ? 'set' : 'removed'} at ${file}:${line}`);
      await refreshBreakpoints();
    } catch (error) {
      console.error('Failed to toggle breakpoint:', error);
    }
  };

  /**
   * Execute debug commands
   * デバッグコマンドを実行
   */
  const handlePause = async () => {
    try {
      await invoke('debug_pause');
    } catch (error) {
      console.error('Failed to pause:', error);
    }
  };

  const handleResume = async () => {
    try {
      await invoke('debug_resume');
    } catch (error) {
      console.error('Failed to resume:', error);
    }
  };

  const handleStepOver = async () => {
    try {
      await invoke('debug_step_over');
    } catch (error) {
      console.error('Failed to step over:', error);
    }
  };

  const handleStepInto = async () => {
    try {
      await invoke('debug_step_into');
    } catch (error) {
      console.error('Failed to step into:', error);
    }
  };

  const handleStepOut = async () => {
    try {
      await invoke('debug_step_out');
    } catch (error) {
      console.error('Failed to step out:', error);
    }
  };

  const handleStop = async () => {
    try {
      await invoke('debug_stop');
    } catch (error) {
      console.error('Failed to stop:', error);
    }
  };

  /**
   * Evaluate watch expression
   * ウォッチ式を評価
   */
  const handleEvaluateExpression = async () => {
    if (!watchExpression.trim()) return;

    try {
      const result = await invoke<string>('debug_evaluate_expression', {
        expr: watchExpression,
      });
      setWatchResult(result);
    } catch (error: any) {
      setWatchResult(`Error: ${error}`);
    }
  };

  // UI Rendering / UI描画
  return (
    <div className="debug-panel">
      {/* Toolbar / ツールバー */}
      <div className="debug-toolbar">
        <button
          onClick={handleResume}
          disabled={debugState !== 'paused'}
          title="Resume / 再開 (F5)"
        >
          ▶️
        </button>
        <button
          onClick={handlePause}
          disabled={debugState !== 'running'}
          title="Pause / 一時停止"
        >
          ⏸️
        </button>
        <button
          onClick={handleStepOver}
          disabled={debugState !== 'paused'}
          title="Step Over / ステップオーバー (F10)"
        >
          ⏭️
        </button>
        <button
          onClick={handleStepInto}
          disabled={debugState !== 'paused'}
          title="Step Into / ステップイン (F11)"
        >
          ⬇️
        </button>
        <button
          onClick={handleStepOut}
          disabled={debugState !== 'paused'}
          title="Step Out / ステップアウト (Shift+F11)"
        >
          ⬆️
        </button>
        <button onClick={handleStop} title="Stop / 停止 (Shift+F5)">
          ⏹️
        </button>
        <span className="debug-state">State: {debugState}</span>
      </div>

      {/* Call Stack / コールスタック */}
      <div className="debug-section">
        <h3>Call Stack</h3>
        <div className="call-stack">
          {callStack.length === 0 ? (
            <p className="empty">No call stack</p>
          ) : (
            <ul>
              {callStack.map((frame, index) => (
                <li
                  key={index}
                  className={frame.depth === 0 ? 'current' : ''}
                  onClick={() => onHighlightLine?.(frame.file, frame.line)}
                >
                  {frame.depth === 0 && '▶ '}
                  <strong>{frame.function}</strong> at {frame.file}:
                  {frame.line}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Variables / 変数 */}
      <div className="debug-section">
        <h3>Variables</h3>
        <div className="variables">
          {variables.length === 0 ? (
            <p className="empty">No variables</p>
          ) : (
            <table>
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Value</th>
                  <th>Type</th>
                  <th>Scope</th>
                </tr>
              </thead>
              <tbody>
                {variables.map((variable, index) => (
                  <tr key={index}>
                    <td className="var-name">{variable.name}</td>
                    <td className="var-value">{variable.value}</td>
                    <td className="var-type">{variable.typeName}</td>
                    <td className="var-scope">{variable.scope}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Watch Expression / ウォッチ式 */}
      <div className="debug-section">
        <h3>Watch</h3>
        <div className="watch-expression">
          <input
            type="text"
            value={watchExpression}
            onChange={(e) => setWatchExpression(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleEvaluateExpression()}
            placeholder="Enter expression to evaluate"
          />
          <button onClick={handleEvaluateExpression}>Evaluate</button>
        </div>
        {watchResult && (
          <div className="watch-result">
            <strong>Result:</strong> {watchResult}
          </div>
        )}
      </div>

      {/* Breakpoints / ブレークポイント */}
      <div className="debug-section">
        <h3>
          Breakpoints
          <button
            onClick={async () => {
              await invoke('debug_clear_breakpoints');
              await refreshBreakpoints();
            }}
            className="small"
          >
            Clear All
          </button>
        </h3>
        <div className="breakpoints">
          {breakpoints.length === 0 ? (
            <p className="empty">No breakpoints</p>
          ) : (
            <ul>
              {breakpoints.map((bp, index) => (
                <li
                  key={index}
                  onClick={() => handleToggleBreakpoint(bp.file, bp.line)}
                >
                  ● {bp.file}:{bp.line}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Current Position / 現在の位置 */}
      {currentPosition && (
        <div className="debug-section current-position">
          <strong>Current:</strong> {currentPosition[0]}:{currentPosition[1]}
        </div>
      )}
    </div>
  );
}

/**
 * Example CSS styles
 * CSSスタイル例
 *
 * Add these styles to your stylesheet or use CSS-in-JS:
 * これらのスタイルをスタイルシートに追加するか、CSS-in-JSを使用してください:
 */
export const debugPanelStyles = `
.debug-panel {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 1rem;
  background: #1e1e1e;
  color: #d4d4d4;
  height: 100%;
  overflow-y: auto;
}

.debug-toolbar {
  display: flex;
  gap: 0.5rem;
  align-items: center;
  padding: 0.5rem;
  background: #252526;
  border-radius: 4px;
}

.debug-toolbar button {
  padding: 0.5rem;
  background: #0e639c;
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
  font-size: 1rem;
}

.debug-toolbar button:hover:not(:disabled) {
  background: #1177bb;
}

.debug-toolbar button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.debug-state {
  margin-left: auto;
  padding: 0.25rem 0.5rem;
  background: #3c3c3c;
  border-radius: 4px;
  font-size: 0.875rem;
}

.debug-section {
  background: #252526;
  border-radius: 4px;
  padding: 1rem;
}

.debug-section h3 {
  margin: 0 0 0.5rem 0;
  font-size: 0.875rem;
  color: #cccccc;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.debug-section button.small {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: #3c3c3c;
}

.call-stack ul,
.breakpoints ul {
  list-style: none;
  padding: 0;
  margin: 0;
}

.call-stack li,
.breakpoints li {
  padding: 0.5rem;
  margin-bottom: 0.25rem;
  background: #3c3c3c;
  border-radius: 4px;
  cursor: pointer;
  font-size: 0.875rem;
}

.call-stack li:hover,
.breakpoints li:hover {
  background: #4c4c4c;
}

.call-stack li.current {
  background: #094771;
}

.variables table {
  width: 100%;
  border-collapse: collapse;
  font-size: 0.875rem;
}

.variables th,
.variables td {
  text-align: left;
  padding: 0.5rem;
  border-bottom: 1px solid #3c3c3c;
}

.variables th {
  background: #3c3c3c;
  font-weight: 600;
}

.var-value {
  font-family: monospace;
  color: #ce9178;
}

.var-type {
  color: #4ec9b0;
}

.var-scope {
  color: #9cdcfe;
  font-size: 0.75rem;
}

.watch-expression {
  display: flex;
  gap: 0.5rem;
  margin-bottom: 0.5rem;
}

.watch-expression input {
  flex: 1;
  padding: 0.5rem;
  background: #3c3c3c;
  border: 1px solid #4c4c4c;
  border-radius: 4px;
  color: #d4d4d4;
  font-size: 0.875rem;
}

.watch-expression button {
  padding: 0.5rem 1rem;
  background: #0e639c;
  border: none;
  border-radius: 4px;
  color: white;
  cursor: pointer;
}

.watch-result {
  padding: 0.5rem;
  background: #3c3c3c;
  border-radius: 4px;
  font-size: 0.875rem;
  font-family: monospace;
}

.empty {
  color: #6a737d;
  font-style: italic;
  font-size: 0.875rem;
}

.current-position {
  background: #094771;
  font-size: 0.875rem;
}
`;
