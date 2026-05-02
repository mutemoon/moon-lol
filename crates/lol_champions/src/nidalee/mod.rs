pub mod buffs;

use bevy::prelude::{Handle, *};
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

use crate::nidalee::buffs::{BuffNidaleeE, BuffNidaleeQ};

#[derive(Default)]
pub struct PluginNidalee;

impl Plugin for PluginNidalee {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nidalee_skill_cast);
        app.add_observer(on_nidalee_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nidalee"))]
#[reflect(Component)]
pub struct Nidalee;

fn on_nidalee_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nidalee.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_nidalee_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_nidalee_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_nidalee_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_nidalee_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nidalee_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nidalee_Q_Cast"),
    });

    // Q is a spear (human form)
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1500.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nidalee_Q_Hit")),
        }],
    });
}

fn cast_nidalee_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nidalee_W_Cast"),
    });

    // W is a trap (human form) or pounce (cougar form)
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 900.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nidalee_W_Hit")),
        }],
    });
}

fn cast_nidalee_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nidalee_E_Cast"),
    });

    // E is a heal (human form) or swipe (cougar form)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNidaleeE::new(100.0, 0.7, 7.0));

    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 300.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nidalee_E_Hit")),
        }],
    });
}

fn cast_nidalee_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nidalee_R_Cast"),
    });

    // R transforms between human and cougar forms
}

fn on_nidalee_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nidalee: Query<(), With<Nidalee>>,
) {
    let source = trigger.source;
    if q_nidalee.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks target
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNidaleeQ::new(100.0, 4.0));
}
