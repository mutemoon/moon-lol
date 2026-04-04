use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::ashe_buffs::BuffAsheQ;
use crate::buffs::cc_debuffs::DebuffSlow;
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

#[allow(dead_code)]
const ASHE_Q_KEY: &str = "Characters/Ashe/Spells/AsheQ/AsheQ";
const ASHE_W_KEY: &str = "Characters/Ashe/Spells/AsheW/AsheW";
#[allow(dead_code)]
const ASHE_E_KEY: &str = "Characters/Ashe/Spells/AsheE/AsheE";
const ASHE_R_KEY: &str = "Characters/Ashe/Spells/AsheR/AsheR";

#[derive(Default)]
pub struct PluginAshe;

impl Plugin for PluginAshe {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_ashe_skill_cast);
        app.add_observer(on_ashe_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ashe"))]
#[reflect(Component)]
pub struct Ashe;

fn on_ashe_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ashe_q(&mut commands, entity),
        SkillSlot::W => cast_ashe_w(&mut commands, entity),
        SkillSlot::E => cast_ashe_e(&mut commands, entity),
        SkillSlot::R => cast_ashe_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_ashe_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ashe_Q_Cast"));

    // Q grants attack speed buff and fires multiple arrows
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAsheQ::new());
}

fn cast_ashe_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ashe_W_Cast"));

    // W is a cone volley
    skill_damage(
        commands,
        entity,
        ASHE_W_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 40.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ashe_W_Hit")),
    );
}

fn cast_ashe_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ashe_E_Cast"));
    // E is global vision - no damage
}

fn cast_ashe_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ashe_R_Cast"));

    // R is a global arrow that stuns - use large sector to simulate global range
    skill_damage(
        commands,
        entity,
        ASHE_R_KEY,
        DamageShape::Sector {
            radius: 20000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ashe_R_Hit")),
    );
}

fn on_ashe_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
) {
    let source = trigger.source;
    if q_ashe.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply frost slow on all damage
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_ashe: Query<Entity, (With<Ashe>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_ashe.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Ashe/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Ashe/Spells/AshePassive/AshePassive",
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
