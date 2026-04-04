pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const IVERN_Q_KEY: &str = "Characters/Ivern/Spells/IvernQ/IvernQ";
#[allow(dead_code)]
const IVERN_W_KEY: &str = "Characters/Ivern/Spells/IvernW/IvernW";
const IVERN_E_KEY: &str = "Characters/Ivern/Spells/IvernE/IvernE";
#[allow(dead_code)]
const IVERN_R_KEY: &str = "Characters/Ivern/Spells/IvernR/IvernR";

#[derive(Default)]
pub struct PluginIvern;

impl Plugin for PluginIvern {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_ivern_skill_cast);
        app.add_observer(on_ivern_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ivern"))]
#[reflect(Component)]
pub struct Ivern;

fn on_ivern_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ivern.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ivern_q(&mut commands, entity),
        SkillSlot::W => cast_ivern_w(&mut commands, entity),
        SkillSlot::E => cast_ivern_e(&mut commands, entity),
        SkillSlot::R => cast_ivern_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_ivern_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ivern_Q_Cast"));

    // Q roots enemies
    skill_damage(
        commands,
        entity,
        IVERN_Q_KEY,
        DamageShape::Sector {
            radius: 1150.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ivern_Q_Hit")),
    );
}

fn cast_ivern_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ivern_W_Cast"));
    // W creates brush
}

fn cast_ivern_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ivern_E_Cast"));

    // E is a shield that explodes
    skill_damage(
        commands,
        entity,
        IVERN_E_KEY,
        DamageShape::Circle { radius: 750.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ivern_E_Hit")),
    );
}

fn cast_ivern_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Ivern_R_Cast"));
    // R summons Daisy
}

fn on_ivern_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ivern: Query<(), With<Ivern>>,
) {
    let source = trigger.source;
    if q_ivern.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_ivern: Query<Entity, (With<Ivern>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_ivern.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Ivern/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Ivern/Spells/IvernPassive/IvernPassive",
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
