import { useState, useCallback } from 'react'
import { v4 as uuidv4 } from 'uuid'
import { convertFileSrc } from '@tauri-apps/api/core'
import {
  ChevronRight,
  ChevronDown,
  GripVertical,
  Trash2,
  Image,
  MousePointer2,
  Type,
  Clock,
  Search,
  GitBranch,
  Repeat,
  Play,
  type LucideIcon,
} from 'lucide-react'
import type { ScriptLine, CommandType } from '../types/script'

interface SimpleModeProps {
  script: ScriptLine[]
  selectedLineId: string | null
  onSelectLine: (id: string | null) => void
  onUpdateLine: (id: string, updates: Partial<ScriptLine>) => void
  onDeleteLine: (id: string) => void
  setScript: React.Dispatch<React.SetStateAction<ScriptLine[]>>
}

/**
 * Command type icon map
 * コマンドタイプのアイコンマップ
 */
const commandIcons: Record<CommandType, LucideIcon> = {
  start: Play,
  click: MousePointer2,
  type: Type,
  wait: Clock,
  find: Search,
  if: GitBranch,
  loop: Repeat,
}

/**
 * Command type labels
 * コマンドタイプのラベル
 */
const commandLabels: Record<CommandType, { en: string; ja: string }> = {
  start: { en: 'Start', ja: '開始' },
  click: { en: 'Click', ja: 'クリック' },
  type: { en: 'Type', ja: '入力' },
  wait: { en: 'Wait', ja: '待機' },
  find: { en: 'Find', ja: '検索' },
  if: { en: 'If Image Exists', ja: '画像があれば' },
  loop: { en: 'Loop', ja: 'ループ' },
}

/**
 * Simple Mode Component - List view for script editing
 * シンプルモードコンポーネント - スクリプト編集のリストビュー
 */
