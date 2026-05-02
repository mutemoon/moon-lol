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
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::leblanc::buffs::{BuffLeBlancE, BuffLeBlancQ};

#[derive(Default)]
pub struct PluginLeBlanc;

impl Plugin for PluginLeBlanc {
    fn build(&self, app: &mut App) {
        app.add_observer(on_leblanc_skill_cast);
        app.add_observer(on_leblanc_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("LeBlanc"))]
#[reflect(Component)]
pub struct LeBlanc;

fn on_leblanc_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leblanc: Query<(), With<LeBlanc>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_leblanc.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_leblanc_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_leblanc_w(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        SkillSlot::E => cast_leblanc_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_leblanc_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_leblanc_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("LeBlanc_Q_Cast"),
    });

    // Q marks enemy
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 700.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("LeBlanc_Q_Hit")),
        }],
    });
}

fn cast_leblanc_w(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("LeBlanc_W_Cast"),
    });

    // W is a dash with damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 100.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("LeBlanc_W_Hit")),
        }],
    });
}

fn cast_leblanc_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("LeBlanc_E_Cast"),
    });

    // E chains enemy
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 950.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("LeBlanc_E_Hit")),
        }],
    });
}

fn cast_leblanc_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
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
        hash: hash_bin("LeBlanc_R_Cast"),
    });

    // R mimics the last used skill
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 700.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("LeBlanc_R_Hit")),
        }],
    });
}

fn on_leblanc_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_leblanc: Query<(), With<LeBlanc>>,
) {
    let source = trigger.source;
    if q_leblanc.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies mark
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLeBlancQ::new(100.0, 3.5));

    // E roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLeBlancE::new(80.0, 1.5, 3.0));
}
