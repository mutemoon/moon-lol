use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::cc_debuffs::DebuffSlow;
use crate::buffs::draven_buffs::BuffDravenPassive;
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

#[allow(dead_code)]
const DRAVEN_Q_KEY: &str = "Characters/Draven/Spells/DravenQ/DravenQ";
#[allow(dead_code)]
const DRAVEN_W_KEY: &str = "Characters/Draven/Spells/DravenW/DravenW";
const DRAVEN_E_KEY: &str = "Characters/Draven/Spells/DravenE/DravenE";
const DRAVEN_R_KEY: &str = "Characters/Draven/Spells/DravenR/DravenR";

#[derive(Default)]
pub struct PluginDraven;

impl Plugin for PluginDraven {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_draven_skill_cast);
        app.add_observer(on_draven_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Draven"))]
#[reflect(Component)]
pub struct Draven;

fn on_draven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_draven.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_draven_q(&mut commands, entity),
        SkillSlot::W => cast_draven_w(&mut commands, entity),
        SkillSlot::E => cast_draven_e(&mut commands, entity),
        SkillSlot::R => cast_draven_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_draven_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_Q_Cast"));

    // Q enhances next attack - handled by buff system
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffDravenPassive::new());
}

fn cast_draven_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_W_Cast"));
    // W is movement speed buff - handled by buff system
}

fn cast_draven_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_E_Cast"));

    // E is a knockback skillshot
    skill_damage(
        commands,
        entity,
        DRAVEN_E_KEY,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Draven_E_Hit")),
    );
}

fn cast_draven_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Draven_R_Cast"));

    // R is global damage
    skill_damage(
        commands,
        entity,
        DRAVEN_R_KEY,
        DamageShape::Circle { radius: 20000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Draven_R_Hit")),
    );
}

fn on_draven_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_draven: Query<(), With<Draven>>,
) {
    let source = trigger.source;
    if q_draven.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.35, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_draven: Query<Entity, (With<Draven>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_draven.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Draven/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Draven/Spells/DravenPassive/DravenPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