export function SimpleMode({
  script,
  selectedLineId,
  onSelectLine,
  onUpdateLine,
  onDeleteLine,
  setScript,
}: SimpleModeProps) {
  const [dragOverId, setDragOverId] = useState<string | null>(null)

  /**
   * Handle drag start
   * ドラッグ開始を処理
   */
  const handleDragStart = (e: React.DragEvent, id: string) => {
    e.dataTransfer.setData('script-line-id', id)
    e.dataTransfer.effectAllowed = 'move'
  }

  /**
   * Handle drag over
   * ドラッグオーバーを処理
   */
  const handleDragOver = (e: React.DragEvent, id: string) => {
    e.preventDefault()
    e.dataTransfer.dropEffect = 'move'
    setDragOverId(id)
  }

  /**
   * Handle drop for reordering
   * 並べ替えのためのドロップを処理
   */
  const handleDrop = useCallback(
    (e: React.DragEvent, targetId: string) => {
      e.preventDefault()
      setDragOverId(null)

      const sourceId = e.dataTransfer.getData('script-line-id')
      const commandType = e.dataTransfer.getData('command-type') as CommandType

      if (commandType) {
        // New command from toolbox
        const newLine: ScriptLine = {
          id: uuidv4(),
          type: commandType,
          similarity: ['click', 'find', 'if'].includes(commandType) ? 0.7 : undefined,
          children: ['if', 'loop'].includes(commandType) ? [] : undefined,
          flowConfig: { x: 200, y: 200 },
        }

        setScript((prev) => {
          const targetIndex = prev.findIndex((line) => line.id === targetId)
          if (targetIndex === -1) return [...prev, newLine]
          const newScript = [...prev]
          newScript.splice(targetIndex + 1, 0, newLine)
          return newScript
        })
      } else if (sourceId && sourceId !== targetId) {
        // Reorder existing lines
        setScript((prev) => {
          const sourceIndex = prev.findIndex((line) => line.id === sourceId)
          const targetIndex = prev.findIndex((line) => line.id === targetId)
          if (sourceIndex === -1 || targetIndex === -1) return prev

          const newScript = [...prev]
          const [removed] = newScript.splice(sourceIndex, 1)
          newScript.splice(targetIndex, 0, removed)
          return newScript
        })
      }
    },
    [setScript]
  )

  /**
   * Handle drop on empty area
   * 空きエリアへのドロップを処理
   */
  const handleDropOnEmpty = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault()
      const commandType = e.dataTransfer.getData('command-type') as CommandType
      if (commandType) {
        const newLine: ScriptLine = {
          id: uuidv4(),
          type: commandType,
          similarity: ['click', 'find', 'if'].includes(commandType) ? 0.7 : undefined,
          children: ['if', 'loop'].includes(commandType) ? [] : undefined,
          flowConfig: { x: 200, y: 200 },
        }
        setScript((prev) => [...prev, newLine])
      }
    },
    [setScript]
  )

  /**
   * Render a single script line
   * スクリプト行を描画
   */
  const renderLine = (line: ScriptLine, depth: number = 0) => {
    const IconComponent = commandIcons[line.type]
    const label = commandLabels[line.type]
    const isSelected = line.id === selectedLineId
    const isContainer = line.type === 'if' || line.type === 'loop'

    return (
      <div key={line.id} className="group">
        <div
          draggable
          onDragStart={(e) => handleDragStart(e, line.id)}
          onDragOver={(e) => handleDragOver(e, line.id)}
          onDrop={(e) => handleDrop(e, line.id)}
          onDragLeave={() => setDragOverId(null)}
          onClick={() => onSelectLine(line.id)}
          className={`script-line transition-all duration-200 ${
            isSelected 
              ? 'bg-sikuli-500/10 border-l-4 border-l-sikuli-500 border-y border-y-transparent' 
              : 'border-l-4 border-l-transparent border-b border-b-dark-border/30 hover:bg-dark-surface'
          } ${
            dragOverId === line.id ? 'drop-target ring-2 ring-sikuli-500 ring-inset' : ''
          }`}
          style={{ paddingLeft: `${depth * 24 + 12}px` }}
        >
          {/* Drag Handle / ドラッグハンドル */}
          <div className="p-1 text-gray-600 group-hover:text-gray-400 cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-100 transition-opacity">
            <GripVertical size={14} />
          </div>

          {/* Collapse Toggle / 折りたたみトグル */}
          {isContainer && (
            <button
              onClick={(e) => {
                e.stopPropagation()
                onUpdateLine(line.id, { isCollapsed: !line.isCollapsed })
              }}
              className="p-0.5 text-gray-500 hover:text-gray-300 hover:bg-dark-hover rounded transition-colors"
            >
              {line.isCollapsed ? (
                <ChevronRight size={14} />
              ) : (
                <ChevronDown size={14} />
              )}
            </button>
          )}

          {/* Command Icon / コマンドアイコン */}
          <div className={`p-1.5 rounded-md ${isSelected ? 'bg-sikuli-500 text-white shadow-md' : 'bg-dark-bg text-gray-400 group-hover:text-sikuli-400 group-hover:bg-dark-surface border border-dark-border/50'}`}>
            <IconComponent size={14} strokeWidth={2} />
          </div>

          {/* Command Label / コマンドラベル */}
          <div className="flex-1 flex flex-col justify-center ml-1">
            <span className={`text-sm font-medium ${isSelected ? 'text-sikuli-100' : 'text-gray-300'}`}>
              {label.en}
            </span>
            {/* Optional: show JA label on hover or subtitle? Keeping it minimal for now */}
          </div>

          {/* Image Thumbnail / 画像サムネイル */}
          {line.target && (
            <div className="relative w-8 h-8 bg-dark-bg rounded-md border border-dark-border overflow-hidden flex items-center justify-center shrink-0 shadow-sm group-hover:border-sikuli-500/50 transition-colors">
              <div className="absolute inset-0 bg-[url('/transparent-grid.png')] opacity-20"></div>
              {line.target.startsWith('data:') ? (
                <img src={line.target} alt="target" className="w-full h-full object-contain relative z-10" />
              ) : line.target.match(/\.(png|jpg|jpeg|gif|bmp)$/i) ? (
                <img
                  src={convertFileSrc(line.target)}
                  alt="target"
                  className="w-full h-full object-contain relative z-10"
                  onError={(e) => {
                    const target = e.target as HTMLImageElement
                    target.style.display = 'none'
                    target.parentElement?.classList.add('show-fallback')
                  }}
                />
              ) : (
                <Image size={14} className="text-gray-500 relative z-10" />
              )}
            </div>
          )}

          {/* Parameters / パラメータ */}
          {line.type === 'type' && (
            <input
              type="text"
              value={line.params || ''}
              onChange={(e) => onUpdateLine(line.id, { params: e.target.value })}
              onClick={(e) => e.stopPropagation()}
              placeholder="Type here..."
              className="w-40 px-2 py-1 text-xs bg-dark-bg border border-dark-border rounded focus:border-sikuli-500 focus:ring-1 focus:ring-sikuli-500 outline-none transition-all text-gray-200 font-mono placeholder-gray-600"
            />
          )}

          {line.type === 'wait' && (
            <div className="flex items-center gap-1 bg-dark-bg border border-dark-border rounded px-2 py-1 focus-within:border-sikuli-500 focus-within:ring-1 focus-within:ring-sikuli-500 transition-all">
              <input
                type="number"
                value={line.params || '1'}
                onChange={(e) => onUpdateLine(line.id, { params: e.target.value })}
                onClick={(e) => e.stopPropagation()}
                min="0"
                step="0.1"
                className="w-12 bg-transparent border-none outline-none text-xs font-mono text-gray-200 text-right p-0"
              />
              <span className="text-[10px] text-gray-500 font-medium">s</span>
            </div>
          )}

          {/* Similarity / 一致率 */}
          {line.similarity !== undefined && (
            <div className="flex items-center gap-2 bg-dark-bg/50 px-2 py-1 rounded border border-transparent hover:border-dark-border transition-colors">
              <input
                type="range"
                value={line.similarity}
                onChange={(e) =>
                  onUpdateLine(line.id, { similarity: parseFloat(e.target.value) })
                }
                onClick={(e) => e.stopPropagation()}
                min="0"
                max="1"
                step="0.05"
                className="w-16 accent-sikuli-500 h-1 bg-dark-surface rounded cursor-pointer"
              />
              <span className="text-[10px] font-mono text-gray-400 w-8 text-right">
                {Math.round(line.similarity * 100)}%
              </span>
            </div>
          )}

          {/* Delete Button / 削除ボタン */}
          <button
            onClick={(e) => {
              e.stopPropagation()
              onDeleteLine(line.id)
            }}
            className="ml-2 p-1.5 text-gray-600 hover:text-red-400 hover:bg-red-500/10 rounded opacity-0 group-hover:opacity-100 transition-all"
            title="Delete line"
          >
            <Trash2 size={14} />
          </button>
        </div>

        {/* Children (for If/Loop) / 子要素（If/Loop用） */}
        {isContainer && !line.isCollapsed && line.children && (
          <div className="border-l border-dark-border/30 ml-[29px] relative">
             {/* Connection line visual aid */}
             <div className="absolute top-0 bottom-0 left-0 w-px bg-gradient-to-b from-dark-border/50 to-transparent"></div>
            {line.children.map((child) => renderLine(child, depth + 1))}
            {line.children.length === 0 && (
              <div className="ml-3 my-1 px-3 py-2 text-xs text-gray-600 italic border border-dashed border-dark-border/50 rounded bg-dark-bg/30">
                Drop commands here
              </div>
            )}
          </div>
        )}
      </div>
    )
  }

  return (
    <div
      className="h-full overflow-y-auto bg-dark-bg"
      onDragOver={(e) => e.preventDefault()}
      onDrop={handleDropOnEmpty}
    >
      {/* Script Lines / スクリプト行 */}
      {script.map((line) => renderLine(line))}

      {/* Empty State / 空の状態 */}
      {script.length === 0 && (
        <div className="flex flex-col items-center justify-center h-full text-gray-500">
          <p className="mb-2">No commands yet</p>
          <p className="text-sm">Drag commands from the toolbox or click to add</p>
        </div>
      )}
    </div>
  )
}
