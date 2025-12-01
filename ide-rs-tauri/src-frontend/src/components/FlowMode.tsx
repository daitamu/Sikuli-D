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
        className={`flow-node absolute cursor-move group ${
          isSelected ? 'ring-2 ring-sikuli-500 ring-offset-2 ring-offset-dark-bg' : ''
        } ${draggingNode === node.id ? 'opacity-75 scale-105 shadow-2xl' : ''}`}
        style={{
          left: node.flowConfig.x,
          top: node.flowConfig.y,
          transform: `scale(${1 / zoom})`,
          transformOrigin: 'top left',
        }}
        onMouseDown={(e) => handleNodeMouseDown(e, node)}
      >
        {/* Node Header / ノードヘッダー */}
        <div className={`flow-node-header ${colorClass.replace('border-', 'border-b-')} border-b-2 bg-dark-bg/50`}>
          <div className={`p-1 rounded-md ${colorClass.replace('border-', 'bg-').replace('500', '500/20')} ${colorClass.replace('border-', 'text-').replace('500', '400')}`}>
            <IconComponent size={14} strokeWidth={2.5} />
          </div>
          <span className="text-xs font-bold text-gray-200 capitalize tracking-wide">{node.type}</span>
        </div>

        {/* Node Body / ノードボディ */}
        <div className="flow-node-body text-xs space-y-3">
          {/* Image Target / 画像ターゲット */}
          {(node.type === 'click' || node.type === 'find' || node.type === 'if') && (
            <div className="w-full h-16 bg-dark-bg rounded-lg border border-dark-border/50 flex items-center justify-center overflow-hidden relative group-hover:border-gray-600 transition-colors">
              {node.target ? (
                <>
                  <div className="absolute inset-0 bg-[url('/transparent-grid.png')] opacity-10"></div>
                  <img
                    src={node.target}
                    alt="target"
                    className="max-w-full max-h-full object-contain relative z-10"
                  />
                </>
              ) : (
                <div className="flex flex-col items-center gap-1 text-gray-600">
                   <MousePointer2 size={16} />
                   <span className="text-[10px]">No target</span>
                </div>
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
              className="w-full px-2 py-1.5 bg-dark-bg border border-dark-border rounded text-xs focus:border-sikuli-500 focus:ring-1 focus:ring-sikuli-500 transition-all"
            />
          )}

          {/* Wait Time / 待機時間 */}
          {node.type === 'wait' && (
            <div className="flex items-center gap-2 bg-dark-bg border border-dark-border rounded px-2 py-1.5">
              <Clock size={12} className="text-gray-500" />
              <input
                type="number"
                value={node.params || '1'}
                onChange={(e) => onUpdateLine(node.id, { params: e.target.value })}
                onMouseDown={(e) => e.stopPropagation()}
                min="0"
                step="0.1"
                className="w-full bg-transparent border-none outline-none text-xs font-mono"
              />
              <span className="text-gray-500 text-[10px]">s</span>
            </div>
          )}

          {/* Similarity / 一致率 */}
          {node.similarity !== undefined && (
            <div className="flex items-center justify-between bg-dark-bg/50 rounded px-2 py-1">
              <span className="text-[10px] text-gray-500 uppercase font-bold">Match</span>
              <span className="font-mono text-sikuli-400">{Math.round(node.similarity * 100)}%</span>
            </div>
          )}
        </div>

        {/* Output Port / 出力ポート */}
        <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 flow-port shadow-sm hover:scale-125 transition-transform" />

        {/* Input Port / 入力ポート */}
        {node.type !== 'start' && (
          <div className="absolute -top-1.5 left-1/2 -translate-x-1/2 flow-port shadow-sm hover:scale-125 transition-transform" />
        )}

        {/* If Node: True/False Ports / Ifノード: True/Falseポート */}
        {node.type === 'if' && (
          <>
            <div className="absolute -bottom-1.5 left-1/4 -translate-x-1/2 flex flex-col items-center">
              <div className="flow-port bg-green-500 border-green-900 shadow-sm" title="True" />
              <span className="absolute top-full text-[10px] font-bold text-green-500 mt-0.5">T</span>
            </div>
            <div className="absolute -bottom-1.5 left-3/4 -translate-x-1/2 flex flex-col items-center">
              <div className="flow-port bg-red-500 border-red-900 shadow-sm" title="False" />
              <span className="absolute top-full text-[10px] font-bold text-red-500 mt-0.5">F</span>
            </div>
          </>
        )}
      </div>
    )
  }

  return (
    <div className="h-full relative overflow-hidden bg-dark-bg group/canvas">
      {/* Canvas / キャンバス */}
      <div
        ref={canvasRef}
        className="flow-canvas absolute inset-0"
        style={{
          cursor: isPanning ? 'grabbing' : draggingNode ? 'grabbing' : 'grab',
        }}
        onMouseDown={handleCanvasMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {/* Transform Container / 変換コンテナ */}
        <div
          className="absolute will-change-transform"
          style={{
            transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})`,
            transformOrigin: '0 0',
          }}
        >
          {/* SVG for Connections / 接続用SVG */}
          <svg
            ref={svgRef}
            className="absolute inset-0 pointer-events-none overflow-visible"
            style={{ width: '5000px', height: '5000px' }}
          >
            <defs>
               <filter id="glow">
                  <feGaussianBlur stdDeviation="2.5" result="coloredBlur"/>
                  <feMerge>
                     <feMergeNode in="coloredBlur"/>
                     <feMergeNode in="SourceGraphic"/>
                  </feMerge>
               </filter>
            </defs>
            {renderConnections()}
          </svg>

          {/* Nodes / ノード */}
          {script.map(renderNode)}
        </div>
      </div>

      {/* Zoom Controls / ズームコントロール */}
      <div className="absolute bottom-6 right-6 flex items-center gap-1 bg-dark-surface/90 backdrop-blur border border-dark-border rounded-lg p-1 shadow-lg">
        <button
          onClick={handleZoomOut}
          className="p-1.5 hover:bg-dark-hover text-gray-400 hover:text-gray-200 rounded transition-colors"
          title="Zoom Out"
        >
          <ZoomOut size={16} />
        </button>
        <span className="text-xs font-mono text-gray-400 min-w-[3rem] text-center select-none">
          {Math.round(zoom * 100)}%
        </span>
        <button
          onClick={handleZoomIn}
          className="p-1.5 hover:bg-dark-hover text-gray-400 hover:text-gray-200 rounded transition-colors"
          title="Zoom In"
        >
          <ZoomIn size={16} />
        </button>
        <div className="w-px h-4 bg-dark-border mx-1"></div>
        <button
          onClick={handleResetView}
          className="p-1.5 hover:bg-dark-hover text-gray-400 hover:text-gray-200 rounded transition-colors"
          title="Reset View"
        >
          <Maximize2 size={16} />
        </button>
      </div>

      {/* Info Overlay / 情報オーバーレイ */}
      <div className="absolute top-4 left-4 px-3 py-1.5 bg-dark-surface/50 backdrop-blur rounded-full border border-dark-border/30 text-[10px] text-gray-500 select-none pointer-events-none opacity-50 group-hover/canvas:opacity-100 transition-opacity">
        Scroll to zoom • Drag canvas to pan • Drag nodes to move
      </div>
    </div>
  )
}
