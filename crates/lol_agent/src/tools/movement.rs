use bevy::prelude::*;
use lol_core::action::Action;
use rig::completion::ToolDefinition;
use rig::tool::Tool;

use super::GLOBAL_ACTION;

#[derive(serde::Deserialize)]
pub struct MoveArgs {
    pub x: f32,
    pub y: f32,
}

pub struct MoveTool;

impl Tool for MoveTool {
    const NAME: &'static str = "move_to";
    type Error = std::convert::Infallible;
    type Args = MoveArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "move_to".to_string(),
            description: "移动到指定的地图坐标 (x, y)".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "x": { "type": "number", "description": "目标位置的 X 轴坐标" },
                    "y": { "type": "number", "description": "目标位置的 Y 轴坐标" }
                },
                "required": ["x", "y"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut act = GLOBAL_ACTION.lock().unwrap();
        *act = Some(Action::Move(Vec2::new(args.x, args.y)));
        Ok("成功执行移动指令".to_string())
    }
}

#[derive(serde::Deserialize)]
pub struct StopArgs {}

pub struct StopTool;

impl Tool for StopTool {
    const NAME: &'static str = "stop";
    type Error = std::convert::Infallible;
    type Args = StopArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "stop".to_string(),
            description: "原地静止，取消当前的所有移动和攻击动作".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let mut act = GLOBAL_ACTION.lock().unwrap();
        *act = Some(Action::Stop);
        Ok("成功执行静止指令".to_string())
    }
}
