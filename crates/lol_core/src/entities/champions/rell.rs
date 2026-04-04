use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::rell_buffs::{BuffRellE, BuffRellR, BuffRellW};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const RELL_Q_KEY: &str = "Characters/Rell/Spells/RellQ/RellQ";
const RELL_W_KEY: &str = "Characters/Rell/Spells/RellW/RellW";
const RELL_E_KEY: &str = "Characters/Rell/Spells/RellE/RellE";
const RELL_R_KEY: &str = "Characters/Rell/Spells/RellR/RellR";

#[derive(Default)]
pub struct PluginRell;

impl Plugin for PluginRell {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_rell_skill_cast);
        app.add_observer(on_rell_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Rell"))]
#[reflect(Component)]
pub struct Rell;

fn on_rell_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_rell.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_rell_q(&mut commands, entity),
        SkillSlot::W => cast_rell_w(&mut commands, entity),
        SkillSlot::E => cast_rell_e(&mut commands, entity),
        SkillSlot::R => cast_rell_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_rell_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Rell_Q_Cast"));

    // Q is shattering strike - damage and armor reduction
    skill_damage(
        commands,
        entity,
        RELL_Q_KEY,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_Q_Hit")),
    );
}

fn cast_rell_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Rell_W_Cast"));

    // W is crashing blow - damage and knockup
    skill_damage(
        commands,
        entity,
        RELL_W_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_W_Hit")),
    );
}

fn cast_rell_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Rell_E_Cast"));

    // E is full bind - stun
    skill_damage(
        commands,
        entity,
        RELL_E_KEY,
        DamageShape::Circle { radius: 500.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_E_Hit")),
    );
}

fn cast_rell_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Rell_R_Cast"));

    // R is catharsis - AoE damage and slow
    skill_damage(
        commands,
        entity,
        RELL_R_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Rell_R_Hit")),
    );
}

fn on_rell_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_rell: Query<(), With<Rell>>,
) {
    let source = trigger.source;
    if q_rell.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellW::new(0.5, 1.5));
    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellE::new(0.75, 1.0));
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffRellR::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_rell: Query<Entity, (With<Rell>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_rell.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Rell/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Rell/Spells/RellPassive/RellPassive",
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
