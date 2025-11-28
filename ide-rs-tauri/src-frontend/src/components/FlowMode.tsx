import { useState, useRef, useCallback, useEffect } from 'react'
import {
  MousePointer2,
  Type,
  Clock,
  Search,
  GitBranch,
  Repeat,
  Play,
  ZoomIn,
  ZoomOut,
  Maximize2,
  type LucideIcon,
} from 'lucide-react'
import type { ScriptLine, CommandType } from '../types/script'

interface FlowModeProps {
  script: ScriptLine[]
  selectedLineId: string | null
  onSelectLine: (id: string | null) => void
  onUpdateLine: (id: string, updates: Partial<ScriptLine>) => void
  onDeleteLine: (id: string) => void
}

/**
 * Command type icon map
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
 * Command type colors
 */
const commandColors: Record<CommandType, string> = {
  start: 'border-green-500',
  click: 'border-blue-500',
  type: 'border-purple-500',
  wait: 'border-yellow-500',
  find: 'border-cyan-500',
  if: 'border-orange-500',
  loop: 'border-pink-500',
}

/**
 * Flow Mode Component - Node graph view for script editing
 * フローモードコンポーネント - スクリプト編集のノードグラフビュー
 */
export function FlowMode({
  script,
  selectedLineId,
  onSelectLine,
  onUpdateLine,
}: FlowModeProps) {
  // Canvas state / キャンバス状態
  const [pan, setPan] = useState({ x: 0, y: 0 })
  const [zoom, setZoom] = useState(1)
  const [isPanning, setIsPanning] = useState(false)
  const [panStart, setPanStart] = useState({ x: 0, y: 0 })

  // Dragging node state / ドラッグ中のノード状態
  const [draggingNode, setDraggingNode] = useState<string | null>(null)
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 })

  // Canvas ref / キャンバス参照
  const canvasRef = useRef<HTMLDivElement>(null)
  const svgRef = useRef<SVGSVGElement>(null)

  /**
   * Handle mouse down for panning
   * パン用のマウスダウンを処理
   */
  const handleCanvasMouseDown = (e: React.MouseEvent) => {
    if (e.target === canvasRef.current || e.target === svgRef.current) {
      setIsPanning(true)
      setPanStart({ x: e.clientX - pan.x, y: e.clientY - pan.y })
      onSelectLine(null)
    }
  }

  /**
   * Handle mouse move for panning
   * パン用のマウス移動を処理
   */
  const handleMouseMove = useCallback(
    (e: React.MouseEvent) => {
      if (isPanning) {
        setPan({ x: e.clientX - panStart.x, y: e.clientY - panStart.y })
      }
      if (draggingNode) {
        const rect = canvasRef.current?.getBoundingClientRect()
        if (rect) {
          const x = (e.clientX - rect.left - pan.x) / zoom - dragOffset.x
          const y = (e.clientY - rect.top - pan.y) / zoom - dragOffset.y
          onUpdateLine(draggingNode, { flowConfig: { x, y } })
        }
      }
    },
    [isPanning, panStart, draggingNode, dragOffset, pan, zoom, onUpdateLine]
  )

  /**
   * Handle mouse up
   * マウスアップを処理
   */
  const handleMouseUp = () => {
    setIsPanning(false)
    setDraggingNode(null)
  }

  /**
   * Handle wheel for zooming
   * ズーム用のホイールを処理
   */
  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault()
    const delta = e.deltaY > 0 ? 0.9 : 1.1
    setZoom((prev) => Math.min(Math.max(prev * delta, 0.25), 2))
  }, [])

  // Add wheel event listener
  useEffect(() => {
    const canvas = canvasRef.current
    if (canvas) {
      canvas.addEventListener('wheel', handleWheel, { passive: false })
      return () => canvas.removeEventListener('wheel', handleWheel)
    }
  }, [handleWheel])

  /**
   * Handle node mouse down for dragging
   * ドラッグ用のノードマウスダウンを処理
   */
  const handleNodeMouseDown = (e: React.MouseEvent, node: ScriptLine) => {
    e.stopPropagation()
    const rect = canvasRef.current?.getBoundingClientRect()
    if (rect && node.flowConfig) {
      setDraggingNode(node.id)
      setDragOffset({
        x: (e.clientX - rect.left - pan.x) / zoom - node.flowConfig.x,
        y: (e.clientY - rect.top - pan.y) / zoom - node.flowConfig.y,
      })
      onSelectLine(node.id)
    }
  }

  /**
   * Zoom controls / ズームコントロール
   */
  const handleZoomIn = () => setZoom((prev) => Math.min(prev * 1.2, 2))
  const handleZoomOut = () => setZoom((prev) => Math.max(prev * 0.8, 0.25))
  const handleResetView = () => {
    setZoom(1)
    setPan({ x: 0, y: 0 })
  }

  /**
   * Calculate bezier curve path between two nodes
   * 2つのノード間のベジェ曲線パスを計算
   */
  const getBezierPath = (from: { x: number; y: number }, to: { x: number; y: number }) => {
    const fromX = from.x + 75 // Center of node (150/2)
    const fromY = from.y + 60 // Bottom of node
    const toX = to.x + 75
    const toY = to.y

    const midY = (fromY + toY) / 2
    return `M ${fromX} ${fromY} C ${fromX} ${midY}, ${toX} ${midY}, ${toX} ${toY}`
  }

  /**
   * Render connections between nodes
   * ノード間の接続を描画
   */
  const renderConnections = () => {
    const paths: JSX.Element[] = []

    for (let i = 0; i < script.length - 1; i++) {
      const from = script[i]
      const to = script[i + 1]
      if (from.flowConfig && to.flowConfig) {
        paths.push(
          <path
            key={`${from.id}-${to.id}`}
            d={getBezierPath(from.flowConfig, to.flowConfig)}
            fill="none"
            stroke="#6b7280"
            strokeWidth={2}
            className="pointer-events-none"
          />
        )
      }
    }

    return paths
  }

  /**
   * Render a single node
   * 単一のノードを描画
   */
  const renderNode = (node: ScriptLine) => {
    if (!node.flowConfig) return null

    const IconComponent = commandIcons[node.type]
    const colorClass = commandColors[node.type]
    const isSelected = node.id === selectedLineId

    return (
      <div
        key={node.id}
        className={`flow-node absolute cursor-move border-t-2 ${colorClass} ${
          isSelected ? 'ring-2 ring-sikuli-500' : ''
        } ${draggingNode === node.id ? 'opacity-75' : ''}`}
        style={{
          left: node.flowConfig.x,
          top: node.flowConfig.y,
          transform: `scale(${1 / zoom})`,
          transformOrigin: 'top left',
        }}
        onMouseDown={(e) => handleNodeMouseDown(e, node)}
      >
        {/* Node Header / ノードヘッダー */}
        <div className="flow-node-header">
          <IconComponent size={16} className="text-sikuli-400" />
          <span className="text-sm font-medium capitalize">{node.type}</span>
        </div>

        {/* Node Body / ノードボディ */}
        <div className="flow-node-body text-xs space-y-2">
          {/* Image Target / 画像ターゲット */}
          {(node.type === 'click' || node.type === 'find' || node.type === 'if') && (
            <div className="w-full h-12 bg-dark-bg rounded border border-dark-border flex items-center justify-center">
              {node.target ? (
                <img
                  src={node.target}
                  alt="target"
                  className="max-w-full max-h-full object-contain"
                />
              ) : (
                <span className="text-gray-500">Click to capture</span>
              )}
            </div>
          )}

          {/* Text Input / テキスト入力 */}
          {node.type === 'type' && (
            <input
              type="text"
              value={node.params || ''}
              onChange={(e) => onUpdateLine(node.id, { params: e.target.value })}
              onMouseDown={(e) => e.stopPropagation()}
              placeholder="Text..."
              className="w-full px-2 py-1 bg-dark-bg border border-dark-border rounded text-xs"
            />
          )}

          {/* Wait Time / 待機時間 */}
          {node.type === 'wait' && (
            <div className="flex items-center gap-1">
              <input
                type="number"
                value={node.params || '1'}
                onChange={(e) => onUpdateLine(node.id, { params: e.target.value })}
                onMouseDown={(e) => e.stopPropagation()}
                min="0"
                step="0.1"
                className="w-16 px-2 py-1 bg-dark-bg border border-dark-border rounded text-xs"
              />
              <span className="text-gray-500">sec</span>
            </div>
          )}

          {/* Similarity / 一致率 */}
          {node.similarity !== undefined && (
            <div className="flex items-center justify-between">
              <span className="text-gray-500">Match:</span>
              <span>{Math.round(node.similarity * 100)}%</span>
            </div>
          )}
        </div>

        {/* Output Port / 出力ポート */}
        <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 flow-port" />

        {/* Input Port / 入力ポート */}
        {node.type !== 'start' && (
          <div className="absolute -top-1.5 left-1/2 -translate-x-1/2 flow-port" />
        )}

        {/* If Node: True/False Ports / Ifノード: True/Falseポート */}
        {node.type === 'if' && (
          <>
            <div className="absolute -bottom-1.5 left-1/4 -translate-x-1/2">
              <div className="flow-port bg-green-500" title="True" />
              <span className="text-xs text-green-500 mt-1 block text-center">T</span>
            </div>
            <div className="absolute -bottom-1.5 left-3/4 -translate-x-1/2">
              <div className="flow-port bg-red-500" title="False" />
              <span className="text-xs text-red-500 mt-1 block text-center">F</span>
            </div>
          </>
        )}
      </div>
    )
  }

  return (
    <div className="h-full relative overflow-hidden bg-dark-bg">
      {/* Canvas / キャンバス */}
      <div
        ref={canvasRef}
        className="flow-canvas absolute inset-0 cursor-grab"
        style={{
          cursor: isPanning ? 'grabbing' : draggingNode ? 'move' : 'grab',
        }}
        onMouseDown={handleCanvasMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {/* Transform Container / 変換コンテナ */}
        <div
          className="absolute"
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transformOrigin: '0 0',
          }}
        >
          {/* SVG for Connections / 接続用SVG */}
          <svg
            ref={svgRef}
            className="absolute inset-0 pointer-events-none"
            style={{ width: '5000px', height: '5000px' }}
          >
            {renderConnections()}
          </svg>

          {/* Nodes / ノード */}
          {script.map(renderNode)}
        </div>
      </div>

      {/* Zoom Controls / ズームコントロール */}
      <div className="absolute bottom-4 right-4 flex items-center gap-2 bg-dark-surface border border-dark-border rounded-lg p-1">
        <button
          onClick={handleZoomOut}
          className="p-2 hover:bg-dark-hover rounded"
          title="Zoom Out / ズームアウト"
        >
          <ZoomOut size={16} />
        </button>
        <span className="text-xs text-gray-400 min-w-12 text-center">
          {Math.round(zoom * 100)}%
        </span>
        <button
          onClick={handleZoomIn}
          className="p-2 hover:bg-dark-hover rounded"
          title="Zoom In / ズームイン"
        >
          <ZoomIn size={16} />
        </button>
        <button
          onClick={handleResetView}
          className="p-2 hover:bg-dark-hover rounded"
          title="Reset View / ビューをリセット"
        >
          <Maximize2 size={16} />
        </button>
      </div>

      {/* Info Overlay / 情報オーバーレイ */}
      <div className="absolute top-4 left-4 text-xs text-gray-500">
        Scroll to zoom • Drag canvas to pan • Drag nodes to move
      </div>
    </div>
  )
}
