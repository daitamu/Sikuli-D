import { useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open, save } from '@tauri-apps/plugin-dialog'
import { readTextFile, writeTextFile, exists, mkdir, readFile } from '@tauri-apps/plugin-fs'
import type { ScriptLine, ConsoleEntry } from '../types/script'
import { parsePythonScript, detectPythonVersion, isSimpleModeScript, addSimpleModeHeader, type PythonVersion } from '../utils/pythonParser'

/**
 * Capture result from backend
 * バックエンドからのキャプチャ結果
 */
interface CaptureResult {
  success: boolean
  message: string
  path?: string
}

/**
 * Capture region coordinates
 * キャプチャ領域の座標
 */
interface CaptureRegion {
  x: number
  y: number
  width: number
  height: number
}

/**
 * Tauri IPC response types
 * Tauri IPCレスポンス型
 */
interface CaptureResponse {
  image: string // Base64 encoded image
  width: number
  height: number
}

interface RunScriptResponse {
  success: boolean
  message?: string
}

/**
 * Script run options for streaming execution
 * ストリーミング実行用のスクリプト実行オプション
 */
interface ScriptRunOptions {
  workingDir?: string
  args?: string[]
  envVars?: Record<string, string>
  debug?: boolean
  timeoutSecs?: number
}

/**
 * Log event payload from Rust
 * Rustからのログイベントペイロード
 */
interface LogEventPayload {
  level: ConsoleEntry['level']
  message: string
}

/**
 * Highlight event payload from Rust
 * Rustからのハイライトイベントペイロード
 */
interface HighlightEventPayload {
  lineId: string
}

/**
 * Custom hook for Tauri IPC communication
 * Tauri IPC通信用カスタムフック
 */
