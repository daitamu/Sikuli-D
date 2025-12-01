import { useCallback, useState, useRef, useEffect } from 'react'
import { X, Trash2, Copy, Check } from 'lucide-react'
import type { ConsoleEntry } from '../types/script'

interface ConsoleProps {
  entries: ConsoleEntry[]
  onClear: () => void
  onClose: () => void
}

/**
 * Console Component - Execution logs and output
 */
export function Console({ entries, onClear, onClose }: ConsoleProps) {
  const [copied, setCopied] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)

  /**
   * Auto-scroll to bottom when new entries are added
   */
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight
    }
  }, [entries])

  /**
   * Get level-specific styles
   */
  const getLevelStyles = (level: ConsoleEntry['level']) => {
    switch (level) {
      case 'error':
        return 'text-red-400 bg-red-950/30'
      case 'warn':
        return 'text-yellow-400 bg-yellow-950/30'
      case 'debug':
        return 'text-gray-400'
      case 'info':
      default:
        return 'text-gray-300'
    }
  }

  /**
   * Format timestamp
   */
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('ja-JP', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    })
  }

  /**
   * Copy all logs to clipboard
   */
  const handleCopy = useCallback(async () => {
    const text = entries
      .map((entry) => '[' + formatTime(entry.timestamp) + '] [' + entry.level.toUpperCase() + '] ' + entry.message)
      .join('\n')

    try {
      await navigator.clipboard.writeText(text)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }, [entries])

  return (
    <div className="h-48 bg-dark-surface/95 backdrop-blur border-t border-dark-border flex flex-col shadow-lg z-20">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-dark-border/50 bg-dark-surface">
        <div className="flex items-center gap-2">
           <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Console Output</h3>
           <span className="text-[10px] bg-dark-bg px-1.5 py-0.5 rounded text-gray-500">{entries.length}</span>
        </div>
        
        <div className="flex items-center gap-1">
          <button
            onClick={handleCopy}
            disabled={entries.length === 0}
            className="p-1.5 text-gray-500 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-colors disabled:opacity-30 disabled:cursor-not-allowed"
            title="Copy to Clipboard"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
          </button>
          <button
            onClick={onClear}
            className="p-1.5 text-gray-500 hover:text-red-400 hover:bg-dark-hover rounded-md transition-colors"
            title="Clear Console"
          >
            <Trash2 size={14} />
          </button>
          <button
            onClick={onClose}
            className="p-1.5 text-gray-500 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-colors"
            title="Close Console"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Log Entries */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto font-mono text-[11px] leading-relaxed p-2 bg-dark-bg">
        {entries.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-600 gap-2">
            <div className="p-3 rounded-full bg-dark-surface border border-dark-border/50">
               <span className="text-xl opacity-50">_</span>
            </div>
            <span>Ready for output</span>
          </div>
        ) : (
          entries.map((entry) => (
            <div
              key={entry.id}
              className={'px-2 py-1 rounded my-0.5 flex items-start gap-2 ' + getLevelStyles(entry.level)}
            >
              <span className="text-gray-600 shrink-0 font-medium select-none">{formatTime(entry.timestamp)}</span>
              <span className="break-all">{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
