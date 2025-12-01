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
    <header className="h-14 bg-dark-surface/80 backdrop-blur border-b border-dark-border flex items-center justify-between px-4 drag z-50">
      {/* App Title & File Controls */}
      <div className="flex items-center gap-6 no-drag">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 bg-sikuli-500 rounded-lg flex items-center justify-center shadow-lg shadow-sikuli-500/20">
             <span className="text-white font-bold text-lg">S</span>
          </div>
          <div className="flex flex-col leading-none">
            <h1 className="text-sm font-bold text-gray-200 tracking-wide">SIKULI-D</h1>
            <span className="text-[10px] text-sikuli-400 font-mono">v{version}</span>
          </div>
        </div>

        {/* File Operations */}
        <div className="flex items-center gap-1 border-l border-dark-border pl-4">
          <button
            onClick={onNewFile}
            className="p-2 text-gray-400 hover:text-gray-100 hover:bg-dark-hover rounded-md transition-all duration-200"
            title="New File"
          >
            <FilePlus size={18} strokeWidth={1.5} />
          </button>
          <button
            onClick={onOpenFile}
            className="p-2 text-gray-400 hover:text-gray-100 hover:bg-dark-hover rounded-md transition-all duration-200"
            title="Open File"
          >
            <FolderOpen size={18} strokeWidth={1.5} />
          </button>
          <button
            onClick={onSaveFile}
            className="p-2 text-gray-400 hover:text-gray-100 hover:bg-dark-hover rounded-md transition-all duration-200"
            title="Save File"
          >
            <Save size={18} strokeWidth={1.5} />
          </button>
        </div>

        {/* Current File Name */}
        {currentFile && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-dark-bg/50 rounded-md border border-dark-border/50 max-w-64">
             <span className="text-xs text-gray-400 font-mono truncate" title={currentFile}>
               {currentFile.split(/[/\\]/).pop()}
             </span>
          </div>
        )}
      </div>

      {/* Center Controls */}
      <div className="flex items-center gap-6 no-drag">
        {/* View Mode Toggle - Segmented Control */}
        <div className="flex items-center bg-dark-bg p-1 rounded-lg border border-dark-border/50 shadow-inner">
          <button
            onClick={() => isSimpleModeAvailable && onViewModeChange('simple')}
            disabled={!isSimpleModeAvailable}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 ${
              viewMode === 'simple'
                ? 'bg-dark-surface text-sikuli-400 shadow-sm'
                : !isSimpleModeAvailable
                  ? 'text-gray-700 cursor-not-allowed'
                  : 'text-gray-500 hover:text-gray-300 hover:bg-dark-hover/50'
            }`}
            title={isSimpleModeAvailable ? 'Simple Mode' : 'Simple Mode (unavailable for external scripts)'}
          >
            <LayoutList size={14} />
            <span>Simple</span>
          </button>
          <button
            onClick={() => isSimpleModeAvailable && onViewModeChange('flow')}
            disabled={!isSimpleModeAvailable}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 ${
              viewMode === 'flow'
                ? 'bg-dark-surface text-sikuli-400 shadow-sm'
                : !isSimpleModeAvailable
                  ? 'text-gray-700 cursor-not-allowed'
                  : 'text-gray-500 hover:text-gray-300 hover:bg-dark-hover/50'
            }`}
            title={isSimpleModeAvailable ? 'Flow Mode' : 'Flow Mode (unavailable for external scripts)'}
          >
            <GitBranch size={14} />
            <span>Flow</span>
          </button>
          <button
            onClick={() => onViewModeChange('code')}
            className={`flex items-center gap-2 px-3 py-1.5 rounded-md text-xs font-medium transition-all duration-200 ${
              viewMode === 'code'
                ? 'bg-dark-surface text-sikuli-400 shadow-sm'
                : 'text-gray-500 hover:text-gray-300 hover:bg-dark-hover/50'
            }`}
            title="Code Mode"
          >
            <Code size={14} />
            <span>Code</span>
          </button>
        </div>

        {/* Run/Stop Button */}
        <div className="flex items-center">
          {isRunning ? (
            <button
              onClick={onStop}
              className="group flex items-center gap-2 px-4 py-1.5 bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 hover:border-red-500/50 text-red-500 rounded-md transition-all duration-200 shadow-sm"
              title="Stop Script (Shift+Alt+C)"
            >
              <Square size={14} className="fill-current" />
              <span className="text-xs font-bold uppercase tracking-wider">Stop</span>
            </button>
          ) : (
            <button
              onClick={onRun}
              className="group flex items-center gap-2 px-4 py-1.5 bg-sikuli-500 hover:bg-sikuli-400 text-white rounded-md transition-all duration-200 shadow-lg shadow-sikuli-500/20 hover:shadow-sikuli-500/30 hover:-translate-y-0.5 active:translate-y-0"
              title="Run Script"
            >
              <Play size={14} className="fill-current" />
              <span className="text-xs font-bold uppercase tracking-wider">Run</span>
            </button>
          )}
        </div>

        {/* Hide Mode Checkbox */}
        <label
          className="flex items-center gap-2 cursor-pointer select-none group"
          title="Hide IDE window when script runs (Shift+Alt+C to stop and restore)"
        >
          <div className="relative flex items-center">
            <input
              type="checkbox"
              checked={hideMode}
              onChange={(e) => onHideModeChange(e.target.checked)}
              className="peer appearance-none w-4 h-4 rounded border border-gray-600 bg-dark-bg checked:bg-sikuli-500 checked:border-sikuli-500 focus:ring-offset-0 focus:ring-1 focus:ring-sikuli-500/50 cursor-pointer transition-colors"
            />
            <svg className="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 w-3 h-3 text-white pointer-events-none opacity-0 peer-checked:opacity-100 transition-opacity" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
              <polyline points="20 6 9 17 4 12"></polyline>
            </svg>
          </div>
          <span className="text-xs text-gray-500 group-hover:text-gray-300 transition-colors">Hide on Run</span>
        </label>
      </div>

      {/* Right Controls */}
      <div className="flex items-center gap-2 no-drag">
        <button
          className="p-2 text-gray-500 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-all duration-200"
          title="Settings"
        >
          <Settings size={18} strokeWidth={1.5} />
        </button>
      </div>
    </header>
  )
}