export function useTauri() {
  /**
   * Run script via Tauri
   * Tauriでスクリプトを実行
   */
  const runScript = useCallback(async (script: ScriptLine[]): Promise<RunScriptResponse> => {
    try {
      const response = await invoke<RunScriptResponse>('run_script', {
        script: JSON.stringify(script),
      })
      return response
    } catch (error) {
      console.error('Failed to run script:', error)
      return { success: false, message: String(error) }
    }
  }, [])

  /**
   * Run script with streaming output via Tauri
   * ストリーミング出力付きでTauriでスクリプトを実行
   */
  const runScriptStreaming = useCallback(
    async (
      scriptPath: string,
      options: ScriptRunOptions = {}
    ): Promise<{ success: boolean; processId?: string; error?: string }> => {
      try {
        // Convert options to backend format (snake_case)
        const backendOptions = {
          working_dir: options.workingDir,
          args: options.args || [],
          env_vars: options.envVars || {},
          debug: options.debug || false,
          timeout_secs: options.timeoutSecs,
        }

        const processId = await invoke<string>('run_script_streaming', {
          scriptPath,
          options: backendOptions,
        })
        return { success: true, processId }
      } catch (error) {
        console.error('Failed to run script streaming:', error)
        return { success: false, error: String(error) }
      }
    },
    []
  )

  /**
   * Stop running script by process ID
   * プロセスIDで実行中のスクリプトを停止
   */
  const stopScriptById = useCallback(async (processId: string): Promise<boolean> => {
    try {
      await invoke('stop_script', { processId })
      return true
    } catch (error) {
      console.error('Failed to stop script:', error)
      return false
    }
  }, [])

  /**
   * Stop all running scripts
   * すべての実行中のスクリプトを停止
   */
  const stopAllScripts = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('stop_all_scripts')
      return true
    } catch (error) {
      console.error('Failed to stop all scripts:', error)
      return false
    }
  }, [])

  /**
   * Stop running script (legacy - stops all)
   * 実行中のスクリプトを停止（レガシー - すべて停止）
   */
  const stopScript = useCallback(async (): Promise<boolean> => {
    return stopAllScripts()
  }, [stopAllScripts])

  /**
   * Capture screen via Tauri
   * Tauriで画面をキャプチャ
   */
  const captureScreen = useCallback(async (): Promise<CaptureResponse | null> => {
    try {
      const response = await invoke<CaptureResponse>('capture_screen')
      return response
    } catch (error) {
      console.error('Failed to capture screen:', error)
      return null
    }
  }, [])

  /**
   * Save script file
   * スクリプトファイルを保存
   */
  const saveFile = useCallback(
    async (script: ScriptLine[], filePath?: string): Promise<string | null> => {
      try {
        const savedPath = await invoke<string>('save_file', {
          script: JSON.stringify(script),
          path: filePath,
        })
        return savedPath
      } catch (error) {
        console.error('Failed to save file:', error)
        return null
      }
    },
    []
  )

  /**
   * Load script file
   * スクリプトファイルを読み込み
   */
  const loadFile = useCallback(async (filePath?: string): Promise<ScriptLine[] | null> => {
    try {
      const content = await invoke<string>('load_file', { path: filePath })
      return JSON.parse(content) as ScriptLine[]
    } catch (error) {
      console.error('Failed to load file:', error)
      return null
    }
  }, [])

  /**
   * Get the Python script name from a .sikuli folder path
   * .sikuli フォルダパスから Python スクリプト名を取得
   * e.g., "C:/Test.sikuli" -> "Test.py"
   */
  const getSikuliScriptName = (sikuliPath: string): string => {
    // Get the folder name without .sikuli extension
    // e.g., "C:/path/Test.sikuli" -> "Test"
    const folderName = sikuliPath
      .replace(/[\\/]$/, '') // Remove trailing slash
      .split(/[\\/]/)
      .pop()
      ?.replace(/\.sikuli$/, '') || 'script'
    return `${folderName}.py`
  }

  /**
   * Open file dialog and load script
   * ファイルダイアログを開いてスクリプトを読み込み
   */
  const openFileDialog = useCallback(async (): Promise<{ path: string; script: ScriptLine[]; sourceCode?: string; pythonVersion?: PythonVersion; isSimpleMode?: boolean } | null> => {
    try {
      // Open directory selection dialog for .sikuli folders
      console.log('Opening directory selection dialog...')
      const selected = await open({
        multiple: false,
        directory: true, // Select directories (folders)
        title: 'Open .sikuli Folder',
      })

      console.log('Dialog result:', selected)
      if (!selected) return null

      const folderPath = typeof selected === 'string' ? selected : selected[0]
      console.log('Selected folder path:', folderPath)

      // Check if it's a .sikuli directory
      if (folderPath.endsWith('.sikuli')) {
        // Get Python script name
        const pyName = getSikuliScriptName(folderPath)
        const pyPath = `${folderPath}/${pyName}`

        console.log('Checking for Python script at:', pyPath)
        const pyExists = await exists(pyPath)
        console.log('Python script exists:', pyExists)

        if (pyExists) {
          // Read Python source code (this is the only source of truth)
          const sourceCode = await readTextFile(pyPath)
          console.log('Python content length:', sourceCode.length)

          // Detect Python version for display (no conversion - runtime does that)
          const pythonVersion = detectPythonVersion(sourceCode)
          console.log('Detected Python version:', pythonVersion)

          // Detect if script was created in Simple mode
          const isSimpleMode = isSimpleModeScript(sourceCode)
          console.log('Is Simple mode script:', isSimpleMode)

          // Parse Python to ScriptLine[] for Simple/Flow mode display
          console.log(`Parsing Python script: ${pyPath}`)
          const script = parsePythonScript(sourceCode, folderPath)
          console.log('Parsed script lines:', script.length)

          return { path: folderPath, script, sourceCode, pythonVersion, isSimpleMode }
        }

        console.error(`No Python script found in ${folderPath}. Expected ${pyName}`)
        return null
      }

      // For non-.sikuli paths, try to load as JSON
      console.log('Not a .sikuli folder, trying to load as JSON')
      const content = await readTextFile(folderPath)
      const script = JSON.parse(content) as ScriptLine[]
      return { path: folderPath, script }
    } catch (error) {
      console.error('Failed to open file:', error)
      return null
    }
  }, [])

  /**
   * Save file dialog and save Python script
   * ファイルダイアログで保存先を選択してPythonスクリプトを保存
   * @param sourceCode - Python source code to save
   * @param currentPath - Current file path (optional)
   * @param isSimpleMode - If true, add Simple mode header (optional, default: false)
   */
  const saveFileDialog = useCallback(
    async (sourceCode: string, currentPath?: string, isSimpleMode: boolean = false): Promise<string | null> => {
      try {
        let filePath = currentPath

        if (!filePath) {
          const selected = await save({
            filters: [
              { name: 'Sikuli Script', extensions: ['sikuli'] },
            ],
            defaultPath: 'untitled.sikuli',
          })

          if (!selected) return null
          filePath = selected
        }

        // Add Simple mode header if saving from Simple mode
        // Simpleモードから保存する場合はヘッダーを追加
        const codeToSave = isSimpleMode ? addSimpleModeHeader(sourceCode) : sourceCode

        // If it's a .sikuli directory, save as {foldername}.py inside
        if (filePath.endsWith('.sikuli')) {
          // Create directory if it doesn't exist
          const dirExists = await exists(filePath)
          if (!dirExists) {
            await mkdir(filePath, { recursive: true })
          }
          const pyName = getSikuliScriptName(filePath)
          const pyPath = `${filePath}/${pyName}`
          await writeTextFile(pyPath, codeToSave)
          return filePath
        }

        // For .py files, save directly
        await writeTextFile(filePath, codeToSave)
        return filePath
      } catch (error) {
        console.error('Failed to save file:', error)
        return null
      }
    },
    []
  )

  /**
   * Start capture overlay
   * キャプチャオーバーレイを開始
   */
  const startCapture = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('start_capture')
      return true
    } catch (error) {
      console.error('Failed to start capture:', error)
      return false
    }
  }, [])

  /**
   * Capture a specific region
   * 特定の領域をキャプチャ
   */
  const captureRegion = useCallback(async (region: CaptureRegion): Promise<CaptureResult | null> => {
    try {
      const result = await invoke<CaptureResult>('capture_region', { region })
      return result
    } catch (error) {
      console.error('Failed to capture region:', error)
      return null
    }
  }, [])

  /**
   * Capture full screen
   * 画面全体をキャプチャ
   */
  const captureFullScreen = useCallback(async (): Promise<CaptureResult | null> => {
    try {
      const result = await invoke<CaptureResult>('capture_full_screen')
      return result
    } catch (error) {
      console.error('Failed to capture full screen:', error)
      return null
    }
  }, [])

  /**
   * Cancel capture
   * キャプチャをキャンセル
   */
  const cancelCapture = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('cancel_capture')
      return true
    } catch (error) {
      console.error('Failed to cancel capture:', error)
      return false
    }
  }, [])

  /**
   * Get last capture path
   * 最後のキャプチャパスを取得
   */
  const getLastCapturePath = useCallback(async (): Promise<string | null> => {
    try {
      const path = await invoke<string | null>('get_last_capture_path')
      return path
    } catch (error) {
      console.error('Failed to get last capture path:', error)
      return null
    }
  }, [])

  /**
   * Load image file as base64 data URL
   * 画像ファイルをbase64データURLとして読み込み
   */
  const loadImageAsBase64 = useCallback(async (imagePath: string): Promise<string | null> => {
    try {
      const data = await readFile(imagePath)
      const base64 = btoa(String.fromCharCode(...data))
      // Detect image type from extension
      const ext = imagePath.toLowerCase().split('.').pop()
      const mimeType = ext === 'png' ? 'image/png' : ext === 'jpg' || ext === 'jpeg' ? 'image/jpeg' : 'image/png'
      return `data:${mimeType};base64,${base64}`
    } catch (error) {
      console.error('Failed to load image:', error)
      return null
    }
  }, [])

  /**
   * Minimize IDE window (for Hide mode)
   * IDEウィンドウを最小化（Hideモード用）
   */
  const minimizeWindow = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('minimize_window')
      return true
    } catch (error) {
      console.error('Failed to minimize window:', error)
      return false
    }
  }, [])

  /**
   * Show and restore IDE window
   * IDEウィンドウを表示・復元
   */
  const showWindow = useCallback(async (): Promise<boolean> => {
    try {
      await invoke('show_window')
      return true
    } catch (error) {
      console.error('Failed to show window:', error)
      return false
    }
  }, [])

  /**
   * Get IDE version (auto-incremented build version)
   * IDEバージョンを取得（自動インクリメントされたビルドバージョン）
   */
  const getIdeVersion = useCallback(async (): Promise<string> => {
    try {
      return await invoke<string>('get_ide_version')
    } catch (error) {
      console.error('Failed to get IDE version:', error)
      return '0.8.0' // fallback
    }
  }, [])

  return {
    runScript,
    runScriptStreaming,
    stopScript,
    stopScriptById,
    stopAllScripts,
    captureScreen,
    saveFile,
    loadFile,
    openFileDialog,
    saveFileDialog,
    startCapture,
    captureRegion,
    captureFullScreen,
    cancelCapture,
    getLastCapturePath,
    loadImageAsBase64,
    minimizeWindow,
    showWindow,
    getIdeVersion,
  }
}

