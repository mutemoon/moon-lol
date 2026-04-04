use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashMoveType};
use crate::base::buff::BuffOf;
use crate::buffs::lucian_buffs::{BuffLucianPassive, BuffLucianW};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, skill_slot_from_index,
    spawn_skill_particle, CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot,
    Skills,
};

const LUCIAN_Q_KEY: &str = "Characters/Lucian/Spells/LucianQ/LucianQ";
const LUCIAN_W_KEY: &str = "Characters/Lucian/Spells/LucianW/LucianW";
const LUCIAN_E_KEY: &str = "Characters/Lucian/Spells/LucianE/LucianE";
const LUCIAN_R_KEY: &str = "Characters/Lucian/Spells/LucianR/LucianR";

#[derive(Default)]
pub struct PluginLucian;

impl Plugin for PluginLucian {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_lucian_skill_cast);
        app.add_observer(on_lucian_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lucian"))]
#[reflect(Component)]
pub struct Lucian;

fn on_lucian_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lucian.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_lucian_q(&mut commands, entity),
        SkillSlot::W => cast_lucian_w(&mut commands, entity),
        SkillSlot::E => cast_lucian_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_lucian_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_lucian_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_Q_Cast"));

    // Q is a piercing light beam
    skill_damage(
        commands,
        entity,
        LUCIAN_Q_KEY,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Lucian_Q_Hit")),
    );
}

fn cast_lucian_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_W_Cast"));

    // W marks enemies and grants movespeed to Lucian
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLucianW::new(60.0, 6.0));

    skill_damage(
        commands,
        entity,
        LUCIAN_W_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lucian_W_Hit")),
    );
}

fn cast_lucian_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_E_Cast"));

    // E is a dash
    reset_skill_attack(commands, entity);

    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: LUCIAN_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 425.0 },
            damage: None,
            speed: 1000.0,
        },
    );
}

fn cast_lucian_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lucian_R_Cast"));

    // R is a barrage of shots
    skill_damage(
        commands,
        entity,
        LUCIAN_R_KEY,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Lucian_R_Hit")),
    );
}

fn on_lucian_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lucian: Query<(), With<Lucian>>,
) {
    let source = trigger.source;
    if q_lucian.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive procs after abilities
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffLucianPassive::new(50.0, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_lucian: Query<Entity, (With<Lucian>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_lucian.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Lucian/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Lucian/Spells/LucianPassive/LucianPassive",
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
