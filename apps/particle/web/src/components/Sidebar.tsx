import React, { useState, useMemo } from 'react'
import {
  Search,
  ChevronDown,
  ChevronRight,
  Sparkles,
  Layers,
  X,
  Shield,
  Sword,
  Loader2,
} from 'lucide-react'
import { Input } from '@/components/ui/input'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Tabs, TabsList, TabsTrigger } from '@/components/ui/tabs'
import type { WebHeroSummary, WebSystemSummaryItem } from '../types/particle'

interface SidebarProps {
  heroes: WebHeroSummary[]
  selectedHero: string | null
  selectedHash: number | null
  onSelectSystem: (heroId: string, system: WebSystemSummaryItem) => void
  isLoadingDetails: boolean
}

export const Sidebar: React.FC<SidebarProps> = ({
  heroes,
  selectedHero,
  selectedHash,
  onSelectSystem,
  isLoadingDetails,
}) => {
  const [query, setQuery] = useState('')
  const [activeCategory, setActiveCategory] = useState<'all' | 'five_tigers' | 'four_sisters'>('all')
  const [expandedHeroes, setExpandedHeroes] = useState<Record<string, boolean>>({
    riven: true,
    aatrox: true,
  })

  const toggleExpand = (id: string) => {
    setExpandedHeroes((prev) => ({
      ...prev,
      [id]: !prev[id],
    }))
  }

  // 过滤后的英雄和系统
  const filteredHeroes = useMemo(() => {
    const q = query.trim().toLowerCase()
    return heroes
      .filter((hero) => {
        // 分类过滤
        if (activeCategory === 'five_tigers' && hero.category !== 'five_tigers') return false
        if (activeCategory === 'four_sisters' && hero.category !== 'four_sisters') return false

        // 搜索关键词过滤
        if (!q) return true
        if (hero.name.toLowerCase().includes(q)) return true
        return hero.systems.some(
          (sys) => sys.name.toLowerCase().includes(q) || sys.hash_hex.toLowerCase().includes(q)
        )
      })
      .map((hero) => {
        const q = query.trim().toLowerCase()
        if (!q) return hero
        const matchedSystems = hero.systems.filter(
          (sys) => sys.name.toLowerCase().includes(q) || sys.hash_hex.toLowerCase().includes(q)
        )
        return {
          ...hero,
          systems: matchedSystems,
        }
      })
  }, [heroes, activeCategory, query])

  return (
    <aside className="w-80 h-full flex flex-col bg-sidebar/50 border-r border-sidebar-border select-none shrink-0 overflow-hidden">
      {/* 侧边栏顶部：标题与搜索 */}
      <div className="p-3 border-b border-sidebar-border/60 flex flex-col gap-2.5">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Layers className="w-4 h-4 text-purple-400" />
            <span className="text-xs font-bold tracking-wide text-foreground uppercase">粒子系统资产库</span>
          </div>
          <Badge variant="outline" className="text-[10px] text-muted-foreground border-border/40">
            {heroes.length} 英雄
          </Badge>
        </div>

        {/* 检索框 */}
        <div className="relative">
          {isLoadingDetails ? (
            <Loader2 className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-purple-400 animate-spin" />
          ) : (
            <Search className="absolute left-2.5 top-2.5 h-3.5 w-3.5 text-muted-foreground/70" />
          )}
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="搜索英雄、粒子名称或 Hash..."
            className="h-8 pl-8 pr-7 text-xs bg-muted/40 border-border/40 focus-visible:ring-purple-500/30"
          />
          {query && (
            <button
              onClick={() => setQuery('')}
              className="absolute right-2 top-2.5 text-muted-foreground/70 hover:text-foreground"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
        </div>

        {/* 预设分类快捷栏 */}
        <Tabs
          value={activeCategory}
          onValueChange={(v) => setActiveCategory(v as any)}
          className="w-full"
        >
          <TabsList className="grid grid-cols-3 h-7 bg-muted/30 p-0.5 text-[11px]">
            <TabsTrigger value="all" className="h-6 text-[11px] data-[state=checked]:bg-background">
              全部
            </TabsTrigger>
            <TabsTrigger value="five_tigers" className="h-6 text-[11px] gap-1 data-[state=checked]:bg-background">
              <Shield className="w-3 h-3 text-amber-400" />
              五虎
            </TabsTrigger>
            <TabsTrigger value="four_sisters" className="h-6 text-[11px] gap-1 data-[state=checked]:bg-background">
              <Sword className="w-3 h-3 text-pink-400" />
              四姐妹
            </TabsTrigger>
          </TabsList>
        </Tabs>
      </div>

      {/* 英雄与系统列表树 */}
      <ScrollArea className="flex-1 p-2">
        <div className="flex flex-col gap-1 pr-2">
          {filteredHeroes.length === 0 ? (
            <div className="py-12 flex flex-col items-center justify-center text-center gap-2 text-muted-foreground">
              <Sparkles className="w-8 h-8 opacity-20" />
              <span className="text-xs">未找到匹配的粒子系统</span>
            </div>
          ) : (
            filteredHeroes.map((hero) => {
              const isExpanded = query.trim() ? true : !!expandedHeroes[hero.id]
              const isHeroActive = selectedHero?.toLowerCase() === hero.id.toLowerCase()

              return (
                <div key={hero.id} className="flex flex-col rounded-md overflow-hidden transition-colors">
                  {/* 英雄行 */}
                  <div
                    onClick={() => toggleExpand(hero.id)}
                    className={`flex items-center justify-between px-2 py-1.5 rounded-md cursor-pointer text-xs font-medium transition-colors ${
                      isHeroActive ? 'bg-purple-950/30 text-purple-200' : 'hover:bg-muted/50 text-foreground'
                    }`}
                  >
                    <div className="flex items-center gap-2 truncate">
                      {isExpanded ? (
                        <ChevronDown className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                      ) : (
                        <ChevronRight className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                      )}
                      {hero.category === 'five_tigers' && (
                        <Shield className="w-3 h-3 text-amber-400 shrink-0" />
                      )}
                      {hero.category === 'four_sisters' && (
                        <Sword className="w-3 h-3 text-pink-400 shrink-0" />
                      )}
                      <span className="truncate">{hero.name}</span>
                    </div>

                    <Badge
                      variant="secondary"
                      className="text-[10px] px-1.5 py-0 h-4 font-mono text-muted-foreground bg-muted/60"
                    >
                      {hero.systems.length}
                    </Badge>
                  </div>

                  {/* 展开的粒子系统项 */}
                  {isExpanded && (
                    <div className="flex flex-col gap-0.5 pl-5 pr-1 py-1 border-l border-border/30 ml-3.5 my-0.5">
                      {hero.systems.map((sys) => {
                        const isSelected = selectedHash === sys.hash

                        return (
                          <div
                            key={sys.hash}
                            onClick={() => onSelectSystem(hero.id, sys)}
                            className={`group relative flex items-center justify-between px-2.5 py-1.5 rounded-sm cursor-pointer text-[11px] transition-all ${
                              isSelected
                                ? 'bg-purple-600/20 text-purple-200 font-medium shadow-sm border border-purple-500/30'
                                : 'hover:bg-muted/40 text-muted-foreground hover:text-foreground'
                            }`}
                          >
                            {isSelected && (
                              <div className="absolute left-0 top-1 bottom-1 w-0.5 bg-purple-500 rounded-full" />
                            )}
                            <div className="flex flex-col truncate pr-2">
                              <span className="truncate" title={sys.name}>
                                {sys.name}
                              </span>
                              <span className="text-[9px] font-mono opacity-60">
                                {sys.hash_hex}
                              </span>
                            </div>

                            <Badge
                              variant="outline"
                              className={`text-[9px] px-1 py-0 h-3.5 shrink-0 font-mono border-0 ${
                                isSelected ? 'bg-purple-950/60 text-purple-300' : 'bg-muted/40 text-muted-foreground'
                              }`}
                            >
                              {sys.emitter_count}E
                            </Badge>
                          </div>
                        )
                      })}
                    </div>
                  )}
                </div>
              )
            })
          )}
        </div>
      </ScrollArea>
    </aside>
  )
}
