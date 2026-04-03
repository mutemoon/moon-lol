use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, BuffOf,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill, SkillOf,
    SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffTwistedFateWSlow, BuffTwistedFateWStun, DamageType, PassiveSkillOf};

const TWISTED_FATE_Q_KEY: &str = "Characters/TwistedFate/Spells/TwistedFateQ/TwistedFateQ";
const TWISTED_FATE_W_KEY: &str = "Characters/TwistedFate/Spells/TwistedFateW/TwistedFateW";
const TWISTED_FATE_E_KEY: &str = "Characters/TwistedFate/Spells/TwistedFateE/TwistedFateE";
const TWISTED_FATE_R_KEY: &str = "Characters/TwistedFate/Spells/TwistedFateR/TwistedFateR";

#[derive(Default)]
pub struct PluginTwistedFate;

impl Plugin for PluginTwistedFate {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_twisted_fate_skill_cast);
        app.add_observer(on_twisted_fate_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("TwistedFate"))]
#[reflect(Component)]
pub struct TwistedFate;

fn on_twisted_fate_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_twisted_fate.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_twisted_fate_q(&mut commands, entity),
        SkillSlot::W => cast_twisted_fate_w(&mut commands, entity),
        SkillSlot::E => cast_twisted_fate_e(&mut commands, entity),
        SkillSlot::R => cast_twisted_fate_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_twisted_fate_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("TwistedFate_Q_Cast"));

    skill_damage(
        commands,
        entity,
        TWISTED_FATE_Q_KEY,
        DamageShape::Sector { radius: 1450.0, angle: 25.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("TwistedFate_Q_Hit")),
    );
}

fn cast_twisted_fate_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("TwistedFate_W_Cast"));

    skill_damage(
        commands,
        entity,
        TWISTED_FATE_W_KEY,
        DamageShape::Circle { radius: 325.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("TwistedFate_W_Hit")),
    );
}

fn cast_twisted_fate_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("TwistedFate_E_Cast"));
}

fn cast_twisted_fate_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("TwistedFate_R_Cast"));
}

fn on_twisted_fate_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_twisted_fate: Query<(), With<TwistedFate>>,
) {
    let source = trigger.source;
    if q_twisted_fate.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands.entity(target).with_related::<BuffOf>(BuffTwistedFateWSlow::new(0.35, 2.0));
    commands.entity(target).with_related::<BuffOf>(BuffTwistedFateWStun::new(1.5));
}

fn add_skills(
    mut commands: Commands,
    q_twisted_fate: Query<Entity, (With<TwistedFate>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_twisted_fate.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/TwistedFate/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/TwistedFate/Spells/TwistedFatePassive/TwistedFatePassive",
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
