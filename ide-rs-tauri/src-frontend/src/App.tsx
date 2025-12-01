import { useState, useCallback, useMemo, useRef, useEffect } from 'react'
import { v4 as uuidv4 } from 'uuid'
import { Header } from './components/Header'
import { Toolbox } from './components/Toolbox'
import { SimpleMode } from './components/SimpleMode'
import { FlowMode } from './components/FlowMode'
import { CodeMode } from './components/CodeMode'
import { Console } from './components/Console'
import { PropertyPanel } from './components/PropertyPanel'
import { useTauri, useTauriEvents } from './hooks/useTauri'
import type { ViewMode, ScriptLine, ConsoleEntry, CommandType } from './types/script'

/**
 * Main Application Component
 * メインアプリケーションコンポーネント
 */
function App() {
  // View mode state / ビューモード状態
  const [viewMode, setViewMode] = useState<ViewMode>('simple')

  // Script data state / スクリプトデータ状態
  const [script, setScript] = useState<ScriptLine[]>([
    {
      id: uuidv4(),
      type: 'start' as const,
      flowConfig: { x: 100, y: 100 },
    },
  ])

  // Selected line state / 選択行状態
  const [selectedLineId, setSelectedLineId] = useState<string | null>(null)

  // Running state / 実行中状態
  const [isRunning, setIsRunning] = useState(false)

  // Current running process ID / 現在実行中のプロセスID
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_runningProcessId, setRunningProcessId] = useState<string | null>(null)

  // Console output state / コンソール出力状態
  const [consoleOutput, setConsoleOutput] = useState<ConsoleEntry[]>([])

  // Console panel visibility / コンソールパネル表示状態
  const [isConsoleVisible, setIsConsoleVisible] = useState(true)

  // Current file path / 現在のファイルパス
  const [currentFile, setCurrentFile] = useState<string | null>(null)

  // Source code for code mode / コードモード用ソースコード
  const [sourceCode, setSourceCode] = useState<string>('')

  // Detected Python version / 検出されたPythonバージョン
  const [pythonVersion, setPythonVersion] = useState<'python2' | 'python3' | 'unknown' | null>(null)

  // Simple mode availability / Simpleモードの可用性
  // true = created in Simple mode, false = external script (Code mode only)
  const [isSimpleModeAvailable, setIsSimpleModeAvailable] = useState(true)

  // Property panel visibility / プロパティパネル表示状態
  const [isPropertyPanelVisible, setIsPropertyPanelVisible] = useState(true)

  // Hide mode state / Hideモード状態
  // When enabled, IDE window minimizes when script runs
  const [hideMode, setHideMode] = useState(false)

  // IDE version from Rust backend / RustバックエンドからのIDEバージョン
  const [ideVersion, setIdeVersion] = useState('0.8.0')

  // Image patterns for Code mode / コードモード用画像パターン
  // Map of image path -> base64 data URL
  const [imagePatterns, setImagePatterns] = useState<Map<string, string>>(new Map())

  // Manual stop flag / 手動停止フラグ
  // Used to suppress "exit code: unknown" message when user manually stops the script
  const isManualStopRef = useRef(false)

  // Hide mode ref for callbacks / コールバック用のHideモードref
  const hideModeRef = useRef(false)

  // Tauri IPC hooks / Tauri IPCフック
  const {
    runScriptStreaming,
    stopAllScripts,
    openFileDialog,
    saveFileDialog,
    startCapture,
    getLastCapturePath,
    loadImageAsBase64,
    minimizeWindow,
    showWindow,
    getIdeVersion,
  } = useTauri()

  // Listen to Tauri events / Tauriイベントをリッスン
  useTauriEvents(
    // onLog callback
    useCallback((entry: { level: ConsoleEntry['level']; message: string }) => {
      setConsoleOutput((prev) => [
        ...prev,
        {
          id: uuidv4(),
          timestamp: new Date(),
          ...entry,
        },
      ])
    }, []),
    // onHighlight callback
    useCallback((lineId: string) => {
      setSelectedLineId(lineId)
    }, []),
    // onScriptEnd callback
    useCallback(() => {
      setIsRunning(false)
      setRunningProcessId(null)
    }, []),
    // onScriptOutput callback
    useCallback((_processId: string, line: string, isError: boolean) => {
      // Parse log level from Rust log format: [timestamp LEVEL module] message
      // Example: [2025-11-28T02:36:41Z INFO  sikulid::runner] Running script
      let level: ConsoleEntry['level'] = isError ? 'error' : 'info'
      let message = line

      if (isError) {
        // Try to parse Rust log format from stderr
        // Module names can contain :: like sikulid::runner, sikulid::python
        const logMatch = line.match(/^\[\d{4}-\d{2}-\d{2}T[\d:]+Z\s+(INFO|WARN|ERROR|DEBUG)\s+[\w:]+\]\s*(.*)$/)
        if (logMatch) {
          const logLevel = logMatch[1].toLowerCase()
          message = logMatch[2] || line
          if (logLevel === 'info') level = 'info'
          else if (logLevel === 'warn') level = 'warn'
          else if (logLevel === 'error') level = 'error'
          else if (logLevel === 'debug') level = 'debug'
        }
      }

      setConsoleOutput((prev) => [
        ...prev,
        {
          id: uuidv4(),
          timestamp: new Date(),
          level,
          message,
        },
      ])
    }, []),
    // onScriptComplete callback
    useCallback((processId: string, exitCode: number | null) => {
      setRunningProcessId((currentId) => {
        if (currentId === processId) {
          setIsRunning(false)
          // Skip message if manually stopped - handleStop already shows appropriate message
          // 手動停止の場合はメッセージをスキップ - handleStopで適切なメッセージを表示済み
          if (!isManualStopRef.current) {
            setConsoleOutput((prev) => [
              ...prev,
              {
                id: uuidv4(),
                timestamp: new Date(),
                level: exitCode === 0 ? 'info' : 'error',
                message: exitCode === 0
                  ? 'Script completed successfully'
                  : `Script finished with exit code: ${exitCode ?? 'unknown'}`,
              },
            ])
          }
          isManualStopRef.current = false  // Reset flag
          return null
        }
        return currentId
      })
    }, [])
  )

  // Sync hideMode state with ref for use in callbacks
  // コールバックで使用するためにhideMode状態をrefと同期
  useEffect(() => {
    hideModeRef.current = hideMode
  }, [hideMode])

  // Flag to skip ImageLoader (for testing)
  // ImageLoaderをスキップするフラグ（テスト用）
  const skipImageLoaderRef = useRef(false)

  // Expose debug API for testing (development only)
  // テスト用デバッグAPI（開発環境のみ）
  useEffect(() => {
    if (import.meta.env.DEV) {
      (window as any).__SIKULID_DEBUG__ = {
        setCurrentFile,
        setSourceCode,
        setImagePatterns,
        setViewMode,
        skipImageLoader: (skip: boolean) => {
          skipImageLoaderRef.current = skip
          console.log('[Debug] skipImageLoader set to:', skip)
        },
        getCurrentState: () => ({
          currentFile,
          sourceCode: sourceCode?.substring(0, 200),
          imagePatternsSize: imagePatterns.size,
          imagePatternsKeys: Array.from(imagePatterns.keys()),
          viewMode,
          skipImageLoader: skipImageLoaderRef.current,
        }),
      }
      console.log('[Debug] Debug API exposed on window.__SIKULID_DEBUG__')
    }
    return () => {
      if (import.meta.env.DEV) {
        delete (window as any).__SIKULID_DEBUG__
      }
    }
  }, [currentFile, sourceCode, imagePatterns, viewMode])

  // Fetch IDE version on mount
  // マウント時にIDEバージョンを取得
  useEffect(() => {
    getIdeVersion().then(setIdeVersion)
  }, [getIdeVersion])

  // Load images referenced in source code
  // ソースコード内で参照されている画像を読み込み
  useEffect(() => {
    console.log('[ImageLoader] Effect triggered:', { sourceCode: sourceCode?.substring(0, 100), currentFile, skipImageLoader: skipImageLoaderRef.current })

    // Skip if flag is set (for testing)
    if (skipImageLoaderRef.current) {
      console.log('[ImageLoader] Skipped (skipImageLoader flag is set)')
      return
    }

    if (!sourceCode || !currentFile) {
      console.log('[ImageLoader] No sourceCode or currentFile, clearing patterns')
      setImagePatterns(new Map())
      return
    }

    // Extract image references from code using the same regex as CodeMode
    const STANDALONE_IMAGE_REGEX = /(?:Pattern\s*\(\s*)?["']([^"']+\.(?:png|jpg|jpeg|gif|bmp))["']/gi
    const images: string[] = []
    let match
    while ((match = STANDALONE_IMAGE_REGEX.exec(sourceCode)) !== null) {
      if (!images.includes(match[1])) {
        images.push(match[1])
      }
    }

    console.log('[ImageLoader] Detected images:', images)

    if (images.length === 0) {
      setImagePatterns(new Map())
      return
    }

    // Get directory of current file for relative path resolution
    // For .sikuli/.sikulix folders, the currentFile might be the folder itself
    // For regular .py files, we need the parent directory
    let currentDir = currentFile
    if (currentFile.match(/\.(sikuli|sikulix)$/i)) {
      // currentFile is a .sikuli folder - use it directly
      currentDir = currentFile
    } else if (currentFile.match(/\.(sikuli|sikulix)[/\\]/i)) {
      // currentFile is inside a .sikuli folder (e.g., Yesman.sikuli/Yesman.py)
      // Extract the .sikuli folder path
      const match = currentFile.match(/^(.+\.(sikuli|sikulix))/i)
      currentDir = match ? match[1] : currentFile.replace(/[/\\][^/\\]*$/, '')
    } else {
      // Regular file - get parent directory
      currentDir = currentFile.replace(/[/\\][^/\\]*$/, '')
    }
    console.log('[ImageLoader] Current directory:', currentDir, '(from file:', currentFile, ')')

    // Load images asynchronously
    const loadImages = async () => {
      const newPatterns = new Map<string, string>()

      for (const imagePath of images) {
        try {
          // Resolve path: if relative, make it relative to current file's directory
          let fullPath = imagePath
          if (!imagePath.match(/^[a-zA-Z]:[/\\]/) && !imagePath.startsWith('/')) {
            // Relative path - resolve from current file's directory
            fullPath = `${currentDir}/${imagePath}`.replace(/\\/g, '/')
          }

          console.log('[ImageLoader] Loading image:', { imagePath, fullPath })
          const base64 = await loadImageAsBase64(fullPath)
          if (base64) {
            console.log('[ImageLoader] Loaded successfully:', imagePath, base64.substring(0, 50) + '...')
            newPatterns.set(imagePath, base64)
          } else {
            console.warn('[ImageLoader] loadImageAsBase64 returned null for:', fullPath)
          }
        } catch (err) {
          console.error('[ImageLoader] Failed to load image:', imagePath, err)
        }
      }

      console.log('[ImageLoader] Setting patterns, count:', newPatterns.size)
      setImagePatterns(newPatterns)
    }

    loadImages()
  }, [sourceCode, currentFile, loadImageAsBase64])

  // Global shortcut for stopping script (Shift+Alt+C)
  // スクリプト停止用グローバルショートカット (Shift+Alt+C)
  useEffect(() => {
    let cleanup: (() => void) | null = null

    const setupShortcut = async () => {
      try {
        // Dynamic import of global-shortcut plugin
        const { register, unregister } = await import('@tauri-apps/plugin-global-shortcut')

        // Register Shift+Alt+C shortcut
        await register('Shift+Alt+C', async () => {
          console.log('Global shortcut triggered: Shift+Alt+C')
          // Stop all scripts and restore window
          const success = await stopAllScripts()
          if (success) {
            setIsRunning(false)
            setRunningProcessId(null)
            // Restore window if it was hidden
            if (hideModeRef.current) {
              await showWindow()
            }
          }
        })

        cleanup = () => {
          unregister('Shift+Alt+C').catch(console.error)
        }
      } catch (error) {
        console.error('Failed to register global shortcut:', error)
      }
    }

    setupShortcut()

    return () => {
      cleanup?.()
    }
  }, [stopAllScripts, showWindow])

  /**
   * Add console message
   * コンソールメッセージを追加
   */
  const addConsoleMessage = useCallback((level: ConsoleEntry['level'], message: string) => {
    const MAX_CONSOLE_MESSAGES = 1000
    setConsoleOutput((prev) => {
      const newEntry = {
        id: uuidv4(),
        timestamp: new Date(),
        level,
        message,
      }
      // Limit messages to prevent memory leak
      const newMessages = [...prev, newEntry]
      if (newMessages.length > MAX_CONSOLE_MESSAGES) {
        return newMessages.slice(-MAX_CONSOLE_MESSAGES)
      }
      return newMessages
    })
  }, [])

  /**
   * Add new command to script
   * スクリプトに新しいコマンドを追加
   */
  const addCommand = useCallback((type: CommandType) => {
    const newLine: ScriptLine = {
      id: uuidv4(),
      type,
      similarity: type === 'click' || type === 'find' || type === 'if' ? 0.7 : undefined,
      children: type === 'if' || type === 'loop' ? [] : undefined,
      flowConfig: { x: 200, y: 200 },
    }
    setScript((prev) => [...prev, newLine])
    addConsoleMessage('info', `Added ${type} command`)
  }, [addConsoleMessage])

  /**
   * Update script line
   * スクリプト行を更新
   */
  const updateLine = useCallback((id: string, updates: Partial<ScriptLine>) => {
    const updateRecursive = (lines: ScriptLine[]): ScriptLine[] => {
      return lines.map((line) => {
        if (line.id === id) {
          return { ...line, ...updates }
        }
        if (line.children) {
          return { ...line, children: updateRecursive(line.children) }
        }
        return line
      })
    }
    setScript((prev) => updateRecursive(prev))
  }, [])

  /**
   * Delete script line
   * スクリプト行を削除
   */
  const deleteLine = useCallback((id: string) => {
    const deleteRecursive = (lines: ScriptLine[]): ScriptLine[] => {
      return lines
        .filter((line) => line.id !== id)
        .map((line) => {
          if (line.children) {
            return { ...line, children: deleteRecursive(line.children) }
          }
          return line
        })
    }
    setScript((prev) => deleteRecursive(prev))
    if (selectedLineId === id) {
      setSelectedLineId(null)
    }
    addConsoleMessage('info', 'Deleted command')
  }, [selectedLineId, addConsoleMessage])

  /**
   * Run script
   * スクリプトを実行
   */
  const handleRun = useCallback(async () => {
    // First save the script
    addConsoleMessage('info', 'Saving script before execution...')
    let scriptPath = currentFile

    // If no current file, save to temp location
    if (!scriptPath) {
      const savedPath = await saveFileDialog(sourceCode, undefined, isSimpleModeAvailable)
      if (!savedPath) {
        addConsoleMessage('warn', 'Please save the script before running')
        return
      }
      scriptPath = savedPath
      setCurrentFile(savedPath)
    } else {
      // Save current changes
      const savedPath = await saveFileDialog(sourceCode, currentFile ?? undefined, isSimpleModeAvailable)
      if (!savedPath) {
        addConsoleMessage('error', 'Failed to save script')
        return
      }
    }

    // Now run the script
    setIsRunning(true)
    addConsoleMessage('info', `Running script: ${scriptPath}`)

    const result = await runScriptStreaming(scriptPath)
    if (result.success && result.processId) {
      setRunningProcessId(result.processId)
      addConsoleMessage('info', `Script started (process: ${result.processId})`)

      // Hide IDE window if Hide mode is enabled
      // Hideモードが有効ならIDEウィンドウを最小化
      if (hideModeRef.current) {
        await minimizeWindow()
        addConsoleMessage('info', 'IDE window minimized (Shift+Alt+C to stop and restore)')
      }
    } else {
      addConsoleMessage('error', result.error || 'Failed to start script')
      setIsRunning(false)
    }
  }, [addConsoleMessage, currentFile, saveFileDialog, sourceCode, runScriptStreaming, isSimpleModeAvailable, minimizeWindow])

  /**
   * Stop script
   * スクリプトを停止
   */
  const handleStop = useCallback(async () => {
    // Set flag to suppress "exit code: unknown" message in onScriptComplete
    // onScriptCompleteで「exit code: unknown」メッセージを抑制するフラグを設定
    isManualStopRef.current = true
    addConsoleMessage('warn', 'Stopping script execution...')
    const success = await stopAllScripts()
    if (success) {
      setIsRunning(false)
      setRunningProcessId(null)
      addConsoleMessage('info', 'Script execution stopped')

      // Restore IDE window if it was hidden
      // IDEウィンドウが隠れていた場合は復元
      if (hideModeRef.current) {
        await showWindow()
        addConsoleMessage('info', 'IDE window restored')
      }
    } else {
      isManualStopRef.current = false  // Reset flag on failure
      addConsoleMessage('error', 'Failed to stop script')
    }
  }, [addConsoleMessage, stopAllScripts, showWindow])

  /**
   * Clear console
   * コンソールをクリア
   */
  const clearConsole = useCallback(() => {
    setConsoleOutput([])
  }, [])

  /**
   * Create new file
   * 新規ファイルを作成
   */
  const handleNewFile = useCallback(() => {
    setScript([
      {
        id: uuidv4(),
        type: 'start' as const,
        flowConfig: { x: 100, y: 100 },
      },
    ])
    setCurrentFile(null)
    setSelectedLineId(null)
    setSourceCode('')
    setPythonVersion(null)
    setIsSimpleModeAvailable(true)  // New files support Simple mode
    setViewMode('simple')  // Start in Simple mode for new files
    addConsoleMessage('info', 'Created new script')
  }, [addConsoleMessage])

  /**
   * Open file
   * ファイルを開く
   */
  const handleOpenFile = useCallback(async () => {
    addConsoleMessage('info', 'Opening file...')
    const result = await openFileDialog()
    console.log('openFileDialog result:', result)
    if (result) {
      console.log('Script lines count:', result.script.length)
      console.log('Script data:', JSON.stringify(result.script, null, 2).substring(0, 500))
      const scriptToSet = result.script.length > 0 ? result.script : [
        {
          id: uuidv4(),
          type: 'start' as const,
          flowConfig: { x: 100, y: 100 },
        },
      ]
      console.log('Setting script with', scriptToSet.length, 'lines')
      setScript(scriptToSet)
      setCurrentFile(result.path)
      setSelectedLineId(null)
      // Set source code for code mode
      if (result.sourceCode) {
        setSourceCode(result.sourceCode)
      } else {
        setSourceCode('')
      }
      // Set Python version
      setPythonVersion(result.pythonVersion || null)

      // Check if Simple mode is available for this script
      // Simpleモードがこのスクリプトで使用可能かチェック
      const simpleModeAvailable = result.isSimpleMode ?? false
      setIsSimpleModeAvailable(simpleModeAvailable)

      // If not Simple mode script, switch to Code mode
      // Simpleモードスクリプトでなければ、Codeモードに切り替え
      if (!simpleModeAvailable) {
        setViewMode('code')
        addConsoleMessage('info', `Opened: ${result.path} (Code mode - external script)`)
      } else {
        addConsoleMessage('info', `Opened: ${result.path}`)
      }
    } else {
      console.log('openFileDialog returned null - user cancelled or error')
    }
  }, [addConsoleMessage, openFileDialog])

  /**
   * Save file
   * ファイルを保存
   */
  const handleSaveFile = useCallback(async () => {
    addConsoleMessage('info', 'Saving file...')
    // Pass isSimpleModeAvailable to add header if saving from Simple mode
    // Simpleモードから保存する場合はヘッダーを追加
    const savedPath = await saveFileDialog(sourceCode, currentFile || undefined, isSimpleModeAvailable)
    if (savedPath) {
      setCurrentFile(savedPath)
      addConsoleMessage('info', `Saved: ${savedPath}`)
    } else {
      addConsoleMessage('warn', 'Save cancelled')
    }
  }, [addConsoleMessage, saveFileDialog, sourceCode, currentFile, isSimpleModeAvailable])

  /**
   * Handle screen capture for selected command
   * 選択したコマンドの画面キャプチャを処理
   */
  const handleCapture = useCallback(async (): Promise<string | null> => {
    addConsoleMessage('info', 'Starting screen capture...')
    const started = await startCapture()
    if (!started) {
      addConsoleMessage('error', 'Failed to start capture')
      return null
    }

    // Wait for capture to complete (capture window will close itself)
    // Poll for the last capture path
    let attempts = 0
    const maxAttempts = 300 // 30 seconds max
    const pollInterval = 100 // 100ms

    while (attempts < maxAttempts) {
      await new Promise((resolve) => setTimeout(resolve, pollInterval))
      const capturePath = await getLastCapturePath()
      if (capturePath) {
        addConsoleMessage('info', `Captured: ${capturePath}`)
        // Load image as base64 for display
        const base64 = await loadImageAsBase64(capturePath)
        return base64
      }
      attempts++
    }

    addConsoleMessage('warn', 'Capture timed out or cancelled')
    return null
  }, [addConsoleMessage, startCapture, getLastCapturePath, loadImageAsBase64])

  /**
   * Find selected line recursively
   * 選択された行を再帰的に検索
   */
  const selectedLine = useMemo(() => {
    if (!selectedLineId) return null

    const findLine = (lines: ScriptLine[]): ScriptLine | null => {
      for (const line of lines) {
        if (line.id === selectedLineId) return line
        if (line.children) {
          const found = findLine(line.children)
          if (found) return found
        }
      }
      return null
    }

    return findLine(script)
  }, [script, selectedLineId])

  return (
    <div className="h-full flex flex-col bg-dark-bg text-gray-200">
      {/* Header / ヘッダー */}
      <Header
        viewMode={viewMode}
        onViewModeChange={setViewMode}
        isRunning={isRunning}
        onRun={handleRun}
        onStop={handleStop}
        onNewFile={handleNewFile}
        onOpenFile={handleOpenFile}
        onSaveFile={handleSaveFile}
        currentFile={currentFile}
        isSimpleModeAvailable={isSimpleModeAvailable}
        hideMode={hideMode}
        onHideModeChange={setHideMode}
        version={ideVersion}
      />

      {/* Main Content / メインコンテンツ */}
      <div className="flex-1 flex overflow-hidden">
        {/* Toolbox Sidebar / ツールボックスサイドバー */}
        <Toolbox onAddCommand={addCommand} />

        {/* Editor Area / エディタエリア */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Simple/Flow/Code Mode / シンプル/フロー/コードモード */}
          <div className="flex-1 overflow-hidden">
            {viewMode === 'simple' ? (
              <SimpleMode
                script={script}
                selectedLineId={selectedLineId}
                onSelectLine={setSelectedLineId}
                onUpdateLine={updateLine}
                onDeleteLine={deleteLine}
                setScript={setScript}
              />
            ) : viewMode === 'flow' ? (
              <FlowMode
                script={script}
                selectedLineId={selectedLineId}
                onSelectLine={setSelectedLineId}
                onUpdateLine={updateLine}
                onDeleteLine={deleteLine}
              />
            ) : (
              <CodeMode
                sourceCode={sourceCode}
                currentFile={currentFile}
                pythonVersion={pythonVersion}
                onSourceCodeChange={setSourceCode}
                onSave={handleSaveFile}
                imagePatterns={imagePatterns}
              />
            )}
          </div>

          {/* Console Panel / コンソールパネル */}
          {isConsoleVisible && (
            <Console
              entries={consoleOutput}
              onClear={clearConsole}
              onClose={() => setIsConsoleVisible(false)}
            />
          )}
        </div>

        {/* Property Panel / プロパティパネル */}
        {isPropertyPanelVisible && selectedLine && (
          <PropertyPanel
            selectedLine={selectedLine}
            onUpdateLine={updateLine}
            onCapture={handleCapture}
            onClose={() => setIsPropertyPanelVisible(false)}
          />
        )}
      </div>

      {/* Console Toggle (when hidden) / コンソールトグル（非表示時） */}
      {!isConsoleVisible && (
        <button
          onClick={() => setIsConsoleVisible(true)}
          className="fixed bottom-4 right-4 px-4 py-2 bg-dark-surface border border-dark-border rounded-lg hover:bg-dark-hover"
        >
          Show Console
        </button>
      )}
    </div>
  )
}

export default App
