use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 游戏动作。序列化形状与服务端 `lol_core::action::Action` 一致（外部标签枚举），
/// 不引入 Bevy 类型，供 CLI / MCP 共用。
///
/// - `Move([x, y])`            → `{"Move":[x,y]}`
/// - `Attack(entity_id)`       → `{"Attack":entity_id}`
/// - `Stop`                    → `"Stop"`
/// - `Skill{index,point}`      → `{"Skill":{"index":..,"point":[x,y]}}`
/// - `SkillLevelUp(index)`     → `{"SkillLevelUp":index}`
#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub enum Action {
    /// 移动到坐标 [x, y]
    Move([f32; 2]),
    /// 攻击指定实体 ID
    Attack(u64),
    /// 停止所有动作
    Stop,
    /// 释放指定索引的技能到坐标 [x, y]
    Skill { index: usize, point: [f32; 2] },
    /// 升级指定索引的技能
    SkillLevelUp(usize),
}
