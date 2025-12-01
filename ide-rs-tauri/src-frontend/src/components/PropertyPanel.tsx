import { useCallback, useState } from 'react'
import { Camera, Image, Trash2, X, Clock, Repeat } from 'lucide-react'
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
    <aside className="w-72 bg-dark-sidebar border-l border-dark-border flex flex-col overflow-hidden shadow-xl z-10">
      {/* Header / ヘッダー */}
      <div className="px-4 py-3 border-b border-dark-border/50 flex items-center justify-between bg-dark-sidebar">
        <h2 className="text-xs font-semibold text-gray-400 uppercase tracking-wider">Properties</h2>
        <button
          onClick={onClose}
          className="p-1.5 text-gray-500 hover:text-gray-200 hover:bg-dark-hover rounded-md transition-colors"
          title="Close / 閉じる"
        >
          <X size={14} />
        </button>
      </div>

      {/* Content / コンテンツ */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {/* Command Type / コマンドタイプ */}
        <div className="bg-dark-surface p-3 rounded-lg border border-dark-border/50">
          <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-1">Command Type</label>
          <div className="text-sm font-medium text-gray-200 flex items-baseline gap-2">
            {label.en} <span className="text-xs text-gray-500 font-normal">({label.ja})</span>
          </div>
        </div>

        {/* Target Image / ターゲット画像 */}
        {needsTarget && (
          <div>
            <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-2">
              Target Image
            </label>
            <div className="space-y-2">
              {/* Image Preview / 画像プレビュー */}
              <div className="relative w-full h-40 bg-dark-bg/50 rounded-lg border border-dark-border overflow-hidden group">
                {selectedLine.target ? (
                  <>
                    <div className="absolute inset-0 bg-[url('/transparent-grid.png')] opacity-10"></div>
                    <img
                      src={selectedLine.target}
                      alt="target"
                      className="relative w-full h-full object-contain p-2"
                    />
                    <button
                      onClick={handleClearTarget}
                      className="absolute top-2 right-2 p-1.5 bg-red-500/80 hover:bg-red-600 text-white rounded-md opacity-0 group-hover:opacity-100 transition-all transform translate-y-1 group-hover:translate-y-0 shadow-md"
                      title="Clear / クリア"
                    >
                      <Trash2 size={14} />
                    </button>
                  </>
                ) : (
                  <div className="w-full h-full flex flex-col items-center justify-center text-gray-600 gap-2">
                    <Image size={32} strokeWidth={1.5} />
                    <span className="text-xs">No image selected</span>
                  </div>
                )}
              </div>

              {/* Capture Button / キャプチャボタン */}
              <button
                onClick={handleCapture}
                disabled={isCapturing}
                className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-sikuli-600 hover:bg-sikuli-500 disabled:bg-gray-700 disabled:cursor-not-allowed text-white rounded-lg transition-all duration-200 shadow-lg shadow-sikuli-600/20"
              >
                <Camera size={16} />
                <span className="text-xs font-bold uppercase tracking-wide">
                  {isCapturing ? 'Capturing...' : 'Capture Screen'}
                </span>
              </button>
            </div>
          </div>
        )}

        {/* Similarity / 一致率 */}
        {selectedLine.similarity !== undefined && (
          <div className="bg-dark-surface p-3 rounded-lg border border-dark-border/50">
            <div className="flex items-center justify-between mb-2">
              <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider">
                Similarity
              </label>
              <span className="text-xs font-mono text-sikuli-400 bg-sikuli-400/10 px-1.5 py-0.5 rounded">
                {Math.round(selectedLine.similarity * 100)}%
              </span>
            </div>
            <input
              type="range"
              value={selectedLine.similarity}
              onChange={(e) =>
                onUpdateLine(selectedLine.id, { similarity: parseFloat(e.target.value) })
              }
              min="0"
              max="1"
              step="0.05"
              className="w-full accent-sikuli-500 h-1.5 bg-dark-bg rounded-lg appearance-none cursor-pointer"
            />
            <div className="flex justify-between text-[10px] text-gray-600 mt-1 font-mono">
              <span>Loose (0%)</span>
              <span>Exact (100%)</span>
            </div>
          </div>
        )}

        {/* Text Input for Type command / Type コマンドのテキスト入力 */}
        {selectedLine.type === 'type' && (
          <div>
            <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-2">
              Text Content
            </label>
            <textarea
              value={selectedLine.params || ''}
              onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
              placeholder="Enter text to type..."
              rows={4}
              className="w-full px-3 py-2.5 bg-dark-bg border border-dark-border rounded-lg text-sm text-gray-200 placeholder-gray-600 focus:outline-none focus:border-sikuli-500 focus:ring-1 focus:ring-sikuli-500 transition-colors resize-none font-mono"
            />
          </div>
        )}

        {/* Wait Time / 待機時間 */}
        {selectedLine.type === 'wait' && (
          <div>
            <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-2">
              Duration
            </label>
            <div className="flex items-center gap-3 bg-dark-bg border border-dark-border rounded-lg px-3 py-2 focus-within:border-sikuli-500 focus-within:ring-1 focus-within:ring-sikuli-500 transition-colors">
              <Clock size={16} className="text-gray-500" />
              <input
                type="number"
                value={selectedLine.params || '1'}
                onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
                min="0"
                step="0.1"
                className="flex-1 bg-transparent border-none focus:ring-0 text-sm font-mono text-gray-200 p-0"
              />
              <span className="text-xs text-gray-500 font-medium">seconds</span>
            </div>
          </div>
        )}

        {/* Loop Count / ループ回数 */}
        {selectedLine.type === 'loop' && (
          <div>
            <label className="block text-[10px] font-bold text-gray-500 uppercase tracking-wider mb-2">
              Iterations
            </label>
            <div className="flex items-center gap-3 bg-dark-bg border border-dark-border rounded-lg px-3 py-2 focus-within:border-sikuli-500 focus-within:ring-1 focus-within:ring-sikuli-500 transition-colors">
              <Repeat size={16} className="text-gray-500" />
              <input
                type="number"
                value={selectedLine.params || '1'}
                onChange={(e) => onUpdateLine(selectedLine.id, { params: e.target.value })}
                min="1"
                className="flex-1 bg-transparent border-none focus:ring-0 text-sm font-mono text-gray-200 p-0"
              />
              <span className="text-xs text-gray-500 font-medium">times</span>
            </div>
          </div>
        )}
      </div>
    </aside>
  )
}
