// ── ServiceProvider ──
// 统一服务入口，管理 ILocalService / ICloudService / IEventBus 的生命周期
// 根据环境（Desktop/Web）和登录状态动态路由数据源

import type { ILocalService } from './local'
import type { ICloudService } from './cloud'
import type {
  SpawnPreset,
  HeroPreset,
  FrontAgentConfig,
  WinCondition,
  GameHistorySummary,
  SavedAgentHistory,
} from './types'
import { EventBusImpl, TauriEventAdapter, type IEventBus } from './eventBus'
import { CloudServiceImpl } from './cloudImpl'
import { isDesktop } from '@/lib/utils'

class ServiceProvider {
  private _local: ILocalService | null = null
  private _cloud: ICloudService | null = null
  private _eventBus: IEventBus = new EventBusImpl()
  private _tauriAdapter: TauriEventAdapter | null = null
  private _initialized = false

  /** Desktop 独占本地服务（Web 端访问会抛出错误） */
  get local(): ILocalService {
    if (!this._local) {
      throw new Error('[ServiceProvider] Local service not available (Web environment or not initialized)')
    }
    return this._local
  }

  /** 云端服务（需先 init） */
  get cloud(): ICloudService {
    if (!this._cloud) {
      throw new Error('[ServiceProvider] Cloud service not initialized. Call init() first.')
    }
    return this._cloud
  }

  /** 统一事件总线 */
  get events(): IEventBus {
    return this._eventBus
  }

  /** 是否已登录（云端可用） */
  get isOnline(): boolean {
    return this._cloud?.isAuthenticated() ?? false
  }

  /** 是否有本地服务可用 */
  get hasLocal(): boolean {
    return this._local !== null
  }

  /**
   * 初始化 ServiceProvider
   * - Desktop：动态加载 TauriLocalServiceImpl + 绑定 Tauri Event Adapter
   * - 所有环境：初始化 CloudServiceImpl
   */
  async init(): Promise<void> {
    if (this._initialized) return

    // 初始化云端服务（Desktop + Web 共用）
    const cloud = new CloudServiceImpl()
    cloud.onUnauthorized = () => {
      this._eventBus.emit('unauthorized', null)
    }
    this._cloud = cloud

    // Desktop：初始化本地服务
    if (isDesktop) {
      const { TauriLocalService } = await import('./localImpl')
      this._local = new TauriLocalService()

      // 绑定 Tauri 事件到 EventBus
      this._tauriAdapter = new TauriEventAdapter()
      await this._tauriAdapter.bind(this._eventBus)
    }

    this._initialized = true
  }

  // ── Spawn Presets (动态路由) ──
  async listSpawnPresets(): Promise<SpawnPreset[]> {
    return this.cloud.listSpawnPresets()
  }

  async saveSpawnPreset(preset: SpawnPreset): Promise<void> {
    const list = await this.cloud.listSpawnPresets()
    const existing = list.find((p) => p.name === preset.name)
    const body = {
      name: preset.name,
      x: preset.x,
      z: preset.z,
      team: preset.team.toLowerCase(),
      visibility: preset.visibility || 'private',
    }
    if (existing?.id) {
      await this.cloud.updateSpawnPreset(existing.id, body)
    } else {
      await this.cloud.createSpawnPreset(body)
    }
  }

  async deleteSpawnPreset(name: string): Promise<void> {
    const list = await this.cloud.listSpawnPresets()
    const existing = list.find((p) => p.name === name)
    if (existing?.id) {
      await this.cloud.deleteSpawnPreset(existing.id)
    }
  }

  // ── Hero Presets (动态路由) ──
  async listHeroPresets(): Promise<HeroPreset[]> {
    const list = await this.cloud.listAgents()
    return list.map((item) => ({
      id: item.id,
      name: item.name,
      champion: item.champion,
      agent_type: item.agent_type,
      prompt: item.prompt,
      preamble: item.preamble,
      model: item.model,
      config_json: item.config_json,
    }))
  }

  async saveHeroPreset(preset: HeroPreset): Promise<void> {
    const body = {
      name: preset.name,
      champion: preset.champion,
      agent_type: preset.agent_type,
      prompt: preset.prompt,
      preamble: preset.preamble || '',
      model: preset.model || '',
      config_json: preset.config_json || {},
      visibility: preset.visibility || 'private',
    }
    const list = await this.cloud.listAgents()
    const existing = list.find((p) => p.name === preset.name)
    if (existing?.id) {
      await this.cloud.updateAgent(existing.id, body)
    } else {
      await this.cloud.createAgent(body)
    }
  }

  async deleteHeroPreset(name: string): Promise<void> {
    const list = await this.cloud.listAgents()
    const existing = list.find((p) => p.name === name)
    if (existing?.id) {
      await this.cloud.deleteAgent(existing.id)
    }
  }

  // ── Custom Scenarios (动态路由) ──
  async listCustomScenarios(): Promise<string[]> {
    return this.cloud.listScenarios().then((list) => list.map((s) => s.name))
  }

  async loadCustomScenario(sceneName: string): Promise<FrontAgentConfig[]> {
    const list = await this.cloud.listScenarios()
    const existing = list.find((s) => s.name === sceneName)
    if (!existing) throw new Error(`Scenario not found: ${sceneName}`)
    return existing.agents
  }

  async saveCustomScenario(sceneName: string, agents: FrontAgentConfig[]): Promise<void> {
    const list = await this.cloud.listScenarios()
    const existing = list.find((s) => s.name === sceneName)
    const body = { name: sceneName, agents }
    if (existing?.id) {
      await this.cloud.updateScenario(existing.id, body)
    } else {
      await this.cloud.createScenario(body)
    }
  }

  async deleteCustomScenario(sceneName: string): Promise<void> {
    const list = await this.cloud.listScenarios()
    const existing = list.find((s) => s.name === sceneName)
    if (existing?.id) {
      await this.cloud.deleteScenario(existing.id)
    }
  }

  // ── Win Conditions (动态路由) ──
  async loadScenarioWinCondition(sceneName: string): Promise<WinCondition | null> {
    const list = await this.cloud.listScenarios()
    const existing = list.find((s) => s.name === sceneName)
    if (!existing) return null
    return this.cloud.getScenarioWinCondition(existing.id)
  }

  async saveScenarioWinCondition(sceneName: string, condition: WinCondition): Promise<void> {
    const list = await this.cloud.listScenarios()
    const existing = list.find((s) => s.name === sceneName)
    if (!existing) throw new Error(`Scenario not found: ${sceneName}`)
    await this.cloud.setScenarioWinCondition(existing.id, condition)
  }

  // ── Game History (动态路由) ──
  async listGameHistories(): Promise<GameHistorySummary[]> {
    return this.cloud.listGameHistories()
  }

  async getGameHistoryDetail(datetime: string): Promise<SavedAgentHistory[]> {
    return this.cloud.getGameHistoryDetail(datetime)
  }

  async deleteGameHistory(datetime: string): Promise<void> {
    return this.cloud.deleteGameHistory(datetime)
  }

  /** 销毁 ServiceProvider（清理事件适配器） */
  dispose(): void {
    this._tauriAdapter?.dispose()
    this._tauriAdapter = null
    this._initialized = false
  }
}

// ── 单例导出 ──

export const services = new ServiceProvider()

/**
 * 初始化 ServiceProvider（在 main.ts 中调用）
 * 替代原来的 getBackendClient()
 */
export async function initServices(): Promise<void> {
  await services.init()
}
