pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffFear;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::nocturne::buffs::BuffNocturneW;

const NOCTURNE_Q_KEY: &str = "Characters/Nocturne/Spells/NocturneQ/NocturneQ";
#[allow(dead_code)]
const NOCTURNE_W_KEY: &str = "Characters/Nocturne/Spells/NocturneW/NocturneW";
const NOCTURNE_E_KEY: &str = "Characters/Nocturne/Spells/NocturneE/NocturneE";
const NOCTURNE_R_KEY: &str = "Characters/Nocturne/Spells/NocturneR/NocturneR";

#[derive(Default)]
pub struct PluginNocturne;

impl Plugin for PluginNocturne {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
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

    match skill.slot {
        SkillSlot::Q => cast_nocturne_q(&mut commands, entity),
        SkillSlot::W => cast_nocturne_w(&mut commands, entity),
        SkillSlot::E => cast_nocturne_e(&mut commands, entity),
        SkillSlot::R => cast_nocturne_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nocturne_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Nocturne_Q_Cast"));

    // Q is a throwing blade that leaves a trail
    skill_damage(
        commands,
        entity,
        NOCTURNE_Q_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Nocturne_Q_Hit")),
    );
}

fn cast_nocturne_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Nocturne_W_Cast"));

    // W grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNocturneW::new(0.5, 5.0));
}

fn cast_nocturne_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Nocturne_E_Cast"));

    // E is a fear after delay
    skill_damage(
        commands,
        entity,
        NOCTURNE_E_KEY,
        DamageShape::Circle { radius: 425.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nocturne_E_Hit")),
    );
}

fn cast_nocturne_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Nocturne_R_Cast"));

    // R is a global fear
    skill_damage(
        commands,
        entity,
        NOCTURNE_R_KEY,
        DamageShape::Circle { radius: 2500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Nocturne_R_Hit")),
    );
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

fn add_skills(
    mut commands: Commands,
    q_nocturne: Query<Entity, (With<Nocturne>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_nocturne.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Nocturne/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Nocturne/Spells/NocturnePassive/NocturnePassive",
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
