pub mod buffs;

use bevy::prelude::{Handle, *};
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffFear;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::nocturne::buffs::BuffNocturneW;

#[derive(Default)]
pub struct PluginNocturne;

impl Plugin for PluginNocturne {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nocturne_skill_cast);
        app.add_observer(on_nocturne_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nocturne"))]
#[reflect(Component)]
pub struct Nocturne;

fn on_nocturne_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nocturne.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_nocturne_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_nocturne_w(&mut commands, entity),
        SkillSlot::E => cast_nocturne_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_nocturne_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_nocturne_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nocturne_Q_Cast"),
    });

    // Q is a throwing blade that leaves a trail
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 15.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Nocturne_Q_Hit")),
        }],
    });
}

fn cast_nocturne_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nocturne_W_Cast"),
    });

    // W grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNocturneW::new(0.5, 5.0));
}

fn cast_nocturne_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nocturne_E_Cast"),
    });

    // E is a fear after delay
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 425.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nocturne_E_Hit")),
        }],
    });
}

fn cast_nocturne_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nocturne_R_Cast"),
    });

    // R is a global fear
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 2500.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Nocturne_R_Hit")),
        }],
    });
}

fn on_nocturne_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nocturne: Query<(), With<Nocturne>>,
) {
    let source = trigger.source;
    if q_nocturne.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E fears
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffFear::new(2.0));
}
