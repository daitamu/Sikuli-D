import {
  X,
  MousePointer2,
  Type,
  Clock,
  Search,
  GitBranch,
  Repeat,
  Play,
  type LucideIcon,
} from 'lucide-react'
import type { CommandType, ToolboxItem } from '../types/script'

interface ToolboxProps {
  onAddCommand: (type: CommandType) => void
  isVisible?: boolean
  onClose?: () => void
}

/**
 * Toolbox items definition
 * ツールボックスアイテム定義
 */
const toolboxItems: ToolboxItem[] = [
  // Logic Commands / ロジックコマンド
  {
    type: 'start',
    label: 'Start',
    labelJa: '開始',
    icon: 'Play',
    category: 'logic',
  },
  {
    type: 'if',
    label: 'If Image Exists',
    labelJa: '画像があれば',
    icon: 'GitBranch',
    category: 'logic',
  },
  {
    type: 'loop',
    label: 'Loop',
    labelJa: 'ループ',
    icon: 'Repeat',
    category: 'logic',
  },
  // Action Commands / アクションコマンド
  {
    type: 'click',
    label: 'Click',
    labelJa: 'クリック',
    icon: 'MousePointer2',
    category: 'actions',
  },
  {
    type: 'type',
    label: 'Type',
    labelJa: '入力',
    icon: 'Type',
    category: 'actions',
  },
  {
    type: 'wait',
    label: 'Wait',
    labelJa: '待機',
    icon: 'Clock',
    category: 'actions',
  },
  {
    type: 'find',
    label: 'Find',
    labelJa: '検索',
    icon: 'Search',
    category: 'actions',
  },
]

/**
 * Icon component map
 * アイコンコンポーネントマップ
 */
const iconMap: Record<string, LucideIcon> = {
  Play,
  GitBranch,
  Repeat,
  MousePointer2,
  Type,
  Clock,
  Search,
}

/**
 * Toolbox Component - Sidebar with draggable commands
 * ツールボックスコンポーネント - ドラッグ可能なコマンドのサイドバー
 */
export function Toolbox({ onAddCommand, isVisible = true, onClose }: ToolboxProps) {
  const logicItems = toolboxItems.filter((item) => item.category === 'logic')
  const actionItems = toolboxItems.filter((item) => item.category === 'actions')

  const handleDragStart = (e: React.DragEvent, type: CommandType) => {
    e.dataTransfer.setData('command-type', type)
    e.dataTransfer.effectAllowed = 'copy'
  }

  const renderItem = (item: ToolboxItem) => {
    const IconComponent = iconMap[item.icon]
    return (
      <div
        key={item.type}
        draggable
        onDragStart={(e) => handleDragStart(e, item.type)}
        onClick={() => onAddCommand(item.type)}
        className="flex items-center gap-3 px-3 py-2.5 rounded-lg cursor-grab active:cursor-grabbing bg-dark-surface border border-dark-border/50 hover:border-sikuli-500/50 hover:bg-dark-hover hover:shadow-md transition-all duration-200 group"
        title={`${item.label} / ${item.labelJa}`}
      >
        {IconComponent && <div className="p-1.5 bg-dark-bg rounded text-sikuli-400 group-hover:text-sikuli-300 group-hover:bg-sikuli-500/10 transition-colors"><IconComponent size={16} strokeWidth={2} /></div>}
        <div className="flex flex-col">
          <span className="text-xs font-medium text-gray-300 group-hover:text-white transition-colors">{item.label}</span>
          <span className="text-[10px] text-gray-500 group-hover:text-gray-400 transition-colors">{item.labelJa}</span>
        </div>
      </div>
    )
  }

  return (
    <aside className={`w-56 bg-dark-sidebar border-r border-dark-border flex flex-col overflow-hidden transition-transform duration-300 ${
      isVisible ? 'translate-x-0' : '-translate-x-full'
    } lg:translate-x-0 absolute lg:relative z-40 h-full`}>
      {/* Header / ヘッダー */}
      <div className="px-4 py-3 border-b border-dark-border/50 bg-dark-sidebar flex items-center justify-between">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Toolbox</h2>
        <button
          onClick={onClose}
          className="lg:hidden p-1 text-gray-400 hover:text-gray-100 hover:bg-dark-hover rounded transition-colors"
          title="Close"
        >
          <X size={14} />
        </button>
      </div>

      {/* Scrollable Content / スクロール可能なコンテンツ */}
      <div className="flex-1 overflow-y-auto p-3 space-y-6">
        {/* Logic Section / ロジックセクション */}
        <div>
          <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3 px-1 opacity-70">
            Flow Control
          </h3>
          <div className="space-y-2">{logicItems.map(renderItem)}</div>
        </div>

        {/* Actions Section / アクションセクション */}
        <div>
          <h3 className="text-[10px] font-bold text-gray-500 uppercase tracking-widest mb-3 px-1 opacity-70">
            Actions
          </h3>
          <div className="space-y-2">{actionItems.map(renderItem)}</div>
        </div>
      </div>

      {/* Help Text / ヘルプテキスト */}
      <div className="px-4 py-3 border-t border-dark-border/50 bg-dark-sidebar text-center">
        <p className="text-[10px] text-gray-500">
          Drag items to the editor
        </p>
      </div>
    </aside>
  )
}
