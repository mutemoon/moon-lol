use bevy::prelude::*;

#[derive(EntityEvent)]
pub struct CommandSkinParticleSpawn {
    pub entity: Entity,
    pub hash: String,
    /// 可选的发射器世界朝向覆盖（如有方向的受击粒子朝向破绽方向）；
    /// None 时沿用锚点实体自身朝向
    pub rotation: Option<Quat>,
    /// 可选的 resolver 查表实体：因为“攻击者携带、播在目标身上”的粒子
    /// （如 Hit_Tar / R Mark）只存在于攻击者皮肤的 resolver 中，所以查表
    /// 实体与挂载实体需要分离；None 时用挂载实体自身的 resolver
    pub resolver_entity: Option<Entity>,
}

#[derive(EntityEvent)]
pub struct CommandSkinParticleDespawn {
    pub entity: Entity,
    pub hash: String,
    /// 同 CommandSkinParticleSpawn.resolver_entity：撤销时需用同一个
    /// resolver 才能解析出同一个 vfx hash
    pub resolver_entity: Option<Entity>,
}

#[derive(EntityEvent)]
pub struct CommandAnimationPlay {
    pub entity: Entity,
    pub hash: String,
    pub repeat: bool,
    pub duration: Option<f32>,
}

/// 皮肤音效播放命令：`key` 为 `ConfigAudio.spells` 的技能名（如 "FioraEAttack"），
/// `hit=true` 播 `on_hit`（命中时刻），`hit=false` 播 `on_cast`（施放/触发时刻）。
/// 用于技能命中、被动触发等没有独立技能施放事件（EventSkillCast）的音效时机。
#[derive(EntityEvent)]
pub struct CommandSkinSoundPlay {
    pub entity: Entity,
    pub key: String,
    pub hit: bool,
}
