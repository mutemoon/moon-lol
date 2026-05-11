use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::ui::{
    HashKey, LOLUiElementEffectAnimationData, LOLUiElementEffectDesaturateData,
    LOLUiElementGroupButtonData, LOLUiElementIconData, LOLUiElementRegionData,
    LOLUiElementTextData,
};

/// UI 元素组件
#[derive(Component, Debug, Clone)]
#[require(Node { position_type: PositionType::Absolute, overflow: Overflow::hidden(), ..default() }, Pickable::IGNORE)]
pub enum UIElement {
    Icon(HashKey<LOLUiElementIconData>),
    Region(HashKey<LOLUiElementRegionData>),
    Animation(HashKey<LOLUiElementEffectAnimationData>),
    Desaturate(HashKey<LOLUiElementEffectDesaturateData>),
    Text(HashKey<LOLUiElementTextData>),
}

/// UI 元素组件
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[require(Node::default(), Pickable::IGNORE)]
pub struct UIElementChild;

/// 血条绑定组件（将 UI 血条绑定到目标实体）
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct HealthBind(pub Entity);

/// UI 绑定关系（将 UI 元素绑定到世界实体）
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[relationship(relationship_target = UIBindTarget)]
pub struct UIBindOf(pub Entity);

/// UI 绑定目标（自动维护，指向所有绑定到此实体的 UI 元素）
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[relationship_target(relationship = UIBindOf, linked_spawn)]
pub struct UIBindTarget(Vec<Entity>);

#[derive(Component, Debug, Clone, Default)]
pub struct UIBindData {
    pub position: Vec3,
    pub anchor: Vec2,
}

/// 按钮组件
#[derive(Component, Debug, Clone)]
#[require(Interaction)]
pub struct UIButton(pub HashKey<LOLUiElementGroupButtonData>);

#[derive(Component, Debug, Clone, Default)]
pub struct UIButtonEntities {
    pub default: Option<Entity>,
    pub hover: Option<Entity>,
    pub clicked: Option<Entity>,
    pub all: Vec<Entity>,
}
