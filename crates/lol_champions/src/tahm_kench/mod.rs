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

use crate::tahm_kench::buffs::BuffTahmKenchE;

#[derive(Default)]
pub struct PluginTahmKench;

impl Plugin for PluginTahmKench {
    fn build(&self, app: &mut App) {
        app.add_observer(on_tahm_kench_skill_cast);
        app.add_observer(on_tahm_kench_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("TahmKench"))]
#[reflect(Component)]
pub struct TahmKench;

fn on_tahm_kench_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tahm_kench: Query<(), With<TahmKench>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tahm_kench.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_tahm_kench_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_tahm_kench_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_tahm_kench_e(&mut commands, entity),
        SkillSlot::R => cast_tahm_kench_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_tahm_kench_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TahmKench_Q_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 900.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("TahmKench_Q_Hit")),
        }],
    });
}

fn cast_tahm_kench_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TahmKench_W_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("TahmKench_W_Hit")),
        }],
    });
}

fn cast_tahm_kench_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TahmKench_E_Cast"),
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffTahmKenchE::new(100.0, 2.0));
}

fn cast_tahm_kench_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("TahmKench_R_Cast"),
    });
}

fn on_tahm_kench_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tahm_kench: Query<(), With<TahmKench>>,
) {
    let source = trigger.source;
    if q_tahm_kench.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTahmKenchE::new(100.0, 2.0));
}
