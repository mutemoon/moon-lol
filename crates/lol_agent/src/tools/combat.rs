use bevy::prelude::*;
use lol_core::action::Action;
use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::{GLOBAL_ACTION, GLOBAL_MINION_ENTITIES, GLOBAL_HERO_ENTITY};

#[derive(serde::Deserialize)]
pub struct AttackMinionArgs {
    #[serde(default)]
    pub index: Option<usize>,
}

pub struct AttackMinionTool;

impl Tool for AttackMinionTool {
    const NAME: &'static str = "attack_minion";
    type Error = std::convert::Infallible;
    type Args = AttackMinionArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "attack_minion".to_string(),
            description: "对指定索引的敌方小兵发起普通攻击。index 对应小兵列表中的序号（从1开始）。不传 index 或传 0 则攻击距离最近的小兵。用于补刀时应选择生命值低于你攻击力的小兵。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "description": "要攻击的小兵序号（从1开始，对应小兵列表编号）。不传则攻击最近的。" }
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let entities = {
            let lock = GLOBAL_MINION_ENTITIES.lock().unwrap();
            lock.clone()
        };

        let idx = args.index.unwrap_or(1).saturating_sub(1);
        let target_entity = entities.get(idx).copied();

        let mut act = GLOBAL_ACTION.lock().unwrap();
        let Some(target) = target_entity else {
            *act = Some(Action::Stop);
            return Ok("指定的小兵不存在，已自动转化为原地静止".to_string());
        };

        *act = Some(Action::Attack(target));
        Ok(format!("成功执行攻击小兵[{}]指令", idx + 1))
    }
}

#[derive(serde::Deserialize)]
pub struct AttackArgs {}

pub struct AttackHeroTool;

impl Tool for AttackHeroTool {
    const NAME: &'static str = "attack_hero";
    type Error = std::convert::Infallible;
    type Args = AttackArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "attack_hero".to_string(),
            description: "对视野内对线的敌方英雄发起普通攻击（用于消耗、走打对拼）".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let target_entity = {
            let lock = GLOBAL_HERO_ENTITY.lock().unwrap();
            *lock
        };
        let mut act = GLOBAL_ACTION.lock().unwrap();
        let Some(target) = target_entity else {
            *act = Some(Action::Stop);
            return Ok("视野内没有敌方英雄，已自动转化为原地静止".to_string());
        };

        *act = Some(Action::Attack(target));
        Ok("成功执行攻击敌方英雄指令".to_string())
    }
}

#[derive(serde::Deserialize)]
pub struct SkillArgs {
    pub index: usize,
    pub x: f32,
    pub y: f32,
}

pub struct SkillTool;

impl Tool for SkillTool {
    const NAME: &'static str = "cast_skill";
    type Error = std::convert::Infallible;
    type Args = SkillArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "cast_skill".to_string(),
            description: "施放指定索引的技能，朝向目标地图坐标 (x, y)。只有技能等级 >= 1 且不在冷却中时才能施放。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "description": "技能索引：0=Q, 1=W, 2=E, 3=R" },
                    "x": { "type": "number", "description": "技能指向位置的 X 轴坐标" },
                    "y": { "type": "number", "description": "技能指向位置的 Y 轴坐标" }
                },
                "required": ["index", "x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut act = GLOBAL_ACTION.lock().unwrap();
        *act = Some(Action::Skill {
            index: args.index,
            point: Vec2::new(args.x, args.y),
        });
        Ok(format!("成功执行施放第 {} 号技能指令", args.index))
    }
}
