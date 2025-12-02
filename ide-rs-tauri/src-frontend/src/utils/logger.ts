/**
 * Development Logger Utility
 * 開発用ロガーユーティリティ
 *
 * Logs are only output in development mode.
 * ログは開発モードでのみ出力されます。
 */

const isDev = import.meta.env.DEV

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LoggerOptions {
  prefix?: string
  enabled?: boolean
}

class Logger {
  private prefix: string
  private enabled: boolean

  constructor(options: LoggerOptions = {}) {
    this.prefix = options.prefix || ''
    this.enabled = options.enabled ?? isDev
  }

  private formatMessage(_level: LogLevel, message: string): string {
    const timestamp = new Date().toISOString().slice(11, 23)
    const prefixStr = this.prefix ? `[${this.prefix}]` : ''
    return `${timestamp} ${prefixStr} ${message}`
  }

  debug(message: string, ...args: unknown[]): void {
    if (this.enabled) {
      console.log(this.formatMessage('debug', message), ...args)
    }
  }

  info(message: string, ...args: unknown[]): void {
    if (this.enabled) {
      console.info(this.formatMessage('info', message), ...args)
    }
  }

  warn(message: string, ...args: unknown[]): void {
    if (this.enabled) {
      console.warn(this.formatMessage('warn', message), ...args)
    }
  }

  error(message: string, ...args: unknown[]): void {
    // Errors are always logged
    console.error(this.formatMessage('error', message), ...args)
  }
}

// Pre-configured loggers for different modules
export const appLogger = new Logger({ prefix: 'App' })
export const imageLogger = new Logger({ prefix: 'ImageLoader' })
export const widgetLogger = new Logger({ prefix: 'ImageWidget' })
export const tauriLogger = new Logger({ prefix: 'Tauri' })
export const debugApiLogger = new Logger({ prefix: 'Debug' })

// Generic logger factory
export function createLogger(prefix: string, enabled?: boolean): Logger {
  return new Logger({ prefix, enabled })
}

// Default export for simple usage
export default new Logger()
