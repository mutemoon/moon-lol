import type { Ref } from "vue";
import type { HeroPreset, AgentPreset, SpawnPreset } from "../stores/gameStore";

// 槽位结构：承载"英雄预设 + 解绑后的临时覆盖"。
// - heroPresetName: 当前绑定的英雄预设名；解绑后置空
// - champion / agentPresetName / spawnPresetName: 展开后的具体值（继承或被覆盖）
// - dirty: 是否已与原预设解绑（存在未保存的临时覆盖）
export interface Slot {
  id: string;
  heroPresetName: string;
  champion: string;
  agentPresetName: string;
  spawnPresetName: string;
  dirty: boolean;
}

// 槽位唯一 id 生成器（模块级自增，避免跨组件实例冲突）
let _idSeq = 0;
export const nextSlotId = () => `slot_${++_idSeq}_${Date.now().toString(36)}`;

export const emptySlot = (): Slot => ({
  id: nextSlotId(),
  heroPresetName: "",
  champion: "",
  agentPresetName: "",
  spawnPresetName: "",
  dirty: false,
});

// 选中一个英雄预设：把槽位字段填充为继承值，绑定态
export function bindHeroPreset(
  slot: Slot,
  name: string,
  heroPresets: HeroPreset[],
) {
  const hero = heroPresets.find((p) => p.name === name);
  if (!hero) {
    slot.heroPresetName = "";
    slot.champion = "";
    slot.agentPresetName = "";
    slot.spawnPresetName = "";
    slot.dirty = false;
    return;
  }
  slot.heroPresetName = name;
  slot.champion = hero.champion;
  slot.agentPresetName = hero.agent_preset_name;
  slot.spawnPresetName = hero.spawn_preset_name;
  slot.dirty = false;
}

// 槽位 Agent / 出生点被覆盖：立即与原预设解绑（仅保留具体值）
export function overrideAgent(slot: Slot, name: string) {
  slot.agentPresetName = name;
  if (slot.heroPresetName) {
    slot.heroPresetName = "";
    slot.dirty = true;
  }
}

export function overrideSpawn(slot: Slot, name: string) {
  slot.spawnPresetName = name;
  if (slot.heroPresetName) {
    slot.heroPresetName = "";
    slot.dirty = true;
  }
}

// 槽位副标题：英雄 · Agent 类型（取当前展开值，而非反查预设）
// 槽位副标题：英雄 · Agent 类型（取当前展开值，而非反查预设）
export function slotSubtitle(slot: Slot, agentPresets: AgentPreset[]): string {
  if (!slot.champion) return "";
  const agent = slot.agentPresetName
    ? agentPresets.find((p) => p.name === slot.agentPresetName)
    : undefined;
  const typeTag = agent ? agent.agent_type.toUpperCase() : "DEFAULT";
  return `${slot.champion} · ${typeTag}`;
}

// 展开：槽位 ↔ 后端 FrontAgentConfig（把所选预设展开为具体值）
export function expandSlot(team: "Order" | "Chaos", slot: Slot, agentPresets: AgentPreset[], spawnPresets: SpawnPreset[]) {
  const champion = slot.champion || "Riven";
  const agent = slot.agentPresetName
    ? agentPresets.find((p) => p.name === slot.agentPresetName)
    : undefined;
  const spawn = slot.spawnPresetName
    ? spawnPresets.find((p) => p.name === slot.spawnPresetName)
    : undefined;
  return {
    champion,
    team,
    prompt: agent?.prompt ?? "",
    spawn_point: spawn ? [spawn.x, spawn.z] : ([1500, 2000] as [number, number]),
    agent_type: agent?.agent_type ?? "llm",
  };
}

export function toBackend(
  team: "Order" | "Chaos",
  slots: Slot[],
  agentPresets: AgentPreset[],
  spawnPresets: SpawnPreset[],
) {
  return slots.filter((s) => s.champion).map((s) => expandSlot(team, s, agentPresets, spawnPresets));
}

// 加载场景：把后端具体值反向匹配回英雄预设名
export function matchHeroPreset(
  champion: string,
  prompt: string,
  agentType: string,
  x: number,
  z: number,
  heroPresets: HeroPreset[],
  agentPresets: AgentPreset[],
  spawnPresets: SpawnPreset[],
): string {
  // 优先精确匹配：英雄预设绑定的 Agent/出生点展开后与场景值完全一致
  for (const hero of heroPresets) {
    if (hero.champion !== champion) continue;
    const agent = hero.agent_preset_name
      ? agentPresets.find((p) => p.name === hero.agent_preset_name)
      : undefined;
    const spawn = hero.spawn_preset_name
      ? spawnPresets.find((p) => p.name === hero.spawn_preset_name)
      : undefined;
    const promptOk = !hero.agent_preset_name || (agent?.prompt ?? "") === (prompt ?? "");
    const typeOk = !hero.agent_preset_name || (agent?.agent_type ?? "llm") === (agentType ?? "llm");
    const spawnOk =
      !hero.spawn_preset_name || (spawn && Math.abs(spawn.x - x) < 1 && Math.abs(spawn.z - z) < 1);
    if (promptOk && typeOk && spawnOk) return hero.name;
  }
  // 退化：仅按英雄名匹配
  return heroPresets.find((p) => p.champion === champion)?.name ?? "";
}

export function fromBackend(
  agents: any[],
  heroPresets: HeroPreset[],
  agentPresets: AgentPreset[],
  spawnPresets: SpawnPreset[],
): Slot[] {
  return agents.map((a) => {
    const matched = matchHeroPreset(
      a.champion,
      a.prompt ?? "",
      a.agent_type ?? "llm",
      a.spawn_point[0],
      a.spawn_point[1],
      heroPresets,
      agentPresets,
      spawnPresets,
    );
    // 反查 Agent / 出生点预设名（按场景里的 prompt/坐标精确匹配）
    const agentName =
      agentPresets.find(
        (p) => (p.prompt ?? "") === (a.prompt ?? "") && (p.agent_type ?? "llm") === (a.agent_type ?? "llm"),
      )?.name ?? "";
    const spawnName =
      spawnPresets.find((p) => Math.abs(p.x - a.spawn_point[0]) < 1 && Math.abs(p.z - a.spawn_point[1]) < 1)
        ?.name ?? "";
    return {
      id: nextSlotId(),
      heroPresetName: matched,
      champion: a.champion,
      agentPresetName: agentName,
      spawnPresetName: spawnName,
      // 匹配不到完整预设 → 视为已解绑的临时配置
      dirty: !matched,
    };
  });
}

// 「存为新预设」自动命名：重名时追加 · 副本 2/3…
export function uniquePresetName(base: string, heroPresets: HeroPreset[], suffix: string = "副本"): string {
  const exists = (n: string) => heroPresets.some((p) => p.name === n);
  if (!exists(base)) return base;
  let i = 2;
  while (exists(`${base} · ${suffix} ${i}`)) i++;
  return `${base} · ${suffix} ${i}`;
}

// 把槽位的预设引用绑定回某个英雄预设名（存为新预设后回填用）
export function rebindSlot(slot: Slot, name: string) {
  slot.heroPresetName = name;
  slot.dirty = false;
}

// 占位：导入 Ref 仅为类型约束（避免某些 lint 规则报未使用）
export type { Ref };
