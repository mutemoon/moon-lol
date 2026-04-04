use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::base::buff::BuffOf;
use crate::buffs::kaisa_buffs::{BuffKaisaE, BuffKaisaPlasma, BuffKaisaR};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const KAISA_Q_KEY: &str = "Characters/Kaisa/Spells/KaisaQ/KaisaQ";
const KAISA_W_KEY: &str = "Characters/Kaisa/Spells/KaisaW/KaisaW";
#[allow(dead_code)]
const KAISA_E_KEY: &str = "Characters/Kaisa/Spells/KaisaE/KaisaE";
const KAISA_R_KEY: &str = "Characters/Kaisa/Spells/KaisaR/KaisaR";

#[derive(Default)]
pub struct PluginKaisa;

impl Plugin for PluginKaisa {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kaisa_skill_cast);
        app.add_observer(on_kaisa_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kaisa"))]
#[reflect(Component)]
pub struct Kaisa;

fn on_kaisa_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kaisa.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kaisa_q(&mut commands, entity),
        SkillSlot::W => cast_kaisa_w(&mut commands, entity),
        SkillSlot::E => cast_kaisa_e(&mut commands, entity),
        SkillSlot::R => cast_kaisa_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_kaisa_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_Q_Cast"));

    // Q fires 6 missiles that spread to nearby enemies
    skill_damage(
        commands,
        entity,
        KAISA_Q_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Kaisa_Q_Hit")),
    );
}

fn cast_kaisa_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_W_Cast"));

    // W is a long-range missile that applies 2 plasma stacks
    skill_damage(
        commands,
        entity,
        KAISA_W_KEY,
        DamageShape::Sector {
            radius: 3000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kaisa_W_Hit")),
    );
}

fn cast_kaisa_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_E_Cast"));

    // E charges movement speed then grants attackspeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaE::new(0.8, 4.0));
}

fn cast_kaisa_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kaisa_R_Cast"));

    // R is a dash to a plasma-marked enemy with shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKaisaR::new(100.0, 4.0));

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: KAISA_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 2000.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1500.0,
        },
    );
}

fn on_kaisa_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kaisa: Query<(), With<Kaisa>>,
) {
    let source = trigger.source;
    if q_kaisa.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply plasma stacks (Q applies 1, W applies 2)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKaisaPlasma::new(1, 5.0));
}

fn add_skills(
    mut commands: Commands,
    q_kaisa: Query<Entity, (With<Kaisa>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kaisa.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kaisa/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kaisa/Spells/KaisaPassive/KaisaPassive",
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
