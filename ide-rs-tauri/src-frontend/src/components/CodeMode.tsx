import { useCallback, useState, useEffect, useRef } from 'react'
import { Copy, Check, Save, Image as ImageIcon } from 'lucide-react'
import Editor, { OnMount, BeforeMount } from '@monaco-editor/react'
import type { editor } from 'monaco-editor'

interface CodeModeProps {
  sourceCode: string
  currentFile: string | null
  pythonVersion?: 'python2' | 'python3' | 'unknown' | null
  onSourceCodeChange?: (code: string) => void
  onSave?: () => void
  imagePatterns?: Map<string, string> // Map of pattern path -> base64 image
}

interface ImagePosition {
  imagePath: string
  lineNumber: number
  column: number
  endColumn: number
}

// Pattern to detect image references in SikuliX code
// Supports both direct strings and Pattern() syntax:
//   click("image.png")
//   click(Pattern("image.png").similar(0.8))
//   find(Pattern("test.png").targetOffset(10, 20))
const IMAGE_PATTERN_REGEX = /(?:click|wait|exists|find|findAll|hover|rightClick|doubleClick)\s*\(\s*(?:Pattern\s*\(\s*)?["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi

// Also match standalone Pattern() definitions and image path assignments
const STANDALONE_IMAGE_REGEX = /(?:Pattern\s*\(\s*)?["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi

/**
 * Extract image references from Python code
 * Pythonコードから画像参照を抽出
 */
function extractImageReferences(code: string): string[] {
  const images: string[] = []

  // First try command-based pattern (click, wait, etc.)
  const commandRegex = new RegExp(IMAGE_PATTERN_REGEX.source, 'gi')
  let match
  while ((match = commandRegex.exec(code)) !== null) {
    if (!images.includes(match[1])) {
      images.push(match[1])
    }
  }

  // Also check for standalone image references (for broader detection)
  const standaloneRegex = new RegExp(STANDALONE_IMAGE_REGEX.source, 'gi')
  while ((match = standaloneRegex.exec(code)) !== null) {
    if (!images.includes(match[1])) {
      images.push(match[1])
    }
  }

  return images
}

/**
 * Find positions of image references in code for inline display
 */
function findImagePositions(code: string): ImagePosition[] {
  const positions: ImagePosition[] = []
  const lines = code.split('\n')
  const imagePathRegex = /["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi

  lines.forEach((line, lineIndex) => {
    let match
    const regex = new RegExp(imagePathRegex.source, 'gi')
    while ((match = regex.exec(line)) !== null) {
      positions.push({
        imagePath: match[1],
        lineNumber: lineIndex + 1,
        column: match.index + 1,
        endColumn: match.index + match[0].length + 1,
      })
    }
  })

  return positions
}

/**
 * CodeMode Component - Monaco-based Python source code editor with syntax highlighting
 * コードモードコンポーネント - シンタックスハイライト付きMonacoベースPythonエディター
 */
export function CodeMode({ sourceCode, currentFile, pythonVersion, onSourceCodeChange, onSave, imagePatterns }: CodeModeProps) {
  const [copied, setCopied] = useState(false)
  const [localCode, setLocalCode] = useState(sourceCode)
  const [isModified, setIsModified] = useState(false)
  const [showImagePanel, setShowImagePanel] = useState(true)
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null)
  const monacoRef = useRef<typeof import('monaco-editor') | null>(null)
  const widgetsRef = useRef<editor.IContentWidget[]>([])
  const decorationsRef = useRef<string[]>([])

  // Extract images from current code
  const imageRefs = extractImageReferences(localCode)

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

  const handleEditorChange = useCallback((value: string | undefined) => {
    const newCode = value || ''
    setLocalCode(newCode)
    setIsModified(newCode !== sourceCode)
    onSourceCodeChange?.(newCode)
  }, [sourceCode, onSourceCodeChange])

  /**
   * Update inline image widgets in the editor
   * エディタのインライン画像ウィジェットを更新
   * @param patterns - Pass imagePatterns directly to avoid stale closure
   */
  const updateImageWidgets = useCallback((patterns: Map<string, string> | undefined) => {
    const editorInstance = editorRef.current
    const monaco = monacoRef.current

    console.log('[ImageWidget] updateImageWidgets called', {
      hasEditor: !!editorInstance,
      hasMonaco: !!monaco,
      hasImagePatterns: !!patterns,
      imagePatternsSize: patterns?.size ?? 0,
      imagePatternsKeys: patterns ? Array.from(patterns.keys()) : []
    })

    if (!editorInstance || !monaco || !patterns) return

    // Remove old widgets
    widgetsRef.current.forEach(widget => {
      editorInstance.removeContentWidget(widget)
    })
    widgetsRef.current = []

    // Remove old decorations
    decorationsRef.current = editorInstance.deltaDecorations(decorationsRef.current, [])

    // Find image positions
    const positions = findImagePositions(localCode)
    const newDecorations: editor.IModelDeltaDecoration[] = []

    console.log('[ImageWidget] positions found:', positions)

    positions.forEach((pos, index) => {
      const imageData = patterns.get(pos.imagePath)
      console.log('[ImageWidget] checking image:', pos.imagePath, 'hasData:', !!imageData)
      if (!imageData) return

      // Create decoration to highlight image path
      newDecorations.push({
        range: new monaco.Range(pos.lineNumber, pos.column, pos.lineNumber, pos.endColumn),
        options: {
          inlineClassName: 'sikuli-image-path',
          hoverMessage: { value: '**' + pos.imagePath + '**' },
        },
      })

      // Create inline content widget for image preview
      const widgetId = 'image-widget-' + index

      // Create DOM node once and cache it
      const container = document.createElement('div')
      container.className = 'sikuli-inline-image-widget'
      container.style.cssText = 'display: inline-flex; align-items: center; justify-content: center; margin-left: 4px; padding: 2px; background: #2d2d2d; border: 1px solid #444; border-radius: 3px; cursor: pointer;'

      const img = document.createElement('img')
      img.src = imageData
      img.style.cssText = 'max-width: 48px; max-height: 24px; object-fit: contain;'
      img.title = pos.imagePath

      container.appendChild(img)

      const widget: editor.IContentWidget = {
        getId: () => widgetId,
        getDomNode: () => container,
        getPosition: () => ({
          position: { lineNumber: pos.lineNumber, column: pos.endColumn },
          preference: [monaco.editor.ContentWidgetPositionPreference.EXACT],
        }),
      }

      editorInstance.addContentWidget(widget)
      widgetsRef.current.push(widget)
      console.log('[ImageWidget] Widget added:', widgetId, 'at line', pos.lineNumber, 'col', pos.endColumn)
    })

    // Apply decorations
    decorationsRef.current = editorInstance.deltaDecorations([], newDecorations)
  }, [localCode])

  // Update widgets when code or images change
  // Note: We pass imagePatterns directly to avoid stale closure issues
  const imagePatternsSize = imagePatterns?.size ?? 0
  useEffect(() => {
    // Only update if we have patterns loaded or need to clear
    console.log('[ImageWidget] useEffect triggered, imagePatternsSize:', imagePatternsSize)
    updateImageWidgets(imagePatterns)
  }, [updateImageWidgets, imagePatterns, imagePatternsSize])

  // Handle Ctrl+S for save
  const handleEditorMount: OnMount = useCallback((editorInstance, monaco) => {
    editorRef.current = editorInstance
    monacoRef.current = monaco

    // Add Ctrl+S keybinding for save
    editorInstance.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, () => {
      if (onSave && isModified) {
        onSave()
      }
    })

    // Add custom CSS for inline image styling
    const existingStyle = document.getElementById('sikuli-editor-styles')
    if (!existingStyle) {
      const style = document.createElement('style')
      style.id = 'sikuli-editor-styles'
      style.textContent = '.sikuli-image-path { background-color: rgba(78, 201, 176, 0.15); border-radius: 2px; } .sikuli-inline-image-widget { z-index: 10; } .sikuli-inline-image-widget:hover { border-color: #4EC9B0 !important; transform: scale(1.1); transition: all 0.15s ease; }'
      document.head.appendChild(style)
    }

    // Note: Widget initialization is handled by useEffect when imagePatterns changes
    // No need for setTimeout here as it would use stale closure

    // Focus the editor
    editorInstance.focus()
  }, [onSave, isModified])

  // Configure Monaco before mount
  const handleEditorWillMount: BeforeMount = useCallback((monaco) => {
    // Define a dark theme similar to VS Code Dark+
    monaco.editor.defineTheme('sikuli-dark', {
      base: 'vs-dark',
      inherit: true,
      rules: [
        { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
        { token: 'keyword', foreground: '569CD6' },
        { token: 'string', foreground: 'CE9178' },
        { token: 'number', foreground: 'B5CEA8' },
        { token: 'function', foreground: 'DCDCAA' },
        { token: 'variable', foreground: '9CDCFE' },
        { token: 'type', foreground: '4EC9B0' },
        { token: 'class', foreground: '4EC9B0' },
        { token: 'decorator', foreground: 'C586C0' },
      ],
      colors: {
        'editor.background': '#18181b', // zinc-900
        'editor.foreground': '#e4e4e7', // zinc-200
        'editor.lineHighlightBackground': '#27272a', // zinc-800
        'editorCursor.foreground': '#38bdf8', // sikuli-400
        'editor.selectionBackground': '#264F78',
        'editor.inactiveSelectionBackground': '#3A3D41',
        'editorLineNumber.foreground': '#71717a', // zinc-500
        'editorLineNumber.activeForeground': '#e4e4e7', // zinc-200
        'editorGutter.background': '#18181b', // zinc-900
      },
    })
  }, [])

  // Get filename from path (handle both / and \ separators)
  const filename = currentFile ? currentFile.split(/[/\\]/).pop() || 'script.py' : 'script.py'

  // Get Python version display label
  const versionLabel = pythonVersion === 'python2' ? 'Python 2'
    : pythonVersion === 'python3' ? 'Python 3'
    : pythonVersion === 'unknown' ? 'Python 2/3'
    : null

  const versionColor = pythonVersion === 'python2' ? 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30'
    : pythonVersion === 'python3' ? 'bg-blue-500/20 text-blue-400 border-blue-500/30'
    : 'bg-green-500/20 text-green-400 border-green-500/30'

  return (
    <div className="h-full flex flex-col bg-dark-bg overflow-hidden">
      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 bg-dark-surface border-b border-dark-border/50">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2">
             <span className="text-xs font-mono text-gray-300">{filename}</span>
             {isModified && (
               <span className="w-2 h-2 bg-yellow-500 rounded-full" title="Unsaved changes"></span>
             )}
          </div>
          <span className="text-xs text-gray-600 border-l border-dark-border pl-4">{localCode.split('\n').length} lines</span>
          {versionLabel && (
            <span className={'px-2 py-0.5 text-[10px] font-medium rounded border ' + versionColor}>
              {versionLabel}
            </span>
          )}
        </div>
        <div className="flex items-center gap-1">
          {imageRefs.length > 0 && (
            <button
              onClick={() => setShowImagePanel(!showImagePanel)}
              className={`flex items-center gap-1.5 px-2.5 py-1.5 text-xs rounded-md transition-colors ${
                showImagePanel 
                  ? 'text-sikuli-400 bg-sikuli-500/10' 
                  : 'text-gray-400 hover:text-gray-200 hover:bg-dark-hover'
              }`}
              title="Toggle Image Panel"
            >
              <ImageIcon size={14} />
              <span className="font-medium">{imageRefs.length}</span>
            </button>
          )}
          {onSave && (
            <button
              onClick={onSave}
              disabled={!isModified}
              className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-colors disabled:opacity-30"
              title="Save (Ctrl+S)"
            >
              <Save size={14} />
              <span>Save</span>
            </button>
          )}
          <button
            onClick={handleCopy}
            disabled={!localCode}
            className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-gray-400 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-colors disabled:opacity-30"
            title="Copy to Clipboard"
          >
            {copied ? <Check size={14} className="text-green-400" /> : <Copy size={14} />}
            <span>{copied ? 'Copied' : 'Copy'}</span>
          </button>
        </div>
      </div>

      {/* Main Editor Area */}
      <div className="flex-1 flex overflow-hidden">
        {/* Monaco Editor */}
        <div className="flex-1 overflow-hidden">
          <Editor
            height="100%"
            defaultLanguage="python"
            theme="sikuli-dark"
            value={localCode}
            onChange={handleEditorChange}
            onMount={handleEditorMount}
            beforeMount={handleEditorWillMount}
            options={{
              fontSize: 13,
              fontFamily: "'JetBrains Mono', 'Consolas', 'Monaco', 'Courier New', monospace",
              fontLigatures: true,
              lineNumbers: 'on',
              minimap: { enabled: false },
              scrollBeyondLastLine: false,
              automaticLayout: true,
              tabSize: 4,
              insertSpaces: true,
              wordWrap: 'off',
              folding: true,
              foldingHighlight: true,
              bracketPairColorization: { enabled: true },
              renderLineHighlight: 'line',
              selectOnLineNumbers: true,
              roundedSelection: true,
              cursorBlinking: 'smooth',
              cursorSmoothCaretAnimation: 'on',
              smoothScrolling: true,
              contextmenu: true,
              mouseWheelZoom: true,
              padding: { top: 12, bottom: 12 },
              scrollbar: {
                vertical: 'visible',
                horizontal: 'visible',
                verticalScrollbarSize: 10,
                horizontalScrollbarSize: 10,
                useShadows: false,
              },
            }}
          />
        </div>

        {/* Image References Panel - SikuliX style inline image display */}
        {showImagePanel && imageRefs.length > 0 && (
          <div className="w-56 bg-dark-sidebar border-l border-dark-border overflow-y-auto">
            <div className="px-4 py-3 border-b border-dark-border/50 bg-dark-sidebar sticky top-0 z-10">
              <span className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Pattern Images</span>
            </div>
            <div className="p-3 space-y-3">
              {imageRefs.map((imagePath, index) => {
                const imageData = imagePatterns?.get(imagePath)
                const filename = imagePath.split(/[/\\]/).pop() || imagePath
                return (
                  <div
                    key={index}
                    className="bg-dark-surface rounded-lg border border-dark-border/50 overflow-hidden group cursor-pointer hover:border-sikuli-500/50 hover:shadow-md transition-all"
                    title={imagePath}
                  >
                    <div className="h-24 flex items-center justify-center bg-dark-bg/50 relative">
                      <div className="absolute inset-0 bg-[url('/transparent-grid.png')] opacity-10"></div>
                      {imageData ? (
                        <img
                          src={imageData}
                          alt={filename}
                          className="max-w-full max-h-full object-contain relative z-10 p-2"
                        />
                      ) : (
                        <div className="flex flex-col items-center gap-1 text-gray-600 relative z-10">
                          <ImageIcon size={20} />
                          <span className="text-[10px]">Not found</span>
                        </div>
                      )}
                    </div>
                    <div className="px-3 py-2 text-xs text-gray-400 truncate font-mono border-t border-dark-border/50 group-hover:text-gray-200 group-hover:bg-dark-hover transition-colors">
                      {filename}
                    </div>
                  </div>
                )
              })}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
