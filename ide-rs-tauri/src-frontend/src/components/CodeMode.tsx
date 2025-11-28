import { useCallback, useState, useRef, useEffect } from 'react'
import { Copy, Check, Save } from 'lucide-react'

interface CodeModeProps {
  sourceCode: string
  currentFile: string | null
  pythonVersion?: 'python2' | 'python3' | 'unknown' | null
  onSourceCodeChange?: (code: string) => void
  onSave?: () => void
}

/**
 * CodeMode Component - Editable Python source code editor
 * コードモードコンポーネント - 編集可能なPythonソースコードエディター
 */
export function CodeMode({ sourceCode, currentFile, pythonVersion, onSourceCodeChange, onSave }: CodeModeProps) {
  const [copied, setCopied] = useState(false)
  const [localCode, setLocalCode] = useState(sourceCode)
  const [isModified, setIsModified] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const lineNumbersRef = useRef<HTMLDivElement>(null)

  // Sync local code with prop changes
  useEffect(() => {
    setLocalCode(sourceCode)
    setIsModified(false)
  }, [sourceCode])

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(localCode)
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }, [localCode])

  const handleCodeChange = useCallback((e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const newCode = e.target.value
    setLocalCode(newCode)
    setIsModified(newCode !== sourceCode)
    onSourceCodeChange?.(newCode)
  }, [sourceCode, onSourceCodeChange])

  // Sync scroll between textarea and line numbers
  const handleScroll = useCallback(() => {
    if (textareaRef.current && lineNumbersRef.current) {
      lineNumbersRef.current.scrollTop = textareaRef.current.scrollTop
    }
  }, [])

  // Get filename from path (handle both / and \ separators)
  const filename = currentFile ? currentFile.split(/[/\\]/).pop() || 'script.py' : 'script.py'

  // Get Python version display label
  // Python2 only - コードにPython2固有の構文がある（要変換）
  // Python3 only - コードにPython3固有の構文がある
  // Python2/3 OK - 両方で動作可能
  const versionLabel = pythonVersion === 'python2' ? 'Python2 only'
    : pythonVersion === 'python3' ? 'Python3 only'
    : pythonVersion === 'unknown' ? 'Python2/3 OK'
    : null

  const versionColor = pythonVersion === 'python2' ? 'bg-yellow-600'  // 警告色 - 変換が必要
    : pythonVersion === 'python3' ? 'bg-blue-600'   // モダン
    : 'bg-green-600'  // 互換性あり

  return (
    <div className="h-full flex flex-col bg-dark-bg overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-dark-surface border-b border-dark-border">
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-400">{filename}</span>
          {isModified && (
            <span className="text-xs text-yellow-400">*</span>
          )}
          <span className="text-xs text-gray-500">({localCode.split('\n').length} lines)</span>
          {versionLabel && (
            <span className={'px-2 py-0.5 text-xs rounded ' + versionColor + ' text-white'}>
              {versionLabel}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {onSave && (
            <button
              onClick={onSave}
              disabled={!isModified}
              className="flex items-center gap-1.5 px-2 py-1 text-sm text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded transition-colors disabled:opacity-50"
              title="Save (Ctrl+S)"
            >
              <Save size={14} />
              <span>Save</span>
            </button>
          )}
          <button
            onClick={handleCopy}
            disabled={!localCode}
            className="flex items-center gap-1.5 px-2 py-1 text-sm text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded transition-colors disabled:opacity-50"
            title="Copy to Clipboard"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
            <span>{copied ? 'Copied!' : 'Copy'}</span>
          </button>
        </div>
      </div>

      {/* Code Editor - Always show textarea for editing */}
      <div className="flex-1 flex overflow-hidden">
        {/* Line Numbers */}
        <div
          ref={lineNumbersRef}
          className="w-12 bg-dark-surface border-r border-dark-border overflow-hidden select-none"
        >
          <div className="py-2 font-mono text-sm leading-6">
            {(localCode || '').split('\n').map((_, i) => (
              <div key={i} className="px-2 text-right text-gray-500">
                {i + 1}
              </div>
            ))}
          </div>
        </div>

        {/* Textarea Editor */}
        <textarea
          ref={textareaRef}
          value={localCode}
          onChange={handleCodeChange}
          onScroll={handleScroll}
          spellCheck={false}
          className="flex-1 p-2 font-mono text-sm leading-6 bg-dark-bg text-gray-200 resize-none outline-none focus:ring-1 focus:ring-sikuli-500/50"
          placeholder="# Python source code here..."
        />
      </div>
    </div>
  )
}
