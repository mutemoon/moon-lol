use lol_core::action::Action;
use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::GLOBAL_ACTION;

#[derive(serde::Deserialize)]
pub struct SkillLevelUpArgs {
    pub index: usize,
}

pub struct SkillLevelUpTool;

impl Tool for SkillLevelUpTool {
    const NAME: &'static str = "skill_level_up";
    type Error = std::convert::Infallible;
    type Args = SkillLevelUpArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "skill_level_up".to_string(),
            description: "使用技能点升级指定技能。只有拥有技能点时才能使用。".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "description": "技能索引：0=Q, 1=W, 2=E, 3=R" }
                },
                "required": ["index"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut act = GLOBAL_ACTION.lock().unwrap();
        *act = Some(Action::SkillLevelUp(args.index));
        Ok(format!("成功执行升级第 {} 号技能指令", args.index))
    }
}
