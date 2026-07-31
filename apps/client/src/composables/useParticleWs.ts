import { reactive, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 粒子渲染 server（lol_particle，默认 9002）只负责「播放」：
// 因为它唯一的输入是一段 ConfigVfxSystemDefinition 的 RON 字符串，所以英雄列表与
// 英雄粒子改由桌面端 Tauri 命令（list_particle_heroes / load_hero_particles）从本地
// 资产读取；本组合式仅通过原生 WebSocket 向 server 发送 play/stop 播放控制命令。
// 协议与 lol_server 一致：请求 { id, cmd, params }、响应 { id, type: "result", ok, data, error }。

export interface ParticleSystem {
  hash: number;
  name: string;
  /** 可直接发给 server 播放的 ConfigVfxSystemDefinition RON 字符串。 */
  defRon: string;
}

interface PendingRequest {
  resolve: (data: any) => void;
  reject: (err: Error) => void;
  timer: ReturnType<typeof setTimeout>;
}

interface WsResultFrame {
  id: number;
  type: "result";
  ok: boolean;
  data?: any;
  error?: string;
}

export const DEFAULT_PARTICLE_WS_URL = "ws://127.0.0.1:9002";

/** 请求超时（毫秒）：play/stop 命令都应秒回，超时即判定连接异常。 */
const REQUEST_TIMEOUT = 8000;

export function useParticleWs() {
  const connected = ref(false);
  const connecting = ref(false);
  const lastError = ref("");

  /** 英雄列表（名称升序，由 Tauri 命令 list_particle_heroes 返回）。 */
  const heroes = ref<string[]>([]);
  /** 已加载英雄 → 粒子系统列表（由 Tauri 命令 load_hero_particles 返回）。 */
  const heroSystems = reactive<Record<string, ParticleSystem[]>>({});
  /** 加载失败英雄 → 错误信息。 */
  const loadErrors = reactive<Record<string, string>>({});

  let ws: WebSocket | null = null;
  let nextId = 1;
  const pending = new Map<number, PendingRequest>();

  function rejectAllPending(reason: string) {
    for (const [, p] of pending) {
      clearTimeout(p.timer);
      p.reject(new Error(reason));
    }
    pending.clear();
  }

  // server 只返回 result 帧（播放/停止的应答），不再广播英雄相关事件。
  function handleFrame(text: string) {
    let frame: WsResultFrame;
    try {
      frame = JSON.parse(text);
    } catch {
      return;
    }
    if (frame.type !== "result") return;

    const p = pending.get(frame.id);
    if (!p) return;
    clearTimeout(p.timer);
    pending.delete(frame.id);
    if (frame.ok) {
      p.resolve(frame.data ?? {});
    } else {
      p.reject(new Error(frame.error || "请求失败"));
    }
  }

  function connect(url: string = DEFAULT_PARTICLE_WS_URL): Promise<void> {
    disconnect();
    lastError.value = "";
    connecting.value = true;

    return new Promise((resolve, reject) => {
      let settled = false;
      try {
        const sock = new WebSocket(url);
        ws = sock;
        sock.onopen = () => {
          connected.value = true;
          connecting.value = false;
          settled = true;
          resolve();
        };
        sock.onmessage = (ev) => handleFrame(typeof ev.data === "string" ? ev.data : "");
        sock.onerror = () => {
          lastError.value = "连接粒子渲染 server 失败";
        };
        sock.onclose = () => {
          connected.value = false;
          connecting.value = false;
          if (ws === sock) ws = null;
          rejectAllPending("连接已关闭");
          if (!settled) {
            settled = true;
            reject(new Error(lastError.value || "连接已关闭"));
          }
        };
      } catch (e: any) {
        connecting.value = false;
        lastError.value = e?.message || "无法建立 WS 连接";
        reject(new Error(lastError.value));
      }
    });
  }

  function disconnect() {
    if (ws) {
      ws.onclose = null;
      ws.onerror = null;
      ws.onmessage = null;
      ws.close();
      ws = null;
    }
    rejectAllPending("已断开连接");
    connected.value = false;
    connecting.value = false;
  }

  /** 发送一条 RPC 命令，返回响应 data；超时或失败则 reject。 */
  function request(cmd: string, params: Record<string, unknown> = {}): Promise<any> {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      return Promise.reject(new Error("粒子渲染 server 未连接"));
    }
    const id = nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        pending.delete(id);
        reject(new Error(`命令 ${cmd} 超时`));
      }, REQUEST_TIMEOUT);
      pending.set(id, { resolve, reject, timer });
      ws!.send(JSON.stringify({ id, cmd, params }));
    });
  }

  // ── 英雄 / 粒子：由桌面端 Tauri 命令读取本地资产（不经过 server） ──

  async function fetchHeroes(): Promise<string[]> {
    heroes.value = await invoke<string[]>("list_particle_heroes");
    return heroes.value;
  }

  /** 加载某英雄的粒子系统列表（含每个系统的 ConfigVfxSystemDefinition RON）。 */
  async function loadHero(name: string): Promise<void> {
    delete loadErrors[name];
    try {
      const systems = await invoke<ParticleSystem[]>("load_hero_particles", { hero: name });
      heroSystems[name] = systems;
    } catch (e: any) {
      loadErrors[name] = e?.message || String(e) || "加载失败";
      throw e;
    }
  }

  // ── 播放控制：原生 WS 向 server 发送 ConfigVfxSystemDefinition RON ──

  /** 播放一个粒子系统：把其 RON 定义发给 server。 */
  function playParticle(defRon: string): Promise<any> {
    return request("play_particle", { def: defRon });
  }

  function stopParticle(): Promise<any> {
    return request("stop_particle");
  }

  return {
    connected,
    connecting,
    lastError,
    heroes,
    heroSystems,
    loadErrors,
    connect,
    disconnect,
    fetchHeroes,
    loadHero,
    playParticle,
    stopParticle,
  };
}
