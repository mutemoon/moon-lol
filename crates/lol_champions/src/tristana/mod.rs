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

use crate::tristana::buffs::BuffTristanaW;

#[derive(Default)]
pub struct PluginTristana;

impl Plugin for PluginTristana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tristana_skill_cast);
        app.add_observer(on_tristana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Tristana"))]
#[reflect(Component)]
pub struct Tristana;

fn on_tristana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_tristana_q(&mut commands, entity),
        SkillSlot::W => cast_tristana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_tristana_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_tristana_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_tristana_q(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Tristana_Q_Cast"),
    });
}

fn cast_tristana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Tristana_W_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Tristana_W_Hit")),
        }],
    });
}

fn cast_tristana_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Tristana_E_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 700.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Tristana_E_Hit")),
        }],
    });
}

fn cast_tristana_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Tristana_R_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 700.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Tristana_R_Hit")),
        }],
    });
}

fn on_tristana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
) {
    let source = trigger.source;
    if q_tristana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTristanaW::new(0.5, 2.0));
}
