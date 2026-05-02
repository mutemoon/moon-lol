pub mod e;
pub mod passive;
pub mod r;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{
    CommandAnimationPlay, CommandSkinParticleDespawn, CommandSkinParticleSpawn,
};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::attack::{BuffAttack, CommandAttackReset};
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::fiora::e::BuffFioraE;
use crate::fiora::r::BuffFioraR;

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<passive::FioraVitalLastDirection>();
        app.add_systems(
            FixedUpdate,
            (
                passive::update_add_vital,
                passive::update_remove_vital,
                r::fixed_update,
            ),
        );
        app.add_observer(on_fiora_skill_cast);
        app.add_observer(passive::on_passive_damage_create);
        app.add_observer(e::on_event_attack_end);
        app.add_observer(r::on_r_damage_create);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn on_fiora_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_transform: Query<&Transform>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_fiora_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill.spell.clone(),
        ),
        SkillSlot::W => cast_fiora_w(&mut commands, entity),
        SkillSlot::E => cast_fiora_e(&mut commands, entity),
        SkillSlot::R => cast_fiora_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_fiora_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Fiora_Q_Dash_Trail_ground"),
    });
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell.clone(),
        move_type: DashMoveType::Pointer { max: 300.0 },
        damage: None,
        speed: 1000.0,
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 300.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("Fiora_Q_Slash_Cas")),
        }],
    });
}

fn cast_fiora_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Fiora_W_Telegraph_Blue"),
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2_in".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Fiora_W_Cas"),
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleDespawn {
        entity,
        hash: hash_bin("Fiora_W_Telegraph_Blue"),
    });
}

fn cast_fiora_e(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert((BuffAttack {
        bonus_attack_speed: 0.5,
    },));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraE::default());
    commands.trigger(CommandAttackReset { entity });
}

fn cast_fiora_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Fiora_R_Indicator_Ring"),
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Fiora_R_ALL_Warning"),
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraR::default());
}
