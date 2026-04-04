use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::buffs::sivir_buffs::BuffSivirW;

const SIVIR_Q_KEY: &str = "Characters/Sivir/Spells/SivirQ/SivirQ";
#[allow(dead_code)]
const SIVIR_W_KEY: &str = "Characters/Sivir/Spells/SivirW/SivirW";
#[allow(dead_code)]
const SIVIR_E_KEY: &str = "Characters/Sivir/Spells/SivirE/SivirE";
#[allow(dead_code)]
const SIVIR_R_KEY: &str = "Characters/Sivir/Spells/SivirR/SivirR";

#[derive(Default)]
pub struct PluginSivir;

impl Plugin for PluginSivir {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_sivir_skill_cast);
        app.add_observer(on_sivir_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sivir"))]
#[reflect(Component)]
pub struct Sivir;

fn on_sivir_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_sivir.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_sivir_q(&mut commands, entity),
        SkillSlot::W => cast_sivir_w(&mut commands, entity),
        SkillSlot::E => cast_sivir_e(&mut commands, entity),
        SkillSlot::R => cast_sivir_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_sivir_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_Q_Cast"));

    // Q is boomerang blade - damage
    skill_damage(
        commands,
        entity,
        SIVIR_Q_KEY,
        DamageShape::Sector {
            radius: 850.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Sivir_Q_Hit")),
    );
}

fn cast_sivir_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_W_Cast"));

    // W is ricochet - attackspeed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}

fn cast_sivir_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_E_Cast"));

    // E is spell shield - magic shield
}

fn cast_sivir_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Sivir_R_Cast"));

    // R is on the hunt - movespeed buff
}

fn on_sivir_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sivir: Query<(), With<Sivir>>,
) {
    let source = trigger.source;
    if q_sivir.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // W gives attackspeed to caster
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffSivirW::new(0.5, 5.0));
}

fn add_skills(
    mut commands: Commands,
    q_sivir: Query<Entity, (With<Sivir>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_sivir.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Sivir/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Sivir/Spells/SivirPassive/SivirPassive",
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
