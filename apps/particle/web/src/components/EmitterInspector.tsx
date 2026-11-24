import React from 'react'
import {
  Play,
  RotateCcw,
  Sliders,
  Image as ImageIcon,
  Palette,
  Wind,
  Sparkles,
} from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Slider } from '@/components/ui/slider'
import { Switch } from '@/components/ui/switch'
import { ScrollArea } from '@/components/ui/scroll-area'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion'
import type { ConfigVfxEmitterDefinition, ConfigVfxSystemDefinition } from '../types/particle'
import { extractSamplerValue, formatNum } from '../lib/vfx-bridge'

interface EmitterInspectorProps {
  systemDef: ConfigVfxSystemDefinition | null
  activeEmitterIndex: number
  setActiveEmitterIndex: (index: number) => void
  onUpdateEmitter: (index: number, updated: ConfigVfxEmitterDefinition) => void
  onPlaySingleEmitter: (index: number) => void
  onResetEmitter: (index: number) => void
}

export const EmitterInspector: React.FC<EmitterInspectorProps> = ({
  systemDef,
  activeEmitterIndex,
  setActiveEmitterIndex,
  onUpdateEmitter,
  onPlaySingleEmitter,
  onResetEmitter,
}) => {
  if (!systemDef) {
    return (
      <div className="w-96 h-full border-l border-border/40 bg-sidebar/40 flex flex-col items-center justify-center p-6 text-center text-muted-foreground select-none">
        <Sparkles className="w-10 h-10 opacity-20 mb-2" />
        <span className="text-xs">请在左侧选择粒子系统以检视发射器</span>
      </div>
    )
  }

  const emitters: ConfigVfxEmitterDefinition[] =
    systemDef.complex_emitter_definition_data || systemDef.simple_emitter_definition_data || []

  if (emitters.length === 0) {
    return (
      <div className="w-96 h-full border-l border-border/40 bg-sidebar/40 flex flex-col items-center justify-center p-6 text-center text-muted-foreground select-none">
        <span className="text-xs">当前粒子系统未包含任何发射器</span>
      </div>
    )
  }

  const currentEmitter = emitters[activeEmitterIndex] || emitters[0]

  // 修改单个字段的通用处理
  const updateField = (key: keyof ConfigVfxEmitterDefinition, value: any) => {
    const updated = {
      ...currentEmitter,
      [key]: value,
    }
    onUpdateEmitter(activeEmitterIndex, updated)
  }

  // 修改采样器向量某分量
  const updateSamplerComp = (key: keyof ConfigVfxEmitterDefinition, compIndex: number, val: number) => {
    const raw = currentEmitter[key]
    const curVals = extractSamplerValue(raw)
    const newVals = [...curVals]
    newVals[compIndex] = val

    // 重新封装为常数采样器
    const newSampler = {
      base_sampler: newVals.length === 1 ? newVals[0] : (newVals as any),
      prob_curves: raw?.prob_curves || [],
    }
    updateField(key, newSampler)
  }

  // 提取当前发射器各项分量数值
  const scaleVals = extractSamplerValue(currentEmitter.birth_scale0)
  const velVals = extractSamplerValue(currentEmitter.birth_velocity)
  const accVals = extractSamplerValue(currentEmitter.birth_acceleration)
  const rotVals = extractSamplerValue(currentEmitter.birth_rotation0)
  const posVals = extractSamplerValue(currentEmitter.emitter_position)
  const colorVals = extractSamplerValue(currentEmitter.birth_color)

  // 混合模式映射
  const blendModeStr = currentEmitter.blend_mode !== undefined ? String(currentEmitter.blend_mode) : '1'

  return (
    <aside className="w-96 h-full border-l border-border/40 bg-sidebar/50 flex flex-col select-none overflow-hidden shrink-0">
      {/* 顶部：发射器标签页与单播控制 */}
      <div className="p-3 border-b border-border/40 flex flex-col gap-2 shrink-0 bg-background/50 backdrop-blur-sm">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-1.5">
            <Sliders className="w-4 h-4 text-purple-400" />
            <span className="text-xs font-bold text-foreground tracking-wide uppercase">发射器参数检查器</span>
          </div>

          <div className="flex items-center gap-1">
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onResetEmitter(activeEmitterIndex)}
              className="h-7 px-2 text-[11px] gap-1 text-muted-foreground hover:text-foreground"
            >
              <RotateCcw className="w-3 h-3" />
              <span>重置发射器</span>
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => onPlaySingleEmitter(activeEmitterIndex)}
              className="h-7 px-2 text-[11px] gap-1 text-purple-300 hover:text-purple-100 hover:bg-purple-950/40"
            >
              <Play className="w-3 h-3 fill-current" />
              <span>单播此发射器</span>
            </Button>
          </div>
        </div>

        {/* 发射器 Tab 列表 */}
        <ScrollArea className="w-full">
          <div className="flex items-center gap-1 py-1">
            {emitters.map((em, idx) => {
              const isActive = idx === activeEmitterIndex
              const label = em.emitter_name || `发射器 #${idx + 1}`

              return (
                <button
                  key={idx}
                  onClick={() => setActiveEmitterIndex(idx)}
                  className={`h-7 px-2.5 rounded-md text-xs font-medium whitespace-nowrap transition-all border ${
                    isActive
                      ? 'bg-purple-600 text-white border-purple-500 shadow-sm'
                      : 'bg-muted/40 text-muted-foreground hover:text-foreground border-border/30 hover:bg-muted/70'
                  }`}
                >
                  {label}
                </button>
              )
            })}
          </div>
        </ScrollArea>
      </div>

      {/* 参数编辑折叠列表 */}
      <ScrollArea className="flex-1 p-3">
        <div className="flex flex-col gap-3 pb-6">
          <Accordion type="multiple" defaultValue={['basic', 'transform', 'color', 'texture']} className="w-full">
            {/* 1. 基本属性 */}
            <AccordionItem value="basic" className="border-border/30">
              <AccordionTrigger className="text-xs font-semibold py-2 hover:no-underline text-purple-300">
                <div className="flex items-center gap-2">
                  <Sliders className="w-3.5 h-3.5" />
                  <span>基本属性 (Basic)</span>
                </div>
              </AccordionTrigger>
              <AccordionContent className="flex flex-col gap-3 pt-2">
                {/* 发射器名称 */}
                <div className="flex flex-col gap-1">
                  <label className="text-[11px] text-muted-foreground">发射器名称</label>
                  <Input
                    value={currentEmitter.emitter_name || ''}
                    onChange={(e) => updateField('emitter_name', e.target.value)}
                    placeholder="例如 Smoke_01"
                    className="h-7 text-xs bg-muted/40"
                  />
                </div>

                {/* 生命周期 (Lifetime) */}
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between text-[11px]">
                    <span className="text-muted-foreground">生命周期 (Lifetime)</span>
                    <span className="font-mono text-foreground font-semibold">
                      {formatNum(currentEmitter.lifetime)}s
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Slider
                      value={[currentEmitter.lifetime ?? 5]}
                      min={0.1}
                      max={100}
                      step={0.1}
                      onValueChange={([val]) => updateField('lifetime', val)}
                      className="flex-1"
                    />
                    <Input
                      type="number"
                      value={currentEmitter.lifetime ?? 5}
                      onChange={(e) => updateField('lifetime', parseFloat(e.target.value) || 0)}
                      className="w-16 h-7 text-xs font-mono bg-muted/40"
                    />
                  </div>
                </div>

                {/* 发射速率 (Rate) */}
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between text-[11px]">
                    <span className="text-muted-foreground">发射速率 (Rate)</span>
                    <span className="font-mono text-foreground font-semibold">
                      {formatNum(extractSamplerValue(currentEmitter.rate)[0])} /s
                    </span>
                  </div>
                  <div className="flex items-center gap-2">
                    <Slider
                      value={[extractSamplerValue(currentEmitter.rate)[0] || 10]}
                      min={0}
                      max={200}
                      step={1}
                      onValueChange={([val]) => updateField('rate', { base_sampler: val, prob_curves: [] })}
                      className="flex-1"
                    />
                    <Input
                      type="number"
                      value={extractSamplerValue(currentEmitter.rate)[0] || 10}
                      onChange={(e) =>
                        updateField('rate', { base_sampler: parseFloat(e.target.value) || 0, prob_curves: [] })
                      }
                      className="w-16 h-7 text-xs font-mono bg-muted/40"
                    />
                  </div>
                </div>

                {/* 混合模式 (Blend Mode) */}
                <div className="flex flex-col gap-1">
                  <label className="text-[11px] text-muted-foreground">混合模式 (Blend Mode)</label>
                  <Select
                    value={blendModeStr}
                    onValueChange={(val) => updateField('blend_mode', parseInt(val, 10))}
                  >
                    <SelectTrigger className="h-7 text-xs bg-muted/40">
                      <SelectValue placeholder="选择混合模式" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="0">0: None (不混合)</SelectItem>
                      <SelectItem value="1">1: AlphaBlend (透明混合)</SelectItem>
                      <SelectItem value="2">2: Additive (加色发光)</SelectItem>
                      <SelectItem value="3">3: Subtractive (减色)</SelectItem>
                      <SelectItem value="4">4: Multiply (乘法)</SelectItem>
                    </SelectContent>
                  </Select>
                </div>

                {/* 开关选项 */}
                <div className="grid grid-cols-2 gap-2 pt-1">
                  <div className="flex items-center justify-between p-2 rounded-md bg-muted/20 border border-border/20">
                    <label htmlFor="is-single" className="text-[11px] text-muted-foreground cursor-pointer">
                      单粒子模式
                    </label>
                    <Switch
                      id="is-single"
                      checked={!!currentEmitter.is_single_particle}
                      onCheckedChange={(val) => updateField('is_single_particle', val)}
                    />
                  </div>

                  <div className="flex items-center justify-between p-2 rounded-md bg-muted/20 border border-border/20">
                    <label htmlFor="is-dir" className="text-[11px] text-muted-foreground cursor-pointer">
                      朝向速度方向
                    </label>
                    <Switch
                      id="is-dir"
                      checked={!!currentEmitter.is_direction_oriented}
                      onCheckedChange={(val) => updateField('is_direction_oriented', val)}
                    />
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>

            {/* 2. 变换与力学 */}
            <AccordionItem value="transform" className="border-border/30">
              <AccordionTrigger className="text-xs font-semibold py-2 hover:no-underline text-purple-300">
                <div className="flex items-center gap-2">
                  <Wind className="w-3.5 h-3.5" />
                  <span>变换与动力学 (Dynamics)</span>
                </div>
              </AccordionTrigger>
              <AccordionContent className="flex flex-col gap-3 pt-2">
                {/* 初始尺寸 (Birth Scale) */}
                <div className="flex flex-col gap-1.5">
                  <span className="text-[11px] text-muted-foreground">初始尺寸 (Birth Scale X / Y / Z)</span>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['X', 'Y', 'Z'].map((axis, i) => (
                      <div key={axis} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{axis}</span>
                        <Input
                          type="number"
                          value={scaleVals[i] ?? 1}
                          onChange={(e) => updateSamplerComp('birth_scale0', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1.5"
                        />
                      </div>
                    ))}
                  </div>
                </div>

                {/* 初始速度 (Birth Velocity) */}
                <div className="flex flex-col gap-1.5">
                  <span className="text-[11px] text-muted-foreground">初速度 (Birth Velocity X / Y / Z)</span>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['X', 'Y', 'Z'].map((axis, i) => (
                      <div key={axis} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{axis}</span>
                        <Input
                          type="number"
                          value={velVals[i] ?? 0}
                          onChange={(e) => updateSamplerComp('birth_velocity', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1.5"
                        />
                      </div>
                    ))}
                  </div>
                </div>

                {/* 发射器位置 (Emitter Position) */}
                <div className="flex flex-col gap-1.5">
                  <span className="text-[11px] text-muted-foreground">发射器相对位置 (Position)</span>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['X', 'Y', 'Z'].map((axis, i) => (
                      <div key={axis} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{axis}</span>
                        <Input
                          type="number"
                          value={posVals[i] ?? 0}
                          onChange={(e) => updateSamplerComp('emitter_position', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1.5"
                        />
                      </div>
                    ))}
                  </div>
                </div>

                {/* 初始加速度 (Acceleration) */}
                <div className="flex flex-col gap-1.5">
                  <span className="text-[11px] text-muted-foreground">加速度 (Acceleration)</span>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['X', 'Y', 'Z'].map((axis, i) => (
                      <div key={axis} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{axis}</span>
                        <Input
                          type="number"
                          value={accVals[i] ?? 0}
                          onChange={(e) => updateSamplerComp('birth_acceleration', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1.5"
                        />
                      </div>
                    ))}
                  </div>
                </div>

                {/* 初始旋转 (Birth Rotation) */}
                <div className="flex flex-col gap-1.5">
                  <span className="text-[11px] text-muted-foreground">初始旋转 (Rotation X / Y / Z)</span>
                  <div className="grid grid-cols-3 gap-1.5">
                    {['X', 'Y', 'Z'].map((axis, i) => (
                      <div key={axis} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{axis}</span>
                        <Input
                          type="number"
                          value={rotVals[i] ?? 0}
                          onChange={(e) => updateSamplerComp('birth_rotation0', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1.5"
                        />
                      </div>
                    ))}
                  </div>
                </div>
              </AccordionContent>
            </AccordionItem>

            {/* 3. 色彩与材质 */}
            <AccordionItem value="color" className="border-border/30">
              <AccordionTrigger className="text-xs font-semibold py-2 hover:no-underline text-purple-300">
                <div className="flex items-center gap-2">
                  <Palette className="w-3.5 h-3.5" />
                  <span>色彩与渲染 (Color & Shading)</span>
                </div>
              </AccordionTrigger>
              <AccordionContent className="flex flex-col gap-3 pt-2">
                {/* 初始色彩 RGBA */}
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between">
                    <span className="text-[11px] text-muted-foreground">初始颜色 (Birth Color RGBA)</span>
                    <div
                      className="w-5 h-5 rounded border border-white/20 shadow-inner"
                      style={{
                        backgroundColor: `rgba(${Math.round((colorVals[0] ?? 1) * 255)}, ${Math.round(
                          (colorVals[1] ?? 1) * 255
                        )}, ${Math.round((colorVals[2] ?? 1) * 255)}, ${colorVals[3] ?? 1})`,
                      }}
                    />
                  </div>

                  <div className="grid grid-cols-4 gap-1.5">
                    {['R', 'G', 'B', 'A'].map((comp, i) => (
                      <div key={comp} className="flex items-center gap-1">
                        <span className="text-[10px] text-muted-foreground font-mono">{comp}</span>
                        <Input
                          type="number"
                          step={0.05}
                          min={0}
                          max={1}
                          value={colorVals[i] ?? 1}
                          onChange={(e) => updateSamplerComp('birth_color', i, parseFloat(e.target.value) || 0)}
                          className="h-7 text-xs font-mono bg-muted/40 px-1"
                        />
                      </div>
                    ))}
                  </div>
                </div>

                {/* Alpha 参考裁剪 */}
                <div className="flex flex-col gap-1.5">
                  <div className="flex items-center justify-between text-[11px]">
                    <span className="text-muted-foreground">Alpha 裁剪基准 (Alpha Ref)</span>
                    <span className="font-mono text-foreground font-semibold">
                      {currentEmitter.alpha_ref ?? 15}
                    </span>
                  </div>
                  <Slider
                    value={[currentEmitter.alpha_ref ?? 15]}
                    min={0}
                    max={255}
                    step={1}
                    onValueChange={([val]) => updateField('alpha_ref', val)}
                  />
                </div>
              </AccordionContent>
            </AccordionItem>

            {/* 4. 贴图与纹理 */}
            <AccordionItem value="texture" className="border-border/30">
              <AccordionTrigger className="text-xs font-semibold py-2 hover:no-underline text-purple-300">
                <div className="flex items-center gap-2">
                  <ImageIcon className="w-3.5 h-3.5" />
                  <span>贴图资源 (Texture Maps)</span>
                </div>
              </AccordionTrigger>
              <AccordionContent className="flex flex-col gap-3 pt-2">
                {/* 主纹理路径 */}
                <div className="flex flex-col gap-1">
                  <label className="text-[11px] text-muted-foreground">主粒子贴图 (Texture Path)</label>
                  <div className="flex items-center gap-1">
                    <Input
                      value={currentEmitter.texture?.path || ''}
                      onChange={(e) => updateField('texture', { path: e.target.value })}
                      placeholder="ASSETS/Characters/..."
                      className="h-7 text-[11px] font-mono bg-muted/40"
                    />
                  </div>
                  {currentEmitter.texture?.path && (
                    <span className="text-[9px] text-muted-foreground truncate" title={currentEmitter.texture.path}>
                      {currentEmitter.texture.path}
                    </span>
                  )}
                </div>

                {/* 侵蚀贴图路径 */}
                <div className="flex flex-col gap-1">
                  <label className="text-[11px] text-muted-foreground">侵蚀贴图 (Alpha Erosion Map)</label>
                  <Input
                    value={currentEmitter.alpha_erosion_definition?.erosion_map?.path || ''}
                    onChange={(e) => {
                      const curErosion = currentEmitter.alpha_erosion_definition || {}
                      updateField('alpha_erosion_definition', {
                        ...curErosion,
                        erosion_map: { path: e.target.value },
                      })
                    }}
                    placeholder="ASSETS/Characters/..._Erosion.png"
                    className="h-7 text-[11px] font-mono bg-muted/40"
                  />
                </div>
              </AccordionContent>
            </AccordionItem>
          </Accordion>
        </div>
      </ScrollArea>
    </aside>
  )
}