/**
 * Script output event payload from Rust
 * Rustからのスクリプト出力イベントペイロード
 */
interface ScriptOutputPayload {
  process_id: string
  line: string
}

/**
 * Script complete event payload from Rust
 * Rustからのスクリプト完了イベントペイロード
 */
interface ScriptCompletePayload {
  process_id: string
  exit_code: number | null
}

/**
 * Custom hook for listening to Tauri events
 * Tauriイベントリスニング用カスタムフック
 */
export function useTauriEvents(
  onLog?: (entry: Omit<ConsoleEntry, 'id' | 'timestamp'>) => void,
  onHighlight?: (lineId: string) => void,
  onScriptEnd?: () => void,
  onScriptOutput?: (processId: string, line: string, isError: boolean) => void,
  onScriptComplete?: (processId: string, exitCode: number | null) => void
) {
  useEffect(() => {
    // Track if effect has been cancelled (for StrictMode double-invocation)
    // StrictModeの二重呼び出し対策でキャンセル状態を追跡
    let cancelled = false
    const unlisteners: UnlistenFn[] = []

    const setupListeners = async () => {
      // Listen for log messages / ログメッセージをリッスン
      if (onLog && !cancelled) {
        const unlistenLog = await listen<LogEventPayload>('log_message', (event) => {
          onLog({ level: event.payload.level, message: event.payload.message })
        })
        if (cancelled) {
          unlistenLog()
        } else {
          unlisteners.push(unlistenLog)
        }
      }

      // Listen for highlight events / ハイライトイベントをリッスン
      if (onHighlight && !cancelled) {
        const unlistenHighlight = await listen<HighlightEventPayload>(
          'highlight_line',
          (event) => {
            onHighlight(event.payload.lineId)
          }
        )
        if (cancelled) {
          unlistenHighlight()
        } else {
          unlisteners.push(unlistenHighlight)
        }
      }

      // Listen for script end events / スクリプト終了イベントをリッスン
      if (onScriptEnd && !cancelled) {
        const unlistenEnd = await listen('script_end', () => {
          onScriptEnd()
        })
        if (cancelled) {
          unlistenEnd()
        } else {
          unlisteners.push(unlistenEnd)
        }
      }

      // Listen for script stdout / スクリプト標準出力をリッスン
      if (onScriptOutput && !cancelled) {
        const unlistenStdout = await listen<ScriptOutputPayload>('script-stdout', (event) => {
          onScriptOutput(event.payload.process_id, event.payload.line, false)
        })
        if (cancelled) {
          unlistenStdout()
        } else {
          unlisteners.push(unlistenStdout)
        }

        // Listen for script stderr / スクリプト標準エラーをリッスン
        if (!cancelled) {
          const unlistenStderr = await listen<ScriptOutputPayload>('script-stderr', (event) => {
            onScriptOutput(event.payload.process_id, event.payload.line, true)
          })
          if (cancelled) {
            unlistenStderr()
          } else {
            unlisteners.push(unlistenStderr)
          }
        }
      }

      // Listen for script complete events / スクリプト完了イベントをリッスン
      if (onScriptComplete && !cancelled) {
        const unlistenComplete = await listen<ScriptCompletePayload>('script-complete', (event) => {
          onScriptComplete(event.payload.process_id, event.payload.exit_code)
        })
        if (cancelled) {
          unlistenComplete()
        } else {
          unlisteners.push(unlistenComplete)
        }
      }
    }

    setupListeners()

    return () => {
      cancelled = true
      unlisteners.forEach((unlisten) => unlisten())
    }
  }, [onLog, onHighlight, onScriptEnd, onScriptOutput, onScriptComplete])
}
