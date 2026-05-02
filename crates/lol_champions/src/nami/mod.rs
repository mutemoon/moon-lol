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

use crate::nami::buffs::{BuffNamiE, BuffNamiQ};

#[derive(Default)]
pub struct PluginNami;

impl Plugin for PluginNami {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nami_skill_cast);
        app.add_observer(on_nami_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nami"))]
#[reflect(Component)]
pub struct Nami;

fn on_nami_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nami.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_nami_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_nami_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_nami_e(&mut commands, entity),
        SkillSlot::R => cast_nami_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_nami_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nami_Q_Cast"),
    });

    // Q is a bubble that roots
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 850.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nami_Q_Hit")),
        }],
    });
}

fn cast_nami_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nami_W_Cast"),
    });

    // W bounces between allies and enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 725.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nami_W_Hit")),
        }],
    });
}

fn cast_nami_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nami_E_Cast"),
    });

    // E buffs allied attacks with bonus damage and slow
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNamiE::new(30.0, 0.3, 6.0));
}

fn cast_nami_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Nami_R_Cast"),
    });

    // R is a giant wave that knocks up
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 2750.0,
                angle: 45.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Nami_R_Hit")),
        }],
    });
}

fn on_nami_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nami: Query<(), With<Nami>>,
) {
    let source = trigger.source;
    if q_nami.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNamiQ::new(1.5, 1.5));
}
