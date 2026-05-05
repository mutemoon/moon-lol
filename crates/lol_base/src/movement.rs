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
    pub tracks_target: Option<bool>,
    pub project_target_to_cast_range: Option<bool>,
    pub use_height_offset_at_end: Option<bool>,
    pub offset_initial_target_height: Option<f32>,
}

/// 导弹行为
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MissileBehavior {
    CastOnHit,
    DestroyOnMovementComplete,
}

/// 高度求解器
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HeightSolver {
    BlendedLinearHeightSolver,
}

/// 垂直朝向
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum VerticalFacing {
    VerticalFacingFaceTarget,
}

/// 导弹规格
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MissileSpecification {
    pub movement_component: MovementType,
    pub missile_width: Option<f32>,
    pub behaviors: Option<Vec<MissileBehavior>>,
    pub height_solver: Option<HeightSolver>,
    pub vertical_facing: Option<VerticalFacing>,
}
