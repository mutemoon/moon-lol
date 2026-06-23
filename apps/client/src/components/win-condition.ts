// 胜利条件类型定义与原子条件清单。
// 抽出到独立 .ts 以便 WinNode / WinConditionBuilder / 页面共享，
// 避免 <script setup> 中使用 ES export（SFC 编译器不支持）。

export interface AtomTypeDef {
  value: string;
  label: string;
  param: { key: string; label: string; def: number };
}

// 基础条件原子清单（与 PRODUCT.md §3.3 对齐）。
export const ATOM_TYPES: AtomTypeDef[] = [
  { value: "minion_kills", label: "补刀数达标", param: { key: "n", label: "补刀数", def: 10 } },
  { value: "kills", label: "击杀英雄", param: { key: "n", label: "击杀数", def: 1 } },
  { value: "turret_destroyed", label: "摧毁防御塔", param: { key: "tier", label: "塔层", def: 1 } },
  {
    value: "inhibitor_destroyed",
    label: "摧毁水晶",
    param: { key: "lane", label: "路线(0/1/2)", def: 0 },
  },
  { value: "nexus_destroyed", label: "摧毁基地", param: { key: "k", label: "-", def: 0 } },
  { value: "gold_lead", label: "经济领先", param: { key: "n", label: "经济差", def: 2000 } },
  { value: "survive_duration", label: "存活时长", param: { key: "t", label: "秒", def: 300 } },
  { value: "total_damage", label: "总伤害量", param: { key: "n", label: "伤害量", def: 10000 } },
];

export interface WinCondition {
  op: "atom" | "and" | "or" | "not";
  type?: string;
  params?: Record<string, number | string>;
  children?: WinCondition[];
  child?: WinCondition;
}
