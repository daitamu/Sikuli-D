import { useCallback, useState } from 'react'
import { Camera, Image, Trash2, X } from 'lucide-react'
import type { ScriptLine, CommandType } from '../types/script'

interface PropertyPanelProps {
  selectedLine: ScriptLine | null
  onUpdateLine: (id: string, updates: Partial<ScriptLine>) => void
  onCapture: () => Promise<string | null>
  onClose: () => void
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
 * Property Panel Component - Edit selected command properties
 * プロパティパネルコンポーネント - 選択したコマンドのプロパティを編集
 */
export function PropertyPanel({
  selectedLine,
  onUpdateLine,
  onCapture,
  onClose,
}: PropertyPanelProps) {
  const [isCapturing, setIsCapturing] = useState(false)

  if (!selectedLine) {
    return null
  }

  const label = commandLabels[selectedLine.type]
  const needsTarget = ['click', 'find', 'if'].includes(selectedLine.type)

  /**
   * Handle capture button click
   * キャプチャボタンクリックを処理
   */
  const handleCapture = useCallback(async () => {
    setIsCapturing(true)
    try {
      const imagePath = await onCapture()
      if (imagePath) {
        onUpdateLine(selectedLine.id, { target: imagePath })
      }
    } finally {
      setIsCapturing(false)
    }
  }, [onCapture, onUpdateLine, selectedLine.id])

  /**
   * Clear target image
   * ターゲット画像をクリア
   */
  const handleClearTarget = useCallback(() => {
    onUpdateLine(selectedLine.id, { target: undefined })
  }, [onUpdateLine, selectedLine.id])

  return (
    <aside className="w-64 bg-dark-surface border-l border-dark-border flex flex-col overflow-hidden">
      {/* Header / ヘッダー */}
      <div className="px-3 py-2 border-b border-dark-border flex items-center justify-between">
        <h2 className="text-sm font-medium text-gray-400">Properties</h2>
        <button
          onClick={onClose}
          className="p-1 text-gray-500 hover:text-gray-300 hover:bg-dark-hover rounded"
          title="Close / 閉じる"
        >
          <X size={16} />
        </button>
      </div>

      {/* Content / コンテンツ */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {/* Command Type / コマンドタイプ */}
        <div>
          <label className="block text-xs text-gray-500 mb-1">Command Type</label>
          <div className="text-sm font-medium">
            {label.en} <span className="text-gray-500">({label.ja})</span>
          </div>
        </div>

        {/* Target Image / ターゲット画像 */}
        {needsTarget && (
          <div>
            <label className="block text-xs text-gray-500 mb-1">
              Target Image / ターゲット画像
            </label>
            <div className="space-y-2">
              {/* Image Preview / 画像プレビュー */}
              <div className="relative w-full h-32 bg-dark-bg rounded border border-dark-border overflow-hidden">
                {selectedLine.target ? (
                  <>
                    <img
                      src={selectedLine.target}
                      alt="target"
                      className="w-full h-full object-contain"
                    />
                    <button
                      onClick={handleClearTarget}
                      className="absolute top-1 right-1 p-1 bg-red-600 hover:bg-red-700 rounded"
                      title="Clear / クリア"
                    >
                      <Trash2 size={14} />
                    </button>
                  </>
                ) : (
                  <div className="w-full h-full flex items-center justify-center text-gray-500">
                    <Image size={32} />
                  </div>
                )}
              </div>

              {/* Capture Button / キャプチャボタン */}
              <button
                onClick={handleCapture}
                disabled={isCapturing}
                className="w-full flex items-center justify-center gap-2 px-3 py-2 bg-sikuli-600 hover:bg-sikuli-700 disabled:bg-gray-600 rounded transition-colors"
              >
                <Camera size={16} />
                <span className="text-sm">
                  {isCapturing ? 'Capturing...' : 'Capture Screen'}
                </span>
              </button>
            </div>
          </div>
        )}

        {/* Similarity / 一致率 */}
        {selectedLine.similarity !== undefined && (
          <div>
            <label className="block text-xs text-gray-500 mb-1">
              Similarity / 一致率: {Math.round(selectedLine.similarity * 100)}%
            </label>
            <input
              type="range"
              value={selectedLine.similarity}
              onChange={(e) =>
                onUpdateLine(selectedLine.id, { similarity: parseFloat(e.target.value) })
              }
              min="0"
              max="1"
              step="0.05"
              className="w-full"
            />
            <div className="flex justify-between text-xs text-gray-500">
              <span>0%</span>
              <span>50%</span>
              <span>100%</span>
            </div>
          </div>
        )}

        {/* Text Input for Type command / Type コマンドのテキスト入力 */}
        {selectedLine.type === 'type' && (
          <div>
            <label className="block text-xs text-gray-500 mb-1">
              Text to Type / 入力テキスト
            </label>
            <textarea
              value={selectedLine.params || ''}
              onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
              placeholder="Enter text..."
              rows={3}
              className="w-full px-3 py-2 bg-dark-bg border border-dark-border rounded text-sm resize-none"
            />
          </div>
        )}

        {/* Wait Time / 待機時間 */}
        {selectedLine.type === 'wait' && (
          <div>
            <label className="block text-xs text-gray-500 mb-1">
              Wait Time (seconds) / 待機時間（秒）
            </label>
            <div className="flex items-center gap-2">
              <input
                type="number"
                value={selectedLine.params || '1'}
                onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
                min="0"
                step="0.1"
                className="flex-1 px-3 py-2 bg-dark-bg border border-dark-border rounded text-sm"
              />
              <span className="text-sm text-gray-500">sec</span>
            </div>
          </div>
        )}

        {/* Loop Count / ループ回数 */}
        {selectedLine.type === 'loop' && (
          <div>
            <label className="block text-xs text-gray-500 mb-1">
              Loop Count / ループ回数
            </label>
            <input
              type="number"
              value={selectedLine.params || '1'}
              onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
              min="1"
              className="w-full px-3 py-2 bg-dark-bg border border-dark-border rounded text-sm"
            />
          </div>
        )}
      </div>
    </aside>
  )
}
