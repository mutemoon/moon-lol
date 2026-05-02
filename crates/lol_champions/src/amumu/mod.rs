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

use crate::amumu::buffs::{BuffAmumuPassive, BuffAmumuR};

#[derive(Default)]
pub struct PluginAmumu;

impl Plugin for PluginAmumu {
    fn build(&self, app: &mut App) {
        app.add_observer(on_amumu_skill_cast);
        app.add_observer(on_amumu_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Amumu"))]
#[reflect(Component)]
pub struct Amumu;

fn on_amumu_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_amumu.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_amumu_q(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::W => cast_amumu_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_amumu_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_amumu_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_amumu_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Amumu_Q_Cast"),
    });

    // Q is a targeted dash that stuns
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 1100.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 1000.0,
    });
}

fn cast_amumu_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Amumu_W_Cast"),
    });

    // W is toggle damage around self
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Amumu_W_Hit")),
        }],
    });
}

fn cast_amumu_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Amumu_E_Cast"),
    });

    // E is area damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 350.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Amumu_E_Hit")),
        }],
    });
}

fn cast_amumu_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Amumu_R_Cast"),
    });

    // R is area stun
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 550.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Amumu_R_Hit")),
        }],
    });

    // Stun all enemies in range
    commands
        .entity(entity)
        .with_related::<BuffOf>(DebuffStun::new(1.5));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAmumuR::new());
}

fn on_amumu_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_amumu: Query<(), With<Amumu>>,
) {
    let source = trigger.source;
    if q_amumu.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns target
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(1.0));
    // Apply passive - Cursed Touch
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffAmumuPassive::new());
}
