import React, { useRef, useState, useEffect } from 'react'
import { Maximize2, Minimize2, RotateCcw } from 'lucide-react'
import { Button } from '@/components/ui/button'


interface ViewportProps {
  iframeRef: React.RefObject<HTMLIFrameElement | null>
  isWasmReady: boolean
  onResetCamera: () => void
  currentParticleName?: string
}

export const Viewport: React.FC<ViewportProps> = ({
  iframeRef,
  isWasmReady,
  onResetCamera,
  currentParticleName,
}) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)

  const toggleFullscreen = () => {
    if (!containerRef.current) return
    if (!document.fullscreenElement) {
      containerRef.current.requestFullscreen().then(() => setIsFullscreen(true)).catch(() => {})
    } else {
      document.exitFullscreen().then(() => setIsFullscreen(false)).catch(() => {})
    }
  }

  useEffect(() => {
    const handler = () => {
      setIsFullscreen(!!document.fullscreenElement)
    }
    document.addEventListener('fullscreenchange', handler)
    return () => document.removeEventListener('fullscreenchange', handler)
  }, [])

  return (
    <div
      ref={containerRef}
      className="relative flex-1 h-full bg-[#0d0f18] overflow-hidden flex flex-col select-none"
    >
      {/* Bevy WASM 渲染视口 iframe */}
      <iframe
        ref={iframeRef}
        src="/wasm/index.html"
        title="Bevy WASM Viewport"
        className="w-full h-full border-0 block"
        allow="cross-origin-isolated"
      />

      {/* 视口浮动左上角：粒子信息与运行状态 */}
      <div className="absolute top-3 left-3 flex items-center gap-2 pointer-events-none z-10">
        <div className="px-2.5 py-1 rounded-md bg-black/60 backdrop-blur-md border border-white/10 flex items-center gap-2 shadow-lg">
          <div className="flex items-center gap-1.5">
            <span
              className={`w-2 h-2 rounded-full ${
                isWasmReady ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-amber-400'
              }`}
            />
            <span className="text-[11px] font-mono text-white/90 uppercase tracking-wider font-semibold">
              {isWasmReady ? 'BEVY WEBGPU' : 'LOADING WASM...'}
            </span>
          </div>

          {currentParticleName && (
            <>
              <div className="w-px h-3 bg-white/20" />
              <span className="text-[11px] font-medium text-purple-300 truncate max-w-[240px]">
                {currentParticleName}
              </span>
            </>
          )}
        </div>
      </div>

      {/* 视口浮动右上角快捷按键 */}
      <div className="absolute top-3 right-3 flex items-center gap-1.5 z-10">
        <Button
          size="sm"
          variant="outline"
          onClick={onResetCamera}
          className="h-8 px-2.5 gap-1.5 bg-black/60 hover:bg-black/80 backdrop-blur-md border-white/10 text-white/80 hover:text-white text-xs shadow-lg"
        >
          <RotateCcw className="w-3.5 h-3.5" />
          <span>居中</span>
        </Button>

        <Button
          size="sm"
          variant="outline"
          onClick={toggleFullscreen}
          className="h-8 w-8 p-0 bg-black/60 hover:bg-black/80 backdrop-blur-md border-white/10 text-white/80 hover:text-white shadow-lg"
        >
          {isFullscreen ? <Minimize2 className="w-3.5 h-3.5" /> : <Maximize2 className="w-3.5 h-3.5" />}
        </Button>
      </div>

      {/* 视口底部操作提示 */}
      <div className="absolute bottom-3 left-1/2 -translate-x-1/2 pointer-events-none z-10">
        <div className="px-3 py-1 rounded-full bg-black/50 backdrop-blur-md border border-white/10 text-[10px] text-white/60 tracking-wider flex items-center gap-2">
          <span>右键旋转视口</span>
          <span>•</span>
          <span>滚轮缩放</span>
          <span>•</span>
          <span>中键拖拽平移</span>
        </div>
      </div>
    </div>
  )
}
