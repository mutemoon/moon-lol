pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::lux::buffs::{BuffLuxIllumination, BuffLuxQ};

const LUX_Q_KEY: &str = "Characters/Lux/Spells/LuxQ/LuxQ";
#[allow(dead_code)]
const LUX_W_KEY: &str = "Characters/Lux/Spells/LuxW/LuxW";
const LUX_E_KEY: &str = "Characters/Lux/Spells/LuxE/LuxE";
const LUX_R_KEY: &str = "Characters/Lux/Spells/LuxR/LuxR";

#[derive(Default)]
pub struct PluginLux;

impl Plugin for PluginLux {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_lux_skill_cast);
        app.add_observer(on_lux_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lux"))]
#[reflect(Component)]
pub struct Lux;

fn on_lux_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lux.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_lux_q(&mut commands, entity),
        SkillSlot::W => cast_lux_w(&mut commands, entity),
        SkillSlot::E => cast_lux_e(&mut commands, entity),
        SkillSlot::R => cast_lux_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_lux_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lux_Q_Cast"));

    // Q roots enemies
    skill_damage(
        commands,
        entity,
        LUX_Q_KEY,
        DamageShape::Sector {
            radius: 1300.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lux_Q_Hit")),
    );
}

fn cast_lux_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lux_W_Cast"));

    // W is a shield
}

fn cast_lux_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lux_E_Cast"));

    // E slows and deals damage
    skill_damage(
        commands,
        entity,
        LUX_E_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lux_E_Hit")),
    );
}

fn cast_lux_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lux_R_Cast"));

    // R is a global beam
    skill_damage(
        commands,
        entity,
        LUX_R_KEY,
        DamageShape::Sector {
            radius: 3400.0,
            angle: 20.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lux_R_Hit")),
    );
}

fn on_lux_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lux: Query<(), With<Lux>>,
) {
    let source = trigger.source;
    if q_lux.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLuxQ::new(2.0, 2.0));

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 5.0));

    // Passive marks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLuxIllumination::new(40.0, 6.0));
}

fn add_skills(
    mut commands: Commands,
    q_lux: Query<Entity, (With<Lux>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_lux.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Lux/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Lux/Spells/LuxPassive/LuxPassive",
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
