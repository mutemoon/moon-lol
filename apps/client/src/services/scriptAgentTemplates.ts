// Script Agent 宿主 API 的 TypeScript 类型声明与内置脚本模板。
//
// 这里的 `.d.ts` 文本会在运行时通过 `monaco.languages.typescript` 注入编辑器，
// 让 Script Agent 的脚本（JS）获得 `observe()` / `action()` / `log()` / `wait_ticks()`
// 的精确补全与类型推导。类型字段与服务端 `lol_agent::models`（Observe* 系列）、
// `lol_core::action::Action` 的序列化结构保持一致——改动 Rust 模型时记得同步这里。

/** Script Agent 宿主 API 的 `.d.ts`，注入 Monaco 作为额外类型库。 */
export const SCRIPT_API_DTS = `
/** 二维坐标，[x, z]（游戏使用 X/Z 平面）。 */
type Vec2 = [number, number];

/** 自动攻击阶段。 */
interface AttackState {
  /** 当前阶段名，如 "Windup" | "Recovery" 等。 */
  phase: string;
  target: number | null;
}

/** 菲奥娜被动破绽（仅菲奥娜对局可见）。 */
interface Vital {
  position: Vec2;
}

/** 单个技能的状态。 */
interface ObserveSkill {
  /** 技能槽位下标：0=Q 1=W 2=E 3=R。 */
  index: number;
  /** 技能等级，0 表示未学习。 */
  level: number;
  /** null 表示就绪可用；否则为剩余冷却秒数。 */
  cooldown_remaining: number | null;
}

/** 敌方/友方英雄的观测信息。 */
interface ObserveHero {
  /** 实体 ID，传给 action 作为攻击目标。 */
  entity: number;
  position: Vec2;
  health: number;
  max_health: number;
  /** 与自己的距离。 */
  distance: number;
}

/** 小兵观测信息。 */
interface ObserveMinion {
  entity: number;
  position: Vec2;
  health: number;
  distance: number;
  vital: Vital | null;
}

/** 自己的完整状态。 */
interface ObserveMyself {
  position: Vec2;
  attack_state: AttackState | null;
  /** 当前移动目标点（若正在移动）。 */
  run_target: Vec2 | null;
  health: number;
  max_health: number;
  level: number;
  /** [当前值, 最大值]，无蓝量资源时为 null。 */
  ability_resource: [number, number] | null;
  attack_damage: number;
  attack_range: number;
  attack_speed: number;
  armor: number;
  /** 可分配的技能点数。 */
  skill_points: number;
  skills: ObserveSkill[];
  gold: number;
  kills: number;
  deaths: number;
  assists: number;
  minion_kills: number;
}

/** 一次完整的局势观测快照。 */
interface Observe {
  /** 对局已进行的秒数。 */
  time: number;
  myself: ObserveMyself;
  /** 视野内的敌方小兵，按距离升序。 */
  minions: ObserveMinion[];
  /** 视野内的友方英雄，按距离升序。 */
  friendly_heroes: ObserveHero[];
  /** 视野内的敌方英雄，按距离升序。 */
  enemy_heroes: ObserveHero[];
}

/**
 * 下发的动作指令（与服务端 serde 外部标签枚举一致）：
 * - "Stop"                                     停止移动与攻击
 * - { Attack: entityId }                       自动攻击目标实体
 * - { Move: [x, z] }                           移动到坐标
 * - { Skill: { index, point } }                在指定点释放技能
 * - { SkillLevelUp: index }                    升级技能
 */
type Action =
  | "Stop"
  | { Attack: number }
  | { Move: Vec2 }
  | { Skill: { index: number; point: Vec2 } }
  | { SkillLevelUp: number };

/** 获取当前局势观测快照。 */
declare function observe(): Observe;

/** 下发一个动作指令到游戏引擎。 */
declare function action(a: Action): void;

/** 输出一条调试日志，会显示在脚本调试面板。 */
declare function log(...args: unknown[]): void;

/** 让脚本挂起 n 个 tick 后再继续（协作式让出，避免阻塞引擎）。 */
declare function wait_ticks(n: number): void;
`;

export interface ScriptTemplate {
  id: string;
  /** i18n key 后缀，用于在选择器里展示本地化名称。 */
  labelKey: string;
  code: string;
}

const KITE = `// 走 A（风筝）：攻击就绪时打最近的敌方英雄，否则朝其反向后撤。
const obs = observe();
const enemy = obs.enemy_heroes[0];
if (!enemy) {
  action("Stop");
} else {
  const me = obs.myself;
  const ready = me.attack_state === null;
  if (ready && enemy.distance <= me.attack_range) {
    action({ Attack: enemy.entity });
  } else {
    // 朝远离敌人的方向后撤一个身位。
    const dx = me.position[0] - enemy.position[0];
    const dz = me.position[1] - enemy.position[1];
    const len = Math.hypot(dx, dz) || 1;
    action({ Move: [me.position[0] + (dx / len) * 150, me.position[1] + (dz / len) * 150] });
  }
}
`;

const LAST_HIT = `// 智能补刀：仅在小兵血量进入一次攻击可击杀区间时出手。
const obs = observe();
const me = obs.myself;
const target = obs.minions.find(
  (m) => m.distance <= me.attack_range && m.health <= me.attack_damage,
);
if (target) {
  action({ Attack: target.entity });
} else if (obs.minions[0]) {
  // 没有可补的兵就靠近兵线等待。
  action({ Move: obs.minions[0].position });
} else {
  action("Stop");
}
`;

const COMBO = `// 技能连招：贴脸 Q(0) -> 普攻 -> 收到位移即追击，资源/冷却就绪才释放。
const obs = observe();
const me = obs.myself;
const enemy = obs.enemy_heroes[0];
if (!enemy) {
  action("Stop");
} else {
  const q = me.skills[0];
  const inRange = enemy.distance <= me.attack_range + 50;
  if (q && q.level > 0 && q.cooldown_remaining === null && inRange) {
    action({ Skill: { index: 0, point: enemy.position } });
  } else if (me.attack_state === null && enemy.distance <= me.attack_range) {
    action({ Attack: enemy.entity });
  } else {
    action({ Move: enemy.position });
  }
}
`;

/** 内置脚本模板库，供编辑器一键填充。 */
export const SCRIPT_TEMPLATES: ScriptTemplate[] = [
  { id: "kite", labelKey: "kite", code: KITE },
  { id: "lastHit", labelKey: "lastHit", code: LAST_HIT },
  { id: "combo", labelKey: "combo", code: COMBO },
];

/** 新建 Script Agent 时的默认脚本骨架。 */
export const DEFAULT_SCRIPT = `// 每个 tick 调用一次：读取观测 -> 决策 -> 下发动作。
// 通过右上角「模板」可一键填入常用脚本（走A / 补刀 / 连招）。
const obs = observe();
log("hp", obs.myself.health, "enemies", obs.enemy_heroes.length);
action("Stop");
`;
