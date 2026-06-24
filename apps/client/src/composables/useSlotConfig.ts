import type { Ref } from "vue";
import type { HeroPreset, SpawnPreset } from "../stores/gameStore";

// 槽位结构：独立承载"选手预设 + 出生点预设"。
export interface Slot {
  id: string;
  heroPresetName: string; // 选手预设名（原英雄预设名）
  champion: string;        // 继承的英雄名
  spawnPresetName: string; // 独立的出生点预设名
  dirty: boolean;          // 是否已与原预设解绑
}

// 槽位唯一 id 生成器
let _idSeq = 0;
export const nextSlotId = () => `slot_${++_idSeq}_${Date.now().toString(36)}`;

export const emptySlot = (): Slot => ({
  id: nextSlotId(),
  heroPresetName: "",
  champion: "",
  spawnPresetName: "",
  dirty: false,
});

// 绑定选手预设到槽位
export function bindHeroPreset(
  slot: Slot,
  name: string,
  heroPresets: HeroPreset[],
) {
  const hero = heroPresets.find((p) => p.name === name);
  if (!hero) {
    slot.heroPresetName = "";
    slot.champion = "";
    slot.dirty = false;
    return;
  }
  slot.heroPresetName = name;
  slot.champion = hero.champion;
  slot.dirty = false;
}

// 槽位出生点选择（不会清除选手绑定，因为二者已独立）
export function selectSpawn(slot: Slot, name: string) {
  slot.spawnPresetName = name;
}

// 槽位副标题：显示英雄名与策略驱动类型 (LLM/RL/SCRIPT)
export function slotSubtitle(slot: Slot, heroPresets: HeroPreset[]): string {
  if (!slot.champion) return "";
  const hero = slot.heroPresetName
    ? heroPresets.find((p) => p.name === slot.heroPresetName)
    : undefined;
  const typeTag = hero ? hero.agent_type.toUpperCase() : "DEFAULT";
  return `${slot.champion} · ${typeTag}`;
}

// 展开槽位配置，输出为后端的 FrontAgentConfig
export function expandSlot(
  team: "Order" | "Chaos",
  slot: Slot,
  heroPresets: HeroPreset[],
  spawnPresets: SpawnPreset[],
) {
  const champion = slot.champion || "Riven";
  const hero = slot.heroPresetName
    ? heroPresets.find((p) => p.name === slot.heroPresetName)
    : undefined;
  const spawn = slot.spawnPresetName
    ? spawnPresets.find((p) => p.name === slot.spawnPresetName)
    : undefined;

  return {
    champion,
    team,
    prompt: hero?.prompt ?? "",
    spawn_point: spawn ? [spawn.x, spawn.z] : (team === "Order" ? [1981, 11441] : [3318, 12875]),
    agent_type: hero?.agent_type ?? "llm",
  };
}

// 将多个槽位转换为后端的 FrontAgentConfig 数组
export function toBackend(
  team: "Order" | "Chaos",
  slots: Slot[],
  heroPresets: HeroPreset[],
  spawnPresets: SpawnPreset[],
) {
  return slots
    .filter((s) => s.champion)
    .map((s) => expandSlot(team, s, heroPresets, spawnPresets));
}

// 将后端状态转换为前端槽位结构
export function matchHeroPreset(
  champion: string,
  prompt: string,
  agentType: string,
  heroPresets: HeroPreset[],
): string {
  for (const hero of heroPresets) {
    if (hero.champion !== champion) continue;
    const promptOk = (hero.prompt ?? "") === (prompt ?? "");
    const typeOk = (hero.agent_type ?? "llm") === (agentType ?? "llm");
    if (promptOk && typeOk) return hero.name;
  }
  return heroPresets.find((p) => p.champion === champion)?.name ?? "";
}

export function fromBackend(
  agents: any[],
  heroPresets: HeroPreset[],
  spawnPresets: SpawnPreset[],
): Slot[] {
  return agents.map((a) => {
    const matched = matchHeroPreset(
      a.champion,
      a.prompt ?? "",
      a.agent_type ?? "llm",
      heroPresets,
    );
    const spawnName =
      spawnPresets.find(
        (p) =>
          Math.abs(p.x - a.spawn_point[0]) < 1 &&
          Math.abs(p.z - a.spawn_point[1]) < 1,
      )?.name ?? "";
    return {
      id: nextSlotId(),
      heroPresetName: matched,
      champion: a.champion,
      spawnPresetName: spawnName,
      dirty: !matched,
    };
  });
}

// 自动命名助手
export function uniquePresetName(
  base: string,
  heroPresets: HeroPreset[],
  suffix: string = "副本",
): string {
  const exists = (n: string) => heroPresets.some((p) => p.name === n);
  if (!exists(base)) return base;
  let i = 2;
  while (exists(`${base} · ${suffix} ${i}`)) i++;
  return `${base} · ${suffix} ${i}`;
}

export function rebindSlot(slot: Slot, name: string) {
  slot.heroPresetName = name;
  slot.dirty = false;
}

export type { Ref };
