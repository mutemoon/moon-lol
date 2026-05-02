pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginJarvan;

impl Plugin for PluginJarvan {
    fn build(&self, app: &mut App) {
        app.add_observer(on_jarvan_skill_cast);
        app.add_observer(on_jarvan_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("JarvanIV"))]
#[reflect(Component)]
pub struct JarvanIV;

fn on_jarvan_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jarvan.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_jarvan_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_jarvan_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_jarvan_e(&mut commands, entity),
        SkillSlot::R => cast_jarvan_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_jarvan_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jarvan_Q_Cast"),
    });

    // Q is a line damage and armor shred
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 785.0,
                angle: 20.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Jarvan_Q_Hit")),
        }],
    });
}

fn cast_jarvan_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jarvan_W_Cast"),
    });

    // W is an AoE slow
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Jarvan_W_Hit")),
        }],
    });
}

fn cast_jarvan_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jarvan_E_Cast"),
    });

    // E grants attack speed aura
}

fn cast_jarvan_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Jarvan_R_Cast"),
    });

    // R is a targeted dash that creates arena
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 650.0 },
        damage: Some(DashDamage {
            radius_end: 150.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            },
        }),
        speed: 800.0,
    });
}

fn on_jarvan_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jarvan: Query<(), With<JarvanIV>>,
) {
    let source = trigger.source;
    if q_jarvan.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
