pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::blitzcrank::buffs::BuffBlitzcrankW;

#[derive(Default)]
pub struct PluginBlitzcrank;

impl Plugin for PluginBlitzcrank {
    fn build(&self, app: &mut App) {
        app.add_observer(on_blitzcrank_skill_cast);
        app.add_observer(on_blitzcrank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Blitzcrank"))]
#[reflect(Component)]
pub struct Blitzcrank;

fn on_blitzcrank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_blitzcrank.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_blitzcrank_q(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::W => cast_blitzcrank_w(&mut commands, entity),
        SkillSlot::E => cast_blitzcrank_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_blitzcrank_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_blitzcrank_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Blitzcrank_Q_Cast"),
    });

    // Q is a hook that pulls enemy
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 1115.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::Champion,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 900.0,
    });
}

fn cast_blitzcrank_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Blitzcrank_W_Cast"),
    });

    // W grants movement and attack speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffBlitzcrankW::new());
}

fn cast_blitzcrank_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Blitzcrank_E_Cast"),
    });

    // E is an empowered attack that knocks up
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 100.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Blitzcrank_E_Hit")),
        }],
    });
}

fn cast_blitzcrank_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Blitzcrank_R_Cast"),
    });

    // R is an AoE that silences
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 600.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Blitzcrank_R_Hit")),
        }],
    });
}

fn on_blitzcrank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_blitzcrank: Query<(), With<Blitzcrank>>,
) {
    let source = trigger.source;
    if q_blitzcrank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q stuns on hit
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.65));
}
