use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::buffs::heimerdinger_buffs::BuffHeimerPassive;

#[allow(dead_code)]
const HEIMER_Q_KEY: &str = "Characters/Heimerdinger/Spells/HeimerdingerQ/HeimerdingerQ";
const HEIMER_W_KEY: &str = "Characters/Heimerdinger/Spells/HeimerdingerW/HeimerdingerW";
const HEIMER_E_KEY: &str = "Characters/Heimerdinger/Spells/HeimerdingerE/HeimerdingerE";
#[allow(dead_code)]
const HEIMER_R_KEY: &str = "Characters/Heimerdinger/Spells/HeimerdingerR/HeimerdingerR";

#[derive(Default)]
pub struct PluginHeimerdinger;

impl Plugin for PluginHeimerdinger {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_heimerdinger_skill_cast);
        app.add_observer(on_heimerdinger_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Heimerdinger"))]
#[reflect(Component)]
pub struct Heimerdinger;

fn on_heimerdinger_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_heimer: Query<(), With<Heimerdinger>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_heimer.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_heimer_q(&mut commands, entity),
        SkillSlot::W => cast_heimer_w(&mut commands, entity),
        SkillSlot::E => cast_heimer_e(&mut commands, entity),
        SkillSlot::R => cast_heimer_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_heimer_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Heimerdinger_Q_Cast"));
    // Q places a turret
}

fn cast_heimer_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Heimerdinger_W_Cast"));

    // W fires rockets
    skill_damage(
        commands,
        entity,
        HEIMER_W_KEY,
        DamageShape::Sector {
            radius: 1150.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Heimerdinger_W_Hit")),
    );
}

fn cast_heimer_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Heimerdinger_E_Cast"));

    // E is a grenade that stuns
    skill_damage(
        commands,
        entity,
        HEIMER_E_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Heimerdinger_E_Hit")),
    );
}

fn cast_heimer_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Heimerdinger_R_Cast"));
    // R enhances the next spell
}

fn on_heimerdinger_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_heimer: Query<(), With<Heimerdinger>>,
) {
    let source = trigger.source;
    if q_heimer.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffHeimerPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_heimer: Query<Entity, (With<Heimerdinger>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_heimer.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Heimerdinger/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Heimerdinger/Spells/HeimerdingerPassive/HeimerdingerPassive",
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
