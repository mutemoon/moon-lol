use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::buffs::common_buffs::BuffCastBlock;
use lol_core::damage::Damage;
use lol_core::life::Health;
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot, get_skill_value};
use lol_core::team::Team;

use crate::riven::Riven;

const RIVEN_W_STUN_DURATION: f32 = 0.75;
const RIVEN_W_STUN_RADIUS: f32 = 250.0;
const RIVEN_W_FIELD_DURATION: f32 = 0.25;
/// W 施法期间阻塞持续时间：8帧 at 30fps ≈ 0.2667s
pub const RIVEN_W_CAST_BLOCK_DURATION: f32 = 8.0 / 30.0;

pub fn on_riven_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_targets: Query<(Entity, &Team, &Transform, &Health)>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let w_damage = get_skill_value(spell_obj, "total_damage", skill.level, |stat| {
        if stat == 2 { damage_value } else { 0.0 }
    })
    .unwrap_or(150.0);

    // 动画 + 伤害场 + 自阻塞
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    commands.trigger(CommandAttachedFieldCreate {
        entity,
        radius: RIVEN_W_STUN_RADIUS,
        damage: w_damage,
        duration: RIVEN_W_FIELD_DURATION,
        grow_from: None,
        grow_duration: None,
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCastBlock::new(RIVEN_W_CAST_BLOCK_DURATION));

    // 对范围内敌人施加眩晕
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(team) = q_team.get(entity) else {
        return;
    };
    use lol_core::action::damage::{DamageShape, is_in_shape};
    let shape = DamageShape::Circle {
        radius: RIVEN_W_STUN_RADIUS,
    };
    let origin = transform.translation;
    let forward = Vec2::ZERO;

    for (target, target_team, target_transform, _) in q_targets.iter() {
        if target_team == team {
            continue;
        }
        if !is_in_shape(target_transform.translation, origin, forward, &shape) {
            continue;
        }
        commands
            .entity(target)
            .with_related::<BuffOf>(DebuffStun::new(RIVEN_W_STUN_DURATION));
    }
}
