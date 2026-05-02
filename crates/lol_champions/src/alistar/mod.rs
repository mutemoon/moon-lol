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
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::alistar::buffs::BuffAlistarR;

#[derive(Default)]
pub struct PluginAlistar;

impl Plugin for PluginAlistar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_alistar_skill_cast);
        app.add_observer(on_alistar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Alistar"))]
#[reflect(Component)]
pub struct Alistar;

fn on_alistar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_alistar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_alistar_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_alistar_w(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::E => cast_alistar_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_alistar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_alistar_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Alistar_Q_Cast"),
    });

    // Q is a knockup and stun in area
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 375.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Alistar_Q_Hit")),
        }],
    });

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
}

fn cast_alistar_w(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Alistar_W_Cast"),
    });

    // W is a dash that knocks back target
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 650.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 800.0,
    });
}

fn cast_alistar_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Alistar_E_Cast"),
    });

    // E is area damage that stuns on 5th hit
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Alistar_E_Hit")),
        }],
    });
}

fn cast_alistar_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Alistar_R_Cast"),
    });

    // R grants damage reduction
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAlistarR::new());
}

fn on_alistar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_alistar: Query<(), With<Alistar>>,
) {
    let source = trigger.source;
    if q_alistar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W stuns and knocks back
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.75));
}
