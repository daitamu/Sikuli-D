/**
 * Script data types for SikuliX IDE
 * SikuliX IDE のスクリプトデータ型
 */

export type CommandType = 'click' | 'type' | 'wait' | 'find' | 'if' | 'loop' | 'start';

/**
 * Script line structure (tree format)
 * スクリプト行構造（ツリー形式）
 */
export interface ScriptLine {
  /** Unique identifier / 一意識別子 */
  id: string;
  /** Command type / コマンドタイプ */
  type: CommandType;
  /** Target: Base64 image or file path / ターゲット: Base64画像またはファイルパス */
  target?: string;
  /** Parameters: text input, wait time, etc. / パラメータ: テキスト入力、待機時間など */
  params?: string;
  /** Image similarity (0.0 - 1.0) / 画像一致率 (0.0 - 1.0) */
  similarity?: number;
  /** Nested blocks for If/Loop / If/Loop用のネストブロック */
  children?: ScriptLine[];
  /** UI collapse state / UI折りたたみ状態 */
  isCollapsed?: boolean;
  /** Flow mode coordinates / フローモード座標データ */
  flowConfig?: {
    x: number;
    y: number;
  };
}

/**
 * View mode types
 * ビューモードタイプ
 */
export type ViewMode = 'simple' | 'flow' | 'code';

/**
 * Application state
 * アプリケーション状態
 */
export interface AppState {
  /** Current view mode / 現在のビューモード */
  viewMode: ViewMode;
  /** Script data / スクリプトデータ */
  script: ScriptLine[];
  /** Currently selected line ID / 現在選択中の行ID */
  selectedLineId: string | null;
  /** Whether script is running / スクリプト実行中かどうか */
  isRunning: boolean;
  /** Console output / コンソール出力 */
  consoleOutput: ConsoleEntry[];
}

/**
 * Console entry
 * コンソールエントリ
 */
export interface ConsoleEntry {
  id: string;
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

/**
 * Toolbox item (draggable command)
 * ツールボックスアイテム（ドラッグ可能なコマンド）
 */
export interface ToolboxItem {
  type: CommandType;
  label: string;
  labelJa: string;
  icon: string;
  category: 'logic' | 'actions';
}
