use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::{LOLEnumData, LOLEnumUiPosition, LOLUiElementGroupButtonData, LOLUiElementIconData};

/// UI 元素组件
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Node { position_type: PositionType::Absolute, overflow: Overflow::hidden(), ..default() }, Pickable::IGNORE)]
pub enum UIElement {
    Handle(Handle<LOLUiElementIconData>),
    Data {
        position: LOLEnumUiPosition,
        texture_data: Option<LOLEnumData>,
    },
}

/// UI 元素组件
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
#[require(Node::default(), Pickable::IGNORE)]
pub struct UIElementChild;

/// 血条绑定组件（将 UI 血条绑定到目标实体）
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct HealthBind(pub Entity);

/// UI 绑定组件（将 UI 元素绑定到世界实体）
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct UIBind {
    pub entity: Entity,
    pub position: Vec3,
    pub offset: Vec2,
    pub anchor: Vec2,
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interaction)]
pub struct UIButton(pub Handle<LOLUiElementGroupButtonData>);
