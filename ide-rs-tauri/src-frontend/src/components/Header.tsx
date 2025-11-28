import { Play, Square, Settings, LayoutList, GitBranch, FolderOpen, Save, FilePlus, Code } from 'lucide-react'
import type { ViewMode } from '../types/script'

interface HeaderProps {
  viewMode: ViewMode
  onViewModeChange: (mode: ViewMode) => void
  isRunning: boolean
  onRun: () => void
  onStop: () => void
  onNewFile: () => void
  onOpenFile: () => void
  onSaveFile: () => void
  currentFile: string | null
  isSimpleModeAvailable?: boolean  // Whether Simple/Flow modes are available
  hideMode: boolean  // Whether to hide IDE when script runs
  onHideModeChange: (hide: boolean) => void
  version: string  // IDE version from Rust backend
}

/**
 * Header Component - Application title, mode switch, and controls
 */
export function Header({
  viewMode,
  onViewModeChange,
  isRunning,
  onRun,
  onStop,
  onNewFile,
  onOpenFile,
  onSaveFile,
  currentFile,
  isSimpleModeAvailable = true,
  hideMode,
  onHideModeChange,
  version,
}: HeaderProps) {
  return (
    <header className="h-12 bg-dark-surface border-b border-dark-border flex items-center justify-between px-4 drag">
      {/* App Title & File Controls */}
      <div className="flex items-center gap-4 no-drag">
        <div className="flex items-center gap-2">
          <h1 className="text-lg font-semibold text-sikuli-400">Sikuli-D IDE</h1>
          <span className="text-xs text-gray-500">v{version}</span>
        </div>

        {/* File Operations */}
        <div className="flex items-center gap-1 border-l border-dark-border pl-3">
          <button
            onClick={onNewFile}
            className="p-2 text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-lg transition-colors"
            title="New File"
          >
            <FilePlus size={18} />
          </button>
          <button
            onClick={onOpenFile}
            className="p-2 text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-lg transition-colors"
            title="Open File"
          >
            <FolderOpen size={18} />
          </button>
          <button
            onClick={onSaveFile}
            className="p-2 text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-lg transition-colors"
            title="Save File"
          >
            <Save size={18} />
          </button>
        </div>

        {/* Current File Name */}
        {currentFile && (
          <span className="text-sm text-gray-400 truncate max-w-48" title={currentFile}>
            {currentFile.split(/[/\\]/).pop()}
          </span>
        )}
      </div>

      {/* Center Controls */}
      <div className="flex items-center gap-4 no-drag">
        {/* View Mode Toggle */}
        <div className="flex items-center bg-dark-bg rounded-lg p-1">
          <button
            onClick={() => isSimpleModeAvailable && onViewModeChange('simple')}
            disabled={!isSimpleModeAvailable}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md transition-colors ${
              viewMode === 'simple'
                ? 'bg-sikuli-600 text-white'
                : !isSimpleModeAvailable
                  ? 'text-gray-600 cursor-not-allowed'
                  : 'text-gray-400 hover:text-gray-200'
            }`}
            title={isSimpleModeAvailable ? 'Simple Mode' : 'Simple Mode (unavailable for external scripts)'}
          >
            <LayoutList size={16} />
            <span className="text-sm">Simple</span>
          </button>
          <button
            onClick={() => isSimpleModeAvailable && onViewModeChange('flow')}
            disabled={!isSimpleModeAvailable}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md transition-colors ${
              viewMode === 'flow'
                ? 'bg-sikuli-600 text-white'
                : !isSimpleModeAvailable
                  ? 'text-gray-600 cursor-not-allowed'
                  : 'text-gray-400 hover:text-gray-200'
            }`}
            title={isSimpleModeAvailable ? 'Flow Mode' : 'Flow Mode (unavailable for external scripts)'}
          >
            <GitBranch size={16} />
            <span className="text-sm">Flow</span>
          </button>
          <button
            onClick={() => onViewModeChange('code')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md transition-colors ${
              viewMode === 'code'
                ? 'bg-sikuli-600 text-white'
                : 'text-gray-400 hover:text-gray-200'
            }`}
            title="Code Mode"
          >
            <Code size={16} />
            <span className="text-sm">Code</span>
          </button>
        </div>

        {/* Run/Stop Button */}
        {isRunning ? (
          <button
            onClick={onStop}
            className="flex items-center gap-2 px-4 py-1.5 bg-red-600 hover:bg-red-700 rounded-lg transition-colors"
            title="Stop Script (Shift+Alt+C)"
          >
            <Square size={16} />
            <span className="text-sm font-medium">Stop</span>
          </button>
        ) : (
          <button
            onClick={onRun}
            className="flex items-center gap-2 px-4 py-1.5 bg-green-600 hover:bg-green-700 rounded-lg transition-colors"
            title="Run Script"
          >
            <Play size={16} />
            <span className="text-sm font-medium">Run</span>
          </button>
        )}

        {/* Hide Mode Checkbox */}
        <label
          className="flex items-center gap-2 cursor-pointer select-none"
          title="Hide IDE window when script runs (Shift+Alt+C to stop and restore)"
        >
          <input
            type="checkbox"
            checked={hideMode}
            onChange={(e) => onHideModeChange(e.target.checked)}
            className="w-4 h-4 rounded border-gray-500 bg-dark-bg text-sikuli-500 focus:ring-sikuli-500 focus:ring-offset-dark-bg cursor-pointer"
          />
          <span className="text-sm text-gray-400">Hide</span>
        </label>
      </div>

      {/* Right Controls */}
      <div className="flex items-center gap-2 no-drag">
        <button
          className="p-2 text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-lg transition-colors"
          title="Settings"
        >
          <Settings size={18} />
        </button>
      </div>
    </header>
  )
}
