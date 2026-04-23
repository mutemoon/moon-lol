use bevy::prelude::*;

/// 皮肤组件 - 包含皮肤缩放等信息
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct Skin {
    pub scale: f32,
}

/// 血条组件
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
pub struct HealthBar {
    pub bar_type: u8,
}

/// 皮肤配置组件 - 指向预构建的皮肤场景
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ConfigSkin {
    pub skin: Handle<DynamicWorld>,
}

/// 角色配置组件 - 逻辑相关
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ConfigCharacterRecord {
    pub character_record: Handle<DynamicWorld>,
}
