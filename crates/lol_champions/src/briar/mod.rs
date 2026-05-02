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

use crate::briar::buffs::{BuffBriarPassive, BuffBriarQ, BuffBriarW};

#[derive(Default)]
pub struct PluginBriar;

impl Plugin for PluginBriar {
    fn build(&self, app: &mut App) {
        app.add_observer(on_briar_skill_cast);
        app.add_observer(on_briar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Briar"))]
#[reflect(Component)]
pub struct Briar;

fn on_briar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_briar_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_briar_w(&mut commands, entity),
        SkillSlot::E => cast_briar_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_briar_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_briar_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Briar_Q_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 475.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Briar_Q_Hit")),
        }],
    });
}

fn cast_briar_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Briar_W_Cast"),
    });
}

fn cast_briar_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Briar_E_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 500.0,
                angle: 30.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Briar_E_Hit")),
        }],
    });
}

fn cast_briar_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Briar_R_Cast"),
    });

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 12000.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Briar_R_Hit")),
        }],
    });
}

fn on_briar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
) {
    let source = trigger.source;
    if q_briar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarPassive::new(1, 10.0, 6.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarQ::new(0.85, 15.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarW::new(0.75, 0.4, 4.0));
}
