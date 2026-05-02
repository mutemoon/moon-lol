pub mod buffs;

use bevy::asset::Handle;
use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::CommandAttackReset;
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::lucian::buffs::{BuffLucianPassive, BuffLucianW};

#[derive(Default)]
pub struct PluginLucian;

impl Plugin for PluginLucian {
    fn build(&self, app: &mut App) {
        app.add_observer(on_lucian_skill_cast);
        app.add_observer(on_lucian_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lucian"))]
#[reflect(Component)]
pub struct Lucian;

fn on_lucian_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_lucian_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_lucian_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_lucian_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::R => cast_lucian_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_lucian_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Lucian_Q_Cast"),
    });

    // Q is a piercing light beam
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1000.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Lucian_Q_Hit")),
        }],
    });
}

fn cast_lucian_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Lucian_W_Cast"),
    });

    // W marks enemies and grants movespeed to Lucian
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLucianW::new(60.0, 6.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 900.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Lucian_W_Hit")),
        }],
    });
}

fn cast_lucian_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Lucian_E_Cast"),
    });

    // E is a dash
    commands.trigger(CommandAttackReset { entity });

    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 425.0 },
        damage: None,
        speed: 1000.0,
    });
}

fn cast_lucian_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Lucian_R_Cast"),
    });

    // R is a barrage of shots
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Lucian_R_Hit")),
        }],
    });
}

fn on_lucian_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
) {
    let source = trigger.source;
    if q_lucian.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive procs after abilities
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffLucianPassive::new(50.0, 1.0));
}
