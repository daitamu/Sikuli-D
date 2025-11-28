import {
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
export function Toolbox({ onAddCommand }: ToolboxProps) {
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
        className="flex items-center gap-3 px-3 py-2 rounded-lg cursor-pointer bg-dark-bg hover:bg-dark-hover border border-transparent hover:border-dark-border transition-all"
        title={`${item.label} / ${item.labelJa}`}
      >
        {IconComponent && <IconComponent size={18} />}
        <div className="flex flex-col">
          <span className="text-sm">{item.label}</span>
          <span className="text-xs text-gray-500">{item.labelJa}</span>
        </div>
      </div>
    )
  }

  return (
    <aside className="w-48 bg-dark-surface border-r border-dark-border flex flex-col overflow-hidden">
      {/* Header / ヘッダー */}
      <div className="px-3 py-2 border-b border-dark-border">
        <h2 className="text-sm font-medium text-gray-400">Toolbox</h2>
      </div>

      {/* Scrollable Content / スクロール可能なコンテンツ */}
      <div className="flex-1 overflow-y-auto p-2 space-y-4">
        {/* Logic Section / ロジックセクション */}
        <div>
          <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 px-1">
            Logic
          </h3>
          <div className="space-y-1">{logicItems.map(renderItem)}</div>
        </div>

        {/* Actions Section / アクションセクション */}
        <div>
          <h3 className="text-xs font-medium text-gray-500 uppercase tracking-wider mb-2 px-1">
            Actions
          </h3>
          <div className="space-y-1">{actionItems.map(renderItem)}</div>
        </div>
      </div>

      {/* Help Text / ヘルプテキスト */}
      <div className="px-3 py-2 border-t border-dark-border">
        <p className="text-xs text-gray-500">
          Click or drag to add commands
        </p>
      </div>
    </aside>
  )
}
