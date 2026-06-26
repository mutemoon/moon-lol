// ── ServiceProvider ──
// 统一服务入口，管理 ILocalService / ICloudService / IEventBus 的生命周期
// 根据环境（Desktop/Web）和登录状态动态路由数据源

import type { ILocalService } from './local'
import type { ICloudService } from './cloud'
import { EventBusImpl, TauriEventAdapter, type IEventBus } from './eventBus'
import { CloudServiceImpl } from './cloudImpl'

// ── 环境检测 ──

const isTauriEnv =
  typeof window !== 'undefined' &&
  ((window as any).__TAURI__ !== undefined || (window as any).__TAURI_INTERNALS__ !== undefined) &&
  (window as any).IS_DESKTOP !== false

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

  /** 是否处于 Tauri Desktop 环境 */
  get isDesktop(): boolean {
    return isTauriEnv
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
    if (isTauriEnv) {
      const { TauriLocalServiceImpl } = await import('./localImpl')
      this._local = new TauriLocalServiceImpl()

      // 绑定 Tauri 事件到 EventBus
      this._tauriAdapter = new TauriEventAdapter()
      await this._tauriAdapter.bind(this._eventBus)
    }

    this._initialized = true
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
