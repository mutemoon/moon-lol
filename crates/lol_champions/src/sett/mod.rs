pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter};
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginSett;

impl Plugin for PluginSett {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sett_q);
        app.add_observer(on_sett_w);
        app.add_observer(on_sett_e);
        app.add_observer(on_sett_r);
        app.add_observer(on_sett_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sett"))]
#[reflect(Component)]
pub struct Sett;

fn on_sett_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }

    cast_sett_q(&mut commands, entity);
}

fn on_sett_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }

    cast_sett_w(&mut commands, entity, skill.spell.clone());
}

fn on_sett_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }

    cast_sett_e(&mut commands, entity, skill.spell.clone());
}

fn on_sett_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_sett.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    cast_sett_r(
        &mut commands,
        &q_transform,
        entity,
        trigger.point,
        skill.spell.clone(),
    );
}

fn cast_sett_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    // Q 组合：强化下 2 次普攻 + 额外伤害 + 移速
    commands.trigger(CommandAttackReset { entity });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(2, 4.0))
        .with_related::<BuffOf>(BuffOnHitBonusDamage {
            flat: 0.0, // TODO: 实际值来自 get_skill_value
            ratio: 0.0,
        })
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.3, 4.0));
}

fn cast_sett_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
    // W deals true damage in a cone and grants shield based on damage taken
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 350.0,
                angle: 75.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::True,
            }],
        }],
    });
    // Grit converts to shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

fn cast_sett_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });
    // E is a两边拉扯 that damages and stuns enemies caught by both sides
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 300.0,
                angle: 90.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_sett_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });
    // R is a dash that carries enemy to target location and deals damage
    commands.entity(entity).insert(DashDamageIntent {
        damage: DashDamage {
            radius_end: 200.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            },
        },
        skill: skill_spell,
    });
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer { max: 400.0 },
        speed: 700.0,
    });
}

/// 监听 Sett 造成的伤害，E/R 命中时眩晕
fn on_sett_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sett: Query<(), With<Sett>>,
) {
    let source = trigger.source;
    if q_sett.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // E/R 命中时眩晕
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}
