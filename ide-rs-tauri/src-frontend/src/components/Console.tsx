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
    <div className="h-48 bg-dark-surface border-t border-dark-border flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-dark-border">
        <h3 className="text-sm font-medium text-gray-400">Console</h3>
        <div className="flex items-center gap-1">
          <button
            onClick={handleCopy}
            disabled={entries.length === 0}
            className="p-1 text-gray-500 hover:text-gray-300 hover:bg-dark-hover rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            title="Copy to Clipboard"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
          </button>
          <button
            onClick={onClear}
            className="p-1 text-gray-500 hover:text-gray-300 hover:bg-dark-hover rounded transition-colors"
            title="Clear Console"
          >
            <Trash2 size={14} />
          </button>
          <button
            onClick={onClose}
            className="p-1 text-gray-500 hover:text-gray-300 hover:bg-dark-hover rounded transition-colors"
            title="Close Console"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Log Entries */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto font-mono text-xs">
        {entries.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-500">
            No output yet
          </div>
        ) : (
          entries.map((entry) => (
            <div
              key={entry.id}
              className={'px-3 py-1 border-b border-dark-border/50 ' + getLevelStyles(entry.level)}
            >
              <span className="text-gray-500 mr-2">[{formatTime(entry.timestamp)}]</span>
              <span className="uppercase text-xs mr-2">[{entry.level}]</span>
              <span>{entry.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  )
}
