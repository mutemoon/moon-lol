use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::belveth_buffs::{BuffBelvethPassive, BuffBelvethW};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const BELVETH_Q_KEY: &str = "Characters/Belveth/Spells/BelvethQ/BelvethQ";
const BELVETH_W_KEY: &str = "Characters/Belveth/Spells/BelvethW/BelvethW";
const BELVETH_E_KEY: &str = "Characters/Belveth/Spells/BelvethE/BelvethE";
const BELVETH_R_KEY: &str = "Characters/Belveth/Spells/BelvethR/BelvethR";

#[derive(Default)]
pub struct PluginBelveth;

impl Plugin for PluginBelveth {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_belveth_skill_cast);
        app.add_observer(on_belveth_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Belveth"))]
#[reflect(Component)]
pub struct Belveth;

fn on_belveth_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_belveth.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_belveth_q(&mut commands, entity),
        SkillSlot::W => cast_belveth_w(&mut commands, entity),
        SkillSlot::E => cast_belveth_e(&mut commands, entity),
        SkillSlot::R => cast_belveth_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_belveth_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_Q_Cast"));

    skill_damage(
        commands,
        entity,
        BELVETH_Q_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Belveth_Q_Hit")),
    );
}

fn cast_belveth_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_W_Cast"));

    skill_damage(
        commands,
        entity,
        BELVETH_W_KEY,
        DamageShape::Sector {
            radius: 660.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Belveth_W_Hit")),
    );
}

fn cast_belveth_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_E_Cast"));

    skill_damage(
        commands,
        entity,
        BELVETH_E_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Belveth_E_Hit")),
    );
}

fn cast_belveth_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Belveth_R_Cast"));
}

fn on_belveth_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_belveth: Query<(), With<Belveth>>,
) {
    let source = trigger.source;
    if q_belveth.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethPassive::new(2, 0.1, 5.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBelvethW::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_belveth: Query<Entity, (With<Belveth>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_belveth.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Belveth/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Belveth/Spells/BelvethPassive/BelvethPassive",
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
