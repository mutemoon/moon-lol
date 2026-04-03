use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::kayle_buffs::{BuffKaylePassive, BuffKayleR, BuffKayleW};
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const KAYLE_Q_KEY: &str = "Characters/Kayle/Spells/KayleQ/KayleQ";
#[allow(dead_code)]
const KAYLE_W_KEY: &str = "Characters/Kayle/Spells/KayleW/KayleW";
#[allow(dead_code)]
const KAYLE_E_KEY: &str = "Characters/Kayle/Spells/KayleE/KayleE";
const KAYLE_R_KEY: &str = "Characters/Kayle/Spells/KayleR/KayleR";

#[derive(Default)]
pub struct PluginKayle;

impl Plugin for PluginKayle {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_kayle_skill_cast);
        app.add_observer(on_kayle_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kayle"))]
#[reflect(Component)]
pub struct Kayle;

fn on_kayle_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kayle.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_kayle_q(&mut commands, entity),
        SkillSlot::W => cast_kayle_w(&mut commands, entity),
        SkillSlot::E => cast_kayle_e(&mut commands, entity),
        SkillSlot::R => cast_kayle_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_kayle_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Kayle_Q_Cast"));

    // Q is a skillshot that slows
    skill_damage(
        commands,
        entity,
        KAYLE_Q_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kayle_Q_Hit")),
    );
}

fn cast_kayle_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Kayle_W_Cast"));

    // W heals and grants movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleW::new(80.0, 0.35, 2.5));
}

fn cast_kayle_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Kayle_E_Cast"));

    // E enhances next attack
}

fn cast_kayle_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Kayle_R_Cast"));

    // R makes Kayle invulnerable and deals damage after delay
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKayleR::new(2.5));

    skill_damage(
        commands,
        entity,
        KAYLE_R_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Kayle_R_Hit")),
    );
}

fn on_kayle_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kayle: Query<(), With<Kayle>>,
) {
    let source = trigger.source;
    if q_kayle.get(source).is_err() {
        return;
    }

    let _target = trigger.event_target();

    // Passive grants attackspeed
    commands
        .entity(source)
        .with_related::<BuffOf>(BuffKaylePassive::new(0.15, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_kayle: Query<Entity, (With<Kayle>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_kayle.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Kayle/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Kayle/Spells/KaylePassive/KaylePassive",
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
