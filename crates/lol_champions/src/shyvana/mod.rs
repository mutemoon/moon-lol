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

use crate::shyvana::buffs::BuffShyvanaE;

#[derive(Default)]
pub struct PluginShyvana;

impl Plugin for PluginShyvana {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shyvana_skill_cast);
        app.add_observer(on_shyvana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shyvana"))]
#[reflect(Component)]
pub struct Shyvana;

fn on_shyvana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_shyvana_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_shyvana_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_shyvana_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_shyvana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shyvana_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shyvana_Q_Cast"),
    });

    // Q is twin bite - damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 250.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Shyvana_Q_Hit")),
        }],
    });
}

fn cast_shyvana_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shyvana_W_Cast"),
    });

    // W is flame breath - damage over time
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 600.0,
                angle: 25.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Shyvana_W_Hit")),
        }],
    });
}

fn cast_shyvana_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shyvana_E_Cast"),
    });

    // E is dragon descent - knockup
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 450.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Shyvana_E_Hit")),
        }],
    });
}

fn cast_shyvana_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Shyvana_R_Cast"),
    });

    // R is shape shift - transformation
}

fn on_shyvana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
) {
    let source = trigger.source;
    if q_shyvana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShyvanaE::new(0.5, 1.0));
}
