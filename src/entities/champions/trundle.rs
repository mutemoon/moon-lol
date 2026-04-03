use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, BuffOf,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill, SkillOf, SkillSlot, Skills,
    TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffTrundleQ, DamageType, PassiveSkillOf};

const TRUNDLE_Q_KEY: &str = "Characters/Trundle/Spells/TrundleQ/TrundleQ";
const TRUNDLE_W_KEY: &str = "Characters/Trundle/Spells/TrundleW/TrundleW";
const TRUNDLE_E_KEY: &str = "Characters/Trundle/Spells/TrundleE/TrundleE";
const TRUNDLE_R_KEY: &str = "Characters/Trundle/Spells/TrundleR/TrundleR";

#[derive(Default)]
pub struct PluginTrundle;

impl Plugin for PluginTrundle {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_trundle_skill_cast);
        app.add_observer(on_trundle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Trundle"))]
#[reflect(Component)]
pub struct Trundle;

fn on_trundle_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_trundle.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_trundle_q(&mut commands, entity),
        SkillSlot::W => cast_trundle_w(&mut commands, entity),
        SkillSlot::E => cast_trundle_e(&mut commands, entity),
        SkillSlot::R => cast_trundle_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_trundle_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_Q_Cast"));
}

fn cast_trundle_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_W_Cast"));
}

fn cast_trundle_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_E_Cast"));

    skill_damage(
        commands,
        entity,
        TRUNDLE_E_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Trundle_E_Hit")),
    );
}

fn cast_trundle_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Trundle_R_Cast"));

    skill_damage(
        commands,
        entity,
        TRUNDLE_R_KEY,
        DamageShape::Nearest { max_distance: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Trundle_R_Hit")),
    );
}

fn on_trundle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_trundle: Query<(), With<Trundle>>,
) {
    let source = trigger.source;
    if q_trundle.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffTrundleQ::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_trundle: Query<Entity, (With<Trundle>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_trundle.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Trundle/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Trundle/Spells/TrundlePassive/TrundlePassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
