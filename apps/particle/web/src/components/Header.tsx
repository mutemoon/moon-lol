import React from 'react'
import {
  Play,
  Square,
  RotateCcw,
  Camera,
  Code2,
  Sparkles,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Badge } from '@/components/ui/badge'
import { Switch } from '@/components/ui/switch'
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from '@/components/ui/tooltip'
import type { WebParticleSystem } from '../types/particle'

interface HeaderProps {
  selectedHero: string | null
  selectedSystem: WebParticleSystem | null
  autoReplay: boolean
  setAutoReplay: (v: boolean) => void
  onPlay: () => void
  onStop: () => void
  onReplay: () => void
  onResetCamera: () => void
  onOpenRonModal: () => void
  isWasmReady: boolean
}

export const Header: React.FC<HeaderProps> = ({
  selectedHero,
  selectedSystem,
  autoReplay,
  setAutoReplay,
  onPlay,
  onStop,
  onReplay,
  onResetCamera,
  onOpenRonModal,
  isWasmReady,
}) => {
  return (
    <header className="h-14 border-b border-border/40 bg-background/80 backdrop-blur-md px-4 flex items-center justify-between select-none shrink-0 z-30">
      {/* 品牌与当前路径 */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-purple-600 via-indigo-600 to-pink-500 flex items-center justify-center shadow-lg shadow-purple-500/20">
            <Sparkles className="w-4 h-4 text-white" />
          </div>
          <div className="flex flex-col">
            <div className="flex items-center gap-1.5">
              <span className="font-bold text-sm tracking-wider text-foreground">MOON</span>
              <span className="text-[10px] font-semibold text-purple-400 bg-purple-950/60 px-1.5 py-0.2 rounded border border-purple-500/30">VFX</span>
            </div>
            <span className="text-[10px] text-muted-foreground/80 tracking-tight">WebGPU Particle Studio</span>
          </div>
        </div>

        <div className="h-4 w-px bg-border/60 mx-1" />

        {/* 当前选中特效信息 */}
        {selectedSystem ? (
          <div className="flex items-center gap-2 text-xs">
            <Badge variant="outline" className="bg-muted/40 font-medium text-foreground capitalize">
              {selectedHero}
            </Badge>
            <span className="text-muted-foreground">/</span>
            <span className="font-semibold text-foreground max-w-[200px] truncate" title={selectedSystem.name}>
              {selectedSystem.name}
            </span>
            <Badge variant="secondary" className="font-mono text-[10px] tracking-wider text-purple-300 bg-purple-950/40 border border-purple-500/20">
              {selectedSystem.hash_hex}
            </Badge>
            <Badge variant="outline" className="text-[10px] text-muted-foreground">
              {selectedSystem.emitter_count} 发射器
            </Badge>
          </div>
        ) : (
          <span className="text-xs text-muted-foreground">未选择粒子系统</span>
        )}
      </div>

      {/* 播放控制与工具栏 */}
      <div className="flex items-center gap-2.5">
        <TooltipProvider delayDuration={200}>
          {/* 自动重播 Switch */}
          <div className="flex items-center gap-2 mr-1">
            <Switch
              id="auto-replay-switch"
              checked={autoReplay}
              onCheckedChange={setAutoReplay}
              className="data-[state=checked]:bg-purple-600"
            />
            <label htmlFor="auto-replay-switch" className="text-xs text-muted-foreground cursor-pointer select-none">
              改动后自动重播
            </label>
          </div>

          <div className="h-4 w-px bg-border/60 mx-1" />

          {/* 播放 / 停止 / 重播 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="default"
                onClick={onPlay}
                className="bg-emerald-600 hover:bg-emerald-500 text-white shadow-md shadow-emerald-900/20 h-8 px-3 gap-1.5"
              >
                <Play className="w-3.5 h-3.5 fill-current" />
                <span>播放</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>播放当前完整特效</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="outline"
                onClick={onReplay}
                className="h-8 px-3 gap-1.5 border-border/60 hover:bg-accent hover:text-accent-foreground"
              >
                <RotateCcw className="w-3.5 h-3.5" />
                <span>重播</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>重置时间并从头播放</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="outline"
                onClick={onStop}
                className="h-8 px-3 gap-1.5 text-rose-400 border-rose-500/20 hover:bg-rose-950/20 hover:text-rose-300"
              >
                <Square className="w-3.5 h-3.5 fill-current" />
                <span>停止</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>停止所有粒子发射</TooltipContent>
          </Tooltip>

          <div className="h-4 w-px bg-border/60 mx-1" />

          {/* 视角居中 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="ghost"
                onClick={onResetCamera}
                className="h-8 w-8 p-0 text-muted-foreground hover:text-foreground"
              >
                <Camera className="w-4 h-4" />
              </Button>
            </TooltipTrigger>
            <TooltipContent>重置相机聚焦视角</TooltipContent>
          </Tooltip>

          {/* RON 源码 */}
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="outline"
                onClick={onOpenRonModal}
                className="h-8 px-2.5 gap-1.5 text-xs font-mono border-purple-500/30 text-purple-300 bg-purple-950/30 hover:bg-purple-900/40"
              >
                <Code2 className="w-3.5 h-3.5" />
                <span>RON</span>
              </Button>
            </TooltipTrigger>
            <TooltipContent>查看/编辑当前粒子 RON 源码</TooltipContent>
          </Tooltip>

          {/* WebGPU 状态 */}
          <div className="flex items-center gap-1.5 pl-2 text-[11px] text-muted-foreground">
            <span className={`w-2 h-2 rounded-full ${isWasmReady ? 'bg-emerald-400 animate-pulse' : 'bg-amber-400'}`} />
            <span className="font-mono text-[10px]">{isWasmReady ? 'WEBGPU' : 'INIT'}</span>
          </div>
        </TooltipProvider>
      </div>
    </header>
  )
}
