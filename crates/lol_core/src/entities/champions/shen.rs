use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::shen_buffs::BuffShenW;
use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::damage::{DamageType, EventDamageCreate};
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const SHEN_Q_KEY: &str = "Characters/Shen/Spells/ShenQ/ShenQ";
#[allow(dead_code)]
const SHEN_W_KEY: &str = "Characters/Shen/Spells/ShenW/ShenW";
#[allow(dead_code)]
const SHEN_E_KEY: &str = "Characters/Shen/Spells/ShenE/ShenE";
#[allow(dead_code)]
const SHEN_R_KEY: &str = "Characters/Shen/Spells/ShenR/ShenR";

#[derive(Default)]
pub struct PluginShen;

impl Plugin for PluginShen {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_shen_skill_cast);
        app.add_observer(on_shen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shen"))]
#[reflect(Component)]
pub struct Shen;

fn on_shen_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_shen_q(&mut commands, entity),
        SkillSlot::W => cast_shen_w(&mut commands, entity),
        SkillSlot::E => cast_shen_e(&mut commands, entity),
        SkillSlot::R => cast_shen_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shen_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Shen_Q_Cast"));

    // Q is shadow dash - damage
    skill_damage(
        commands,
        entity,
        SHEN_Q_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shen_Q_Hit")),
    );
}

fn cast_shen_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Shen_W_Cast"));

    // W is spirits refuge - dodge
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}

fn cast_shen_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Shen_E_Cast"));

    // E is leap - dash
}

fn cast_shen_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Shen_R_Cast"));

    // R is stand united - global shield
}

fn on_shen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
) {
    let source = trigger.source;
    if q_shen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks with spirit
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_shen: Query<Entity, (With<Shen>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_shen.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Shen/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Shen/Spells/ShenPassive/ShenPassive",
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
