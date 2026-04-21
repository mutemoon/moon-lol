use serde::{Deserialize, Serialize};

/// 移动类型
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum MovementType {
    MovementTypeFixedSpeed(MovementTypeFixedSpeed),
    // 其他类型暂不实现
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MovementTypeFixedSpeed {
    pub speed: Option<f32>,
    pub start_bone_name: Option<String>,
}

/// 导弹规格
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub movement_component: MovementType,
}
