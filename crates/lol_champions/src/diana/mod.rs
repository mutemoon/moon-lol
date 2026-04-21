pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
};

use crate::diana::buffs::BuffDianaPassive;

const DIANA_Q_KEY: &str = "Characters/Diana/Spells/DianaQ/DianaQ";
const DIANA_W_KEY: &str = "Characters/Diana/Spells/DianaW/DianaW";
const DIANA_E_KEY: &str = "Characters/Diana/Spells/DianaE/DianaE";
const DIANA_R_KEY: &str = "Characters/Diana/Spells/DianaR/DianaR";

#[derive(Default)]
pub struct PluginDiana;

impl Plugin for PluginDiana {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_diana_skill_cast);
        app.add_observer(on_diana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Diana"))]
#[reflect(Component)]
pub struct Diana;

fn on_diana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_diana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_diana_q(&mut commands, entity),
        SkillSlot::W => cast_diana_w(&mut commands, entity),
        SkillSlot::E => cast_diana_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_diana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_diana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Diana_Q_Cast"));

    // Q is a crescent arc
    skill_damage(
        commands,
        entity,
        DIANA_Q_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 180.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Diana_Q_Hit")),
    );
}

fn cast_diana_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Diana_W_Cast"));

    // W creates orbs that damage nearby enemies
    skill_damage(
        commands,
        entity,
        DIANA_W_KEY,
        DamageShape::Circle { radius: 200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Diana_W_Hit")),
    );
}

fn cast_diana_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Diana_E_Cast"));

    // E is a dash to target
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: DIANA_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 825.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 1000.0,
        },
    );
}

fn cast_diana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Diana_R_Cast"));

    // R pulls and damages nearby enemies
    skill_damage(
        commands,
        entity,
        DIANA_R_KEY,
        DamageShape::Circle { radius: 475.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Diana_R_Hit")),
    );
}

fn on_diana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_diana: Query<(), With<Diana>>,
) {
    let source = trigger.source;
    if q_diana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies moonlight
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffDianaPassive::new());
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_diana: Query<Entity, (With<Diana>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_diana.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Diana/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Diana/Spells/DianaPassive/DianaPassive",
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
