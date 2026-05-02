pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

use crate::anivia::buffs::BuffAniviaR;

const ANIVIA_Q_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginAnivia;

impl Plugin for PluginAnivia {
    fn build(&self, app: &mut App) {
        app.add_observer(on_anivia_skill_cast);
        app.add_observer(on_anivia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Anivia"))]
#[reflect(Component)]
pub struct Anivia;

fn on_anivia_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_anivia_q(
            &mut commands,
            entity,
            skill_spell,
            trigger.skill_entity,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_anivia_w(&mut commands, entity),
        SkillSlot::E => cast_anivia_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_anivia_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_anivia_q(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: launch the crystal
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: hash_bin("Anivia_Q_Cast"),
        });
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, ANIVIA_Q_RECAST_WINDOW));
    } else {
        // Second cast: detonate for extra damage and stun
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: hash_bin("Anivia_Q_Explode"),
        });
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Circle { radius: 150.0 },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Magic,
                }],
                particle: Some(hash_bin("Anivia_Q_Hit")),
            }],
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });
    }
}

fn cast_anivia_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Anivia_W_Cast"),
    });
    // W creates a wall that blocks movement
}

fn cast_anivia_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Anivia_E_Cast"),
    });

    // E deals extra damage to frozen targets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Anivia_E_Hit")),
        }],
    });
}

fn cast_anivia_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Anivia_R_Cast"),
    });

    // R is a continuous storm
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 750.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Anivia_R_Hit")),
        }],
    });

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAniviaR::new());
}

fn on_anivia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
) {
    let source = trigger.source;
    if q_anivia.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q and R slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.2, 2.0));
}
