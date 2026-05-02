pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::taric::buffs::BuffTaricE;

#[derive(Default)]
pub struct PluginTaric;

impl Plugin for PluginTaric {
    fn build(&self, app: &mut App) {
        app.add_observer(on_taric_skill_cast);
        app.add_observer(on_taric_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Taric"))]
#[reflect(Component)]
pub struct Taric;

fn on_taric_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taric: Query<(), With<Taric>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_taric.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_taric_q(&mut commands, entity),
        SkillSlot::W => cast_taric_w(&mut commands, entity),
        SkillSlot::E => cast_taric_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_taric_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_taric_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Taric_Q_Cast"),
    });
}

fn cast_taric_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Taric_W_Cast"),
    });
}

fn cast_taric_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Taric_E_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Taric_E_Hit")),
        }],
    });
}

fn cast_taric_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Taric_R_Cast"),
    });
}

fn on_taric_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_taric: Query<(), With<Taric>>,
) {
    let source = trigger.source;
    if q_taric.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTaricE::new(1.0, 1.5));
}
