pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::kled::buffs::{BuffKledE, BuffKledR, BuffKledW};

const KLED_Q_KEY: &str = "Characters/Kled/Spells/KledQ/KledQ";
#[allow(dead_code)]
const KLED_W_KEY: &str = "Characters/Kled/Spells/KledW/KledW";
const KLED_E_KEY: &str = "Characters/Kled/Spells/KledE/KledE";
const KLED_R_KEY: &str = "Characters/Kled/Spells/KledR/KledR";

#[derive(Default)]
pub struct PluginKled;

impl Plugin for PluginKled {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kled_skill_cast);
        app.add_observer(on_kled_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kled"))]
#[reflect(Component)]
pub struct Kled;

fn on_kled_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kled: Query<(), With<Kled>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kled.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kled_q(&mut commands, entity),
        SkillSlot::W => cast_kled_w(&mut commands, entity),
        SkillSlot::E => cast_kled_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_kled_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_kled_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kled_Q_Cast"));

    // Q throws bear trap
    skill_damage(
        commands,
        entity,
        KLED_Q_KEY,
        DamageShape::Sector {
            radius: 800.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kled_Q_Hit")),
    );
}

fn cast_kled_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kled_W_Cast"));

    // W grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledW::new(0.7, 4.0));
}

fn cast_kled_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kled_E_Cast"));

    // E is a dash
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: KLED_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 550.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 900.0,
        },
    );

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledE::new(0.5, 2.0));
}

fn cast_kled_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kled_R_Cast"));

    // R is a charge that provides shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKledR::new(0.5, 100.0, 4.0));

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: KLED_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 3500.0 },
            damage: None,
            speed: 1500.0,
        },
    );
}

fn on_kled_damage_hit(
    trigger: On<EventDamageCreate>,
    _commands: Commands,
    q_kled: Query<(), With<Kled>>,
) {
    let source = trigger.source;
    if q_kled.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive: Kled gains courage on hit
}

fn add_skills(
    mut commands: Commands,
    q_kled: Query<Entity, (With<Kled>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kled.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kled/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kled/Spells/KledPassive/KledPassive",
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
