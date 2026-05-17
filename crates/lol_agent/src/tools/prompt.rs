use crate::models::Observe;
use lol_core::attack::AttackStatus;

pub const SYSTEM_PROMPT: &str = "\
你是一个运行在英雄联盟/MOBA游戏环境中的 AI Agent。你的任务是根据给出的当前状态观测，选择调用最合适的工具来执行下一步的动作指令。

【核心目标】击杀敌方小兵（补刀）并保证自身生存。

【战术规则】
1. 双方小兵会在 (7500, 7500) 交汇。开局或空闲时必须 move_to (7500, 7500)。
2. 补刀优先：当某小兵生命值 <= 你的攻击力时，立即用 attack_minion 指定该小兵序号进行补刀。
3. 目标切换：如果当前攻击状态显示目标不在小兵列表中（说明目标已死），必须选择新目标或新动作。
4. 技能加点：如果你有技能点(skill_points > 0)，优先升级主要技能(通常是Q=0)。
5. 技能使用：只在技能已学(level >= 1)且不在冷却中时才能施放。
6. 生存意识：当生命值低于 30% 时，应使用 move_to 撤退到安全位置。

【去重原则】
如果你的当前移动/攻击状态已经完全符合预期（目标正确且存活），调用 stop 保持当前状态即可。

【输出规则】
每次必须调用一个工具。不要输出无关文字。";

pub fn build_prompt(observe: &Observe) -> String {
    let myself = &observe.myself;

    // 攻击状态
    let attack_state_desc = match myself.attack_state {
        Some(ref state) => {
            let status_str = match state.status {
                AttackStatus::Windup { target, .. } => {
                    format!("前摇中（准备攻击目标 {:?}）", target)
                }
                AttackStatus::Cooldown { .. } => "攻击冷却中".to_owned(),
            };
            format!(
                "正在攻击，状态：{}，当前设置的目标：{:?}",
                status_str, state.target
            )
        }
        None => "空闲，无攻击动作".to_owned(),
    };

    // 移动状态
    let run_target_desc = match myself.run_target {
        Some(pos) => format!(
            "已在移动中，目的地: x={:.0}, y={:.0}",
            pos.x, pos.y
        ),
        None => "静止中，无移动指令".to_owned(),
    };

    // 小兵信息
    let minions_desc = if observe.minions.is_empty() {
        "附近 2000 单位内没有发现敌方小兵。".to_owned()
    } else {
        observe
            .minions
            .iter()
            .enumerate()
            .map(|(i, m)| {
                format!(
                    "小兵[{}]: 距离{:.0} 位置({:.0},{:.0}) 血量{:.0}",
                    i + 1,
                    m.distance,
                    m.position.x,
                    m.position.y,
                    m.health
                )
            })
            .collect::<Vec<String>>()
            .join(" | ")
    };

    // 敌方英雄信息
    let hero_desc = match observe.enemy_hero {
        Some(ref h) => format!(
            "位置({:.0},{:.0}) 血量{:.0}/{:.0} ({:.0}%)",
            h.position.x, h.position.y, h.health, h.max_health,
            h.health / h.max_health * 100.0
        ),
        None => "视野内没有发现敌方英雄".to_owned(),
    };

    // 技能状态
    let skills_desc = if myself.skills.is_empty() {
        "无技能数据".to_owned()
    } else {
        let slot_names = ["Q", "W", "E", "R"];
        myself.skills.iter().map(|s| {
            let name = slot_names.get(s.index).unwrap_or(&"?");
            if s.level == 0 {
                format!("{}(未学)", name)
            } else {
                match s.cooldown_remaining {
                    Some(cd) => format!("{}(Lv{} CD{:.1}s)", name, s.level, cd),
                    None => format!("{}(Lv{} 可用)", name, s.level),
                }
            }
        }).collect::<Vec<_>>().join(" | ")
    };

    // 蓝量/能量
    let resource_desc = match myself.ability_resource {
        Some((value, max)) => format!("{:.0}/{:.0}", value, max),
        None => "无".to_owned(),
    };

    format!(
        "仿真时间: {:.1}s\n\
         === 自身状态 ===\n\
         位置: ({:.0}, {:.0}) | 等级: {} | 生命值: {:.0}/{:.0} ({:.0}%) | 蓝量: {}\n\
         攻击力: {:.0} | 攻击范围: {:.0} | 攻击速度: {:.2} | 护甲: {:.0}\n\
         技能点: {} | 技能: {}\n\
         移动: {} | 普攻: {}\n\
         === 敌方小兵 (2000单位内) ===\n\
         {}\n\
         === 敌方英雄 ===\n\
         {}\n\n\
         请调用一个工具执行动作。",
        observe.time,
        myself.position.x, myself.position.y,
        myself.level,
        myself.health, myself.max_health,
        if myself.max_health > 0.0 { myself.health / myself.max_health * 100.0 } else { 0.0 },
        resource_desc,
        myself.attack_damage,
        myself.attack_range,
        myself.attack_speed,
        myself.armor,
        myself.skill_points,
        skills_desc,
        run_target_desc,
        attack_state_desc,
        minions_desc,
        hero_desc,
    )
}
