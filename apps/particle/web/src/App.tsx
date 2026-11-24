import { useState, useEffect, useRef, useCallback } from 'react'
import { Header } from './components/Header'
import { Sidebar } from './components/Sidebar'
import { Viewport } from './components/Viewport'
import { EmitterInspector } from './components/EmitterInspector'
import { RonDialog } from './components/RonDialog'
import type {
  WebHeroSummary,
  WebHeroDetail,
  WebParticleSystem,
  WebSystemSummaryItem,
  ConfigVfxEmitterDefinition,
  ConfigVfxSystemDefinition,
} from './types/particle'
import {
  playVfxRon,
  playSingleEmitter,
  stopVfx,
  resetCamera,
} from './lib/vfx-bridge'

export function App() {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const [heroes, setHeroes] = useState<WebHeroSummary[]>([])
  const [selectedHero, setSelectedHero] = useState<string | null>(null)
  const [selectedHash, setSelectedHash] = useState<number | null>(null)
  const [selectedSystem, setSelectedSystem] = useState<WebParticleSystem | null>(null)
  const [workingDef, setWorkingDef] = useState<ConfigVfxSystemDefinition | null>(null)
  const [activeEmitterIndex, setActiveEmitterIndex] = useState<number>(0)

  const [isWasmReady, setIsWasmReady] = useState(false)
  const [autoReplay, setAutoReplay] = useState(true)
  const [isRonModalOpen, setIsRonModalOpen] = useState(false)
  const [isLoadingDetails, setIsLoadingDetails] = useState(false)

  // 英雄详情数据缓存 (hero_id -> WebHeroDetail)
  const heroCache = useRef<Map<string, WebHeroDetail>>(new Map())

  // 1. 首屏加载英雄目录
  useEffect(() => {
    fetch('/data/heroes_catalog.json')
      .then((res) => res.json())
      .then((data: WebHeroSummary[]) => {
        setHeroes(data)
        // 默认选中 Riven
        const riven = data.find((h) => h.id === 'riven') || data[0]
        if (riven && riven.systems.length > 0) {
          handleSelectSystem(riven.id, riven.systems[0])
        }
      })
      .catch((err) => console.error('加载 heroes_catalog.json 失败:', err))
  }, [])

  // 2. 监听来自 Bevy WASM iframe 的就绪握手
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      try {
        const data = typeof event.data === 'string' ? JSON.parse(event.data) : event.data
        if (data?.type === 'BEVY_READY') {
          console.log('[Bevy WASM] WebGPU 渲染器初始化就绪！')
          setIsWasmReady(true)
        }
      } catch {}
    }

    window.addEventListener('message', handleMessage)
    // 轮询检查一次 iframe window
    const timer = setTimeout(() => {
      if (iframeRef.current?.contentWindow) {
        setIsWasmReady(true)
      }
    }, 1500)

    return () => {
      window.removeEventListener('message', handleMessage)
      clearTimeout(timer)
    }
  }, [])

  // 3. 选择某个粒子系统
  const handleSelectSystem = async (heroId: string, item: WebSystemSummaryItem) => {
    setSelectedHero(heroId)
    setSelectedHash(item.hash)
    setActiveEmitterIndex(0)

    let detail = heroCache.current.get(heroId)
    if (!detail) {
      setIsLoadingDetails(true)
      try {
        const res = await fetch(`/data/heroes/${heroId.toLowerCase()}.json`)
        detail = await res.json()
        if (detail) {
          heroCache.current.set(heroId, detail)
        }
      } catch (e) {
        console.error(`加载英雄详情 ${heroId} 失败:`, e)
      } finally {
        setIsLoadingDetails(false)
      }
    }

    if (detail) {
      const found = detail.systems.find((s) => s.hash === item.hash)
      if (found) {
        setSelectedSystem(found)
        // 深拷贝作为工作副本
        const clonedDef = JSON.parse(JSON.stringify(found.def))
        setWorkingDef(clonedDef)

        // 立即向 WASM 发送播放指令
        playVfxRon(iframeRef.current?.contentWindow || null, found.ron_str)
      }
    }
  }

  // 4. 播放 / 停止 / 重播
  const handlePlay = useCallback(() => {
    if (!selectedSystem) return
    playVfxRon(iframeRef.current?.contentWindow || null, selectedSystem.ron_str)
  }, [selectedSystem])

  const handleStop = useCallback(() => {
    stopVfx(iframeRef.current?.contentWindow || null)
  }, [])

  const handleReplay = useCallback(() => {
    handlePlay()
  }, [handlePlay])

  const handleResetCamera = useCallback(() => {
    resetCamera(iframeRef.current?.contentWindow || null)
  }, [])

  // 5. 单发射器播放
  const handlePlaySingleEmitter = (emitterIndex: number) => {
    if (!selectedSystem) return
    playSingleEmitter(iframeRef.current?.contentWindow || null, selectedSystem.ron_str, emitterIndex)
  }

  // 6. 重置单发射器
  const handleResetEmitter = (emitterIndex: number) => {
    if (!selectedSystem) return
    const origEmitters =
      selectedSystem.def.complex_emitter_definition_data ||
      selectedSystem.def.simple_emitter_definition_data ||
      []
    if (origEmitters[emitterIndex]) {
      handleUpdateEmitter(emitterIndex, JSON.parse(JSON.stringify(origEmitters[emitterIndex])))
    }
  }

  // 7. 更新发射器参数
  const handleUpdateEmitter = (emitterIndex: number, updated: ConfigVfxEmitterDefinition) => {
    if (!workingDef) return

    const newDef = { ...workingDef }
    if (newDef.complex_emitter_definition_data) {
      const list = [...newDef.complex_emitter_definition_data]
      list[emitterIndex] = updated
      newDef.complex_emitter_definition_data = list
    } else if (newDef.simple_emitter_definition_data) {
      const list = [...newDef.simple_emitter_definition_data]
      list[emitterIndex] = updated
      newDef.simple_emitter_definition_data = list
    }

    setWorkingDef(newDef)

    // 如果开启自动重播
    if (autoReplay && selectedSystem) {
      playVfxRon(iframeRef.current?.contentWindow || null, selectedSystem.ron_str)
    }
  }

  // 8. 应用 RON 文本
  const handleApplyRon = (newRon: string) => {
    if (selectedSystem) {
      setSelectedSystem({
        ...selectedSystem,
        ron_str: newRon,
      })
    }
    playVfxRon(iframeRef.current?.contentWindow || null, newRon)
  }

  return (
    <div className="flex flex-col w-screen h-screen overflow-hidden bg-background text-foreground antialiased selection:bg-purple-500/20">
      {/* 顶部控制栏 */}
      <Header
        selectedHero={selectedHero}
        selectedSystem={selectedSystem}
        autoReplay={autoReplay}
        setAutoReplay={setAutoReplay}
        onPlay={handlePlay}
        onStop={handleStop}
        onReplay={handleReplay}
        onResetCamera={handleResetCamera}
        onOpenRonModal={() => setIsRonModalOpen(true)}
        isWasmReady={isWasmReady}
      />

      {/* 主工作区：左侧资产树 + 中央 3D 视口 + 右侧发射器检视面板 */}
      <div className="flex-1 flex overflow-hidden">
        <Sidebar
          heroes={heroes}
          selectedHero={selectedHero}
          selectedHash={selectedHash}
          onSelectSystem={handleSelectSystem}
          isLoadingDetails={isLoadingDetails}
        />

        <Viewport
          iframeRef={iframeRef}
          isWasmReady={isWasmReady}
          onResetCamera={handleResetCamera}
          currentParticleName={selectedSystem?.name}
        />

        <EmitterInspector
          systemDef={workingDef}
          activeEmitterIndex={activeEmitterIndex}
          setActiveEmitterIndex={setActiveEmitterIndex}
          onUpdateEmitter={handleUpdateEmitter}
          onPlaySingleEmitter={handlePlaySingleEmitter}
          onResetEmitter={handleResetEmitter}
        />
      </div>

      {/* RON 源码弹窗 */}
      <RonDialog
        isOpen={isRonModalOpen}
        onClose={() => setIsRonModalOpen(false)}
        ronText={selectedSystem?.ron_str || ''}
        particleName={selectedSystem?.name || ''}
        onApplyRon={handleApplyRon}
      />
    </div>
  )
}
export default App
