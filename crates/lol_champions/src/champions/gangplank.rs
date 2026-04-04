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

use crate::buffs::gangplank_buffs::BuffGangplankPassive;

const GANGPLANK_Q_KEY: &str = "Characters/Gangplank/Spells/GangplankQ/GangplankQ";
#[allow(dead_code)]
const GANGPLANK_W_KEY: &str = "Characters/Gangplank/Spells/GangplankW/GangplankW";
#[allow(dead_code)]
const GANGPLANK_E_KEY: &str = "Characters/Gangplank/Spells/GangplankE/GangplankE";
const GANGPLANK_R_KEY: &str = "Characters/Gangplank/Spells/GangplankR/GangplankR";

#[derive(Default)]
pub struct PluginGangplank;

impl Plugin for PluginGangplank {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_gangplank_skill_cast);
        app.add_observer(on_gangplank_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Gangplank"))]
#[reflect(Component)]
pub struct Gangplank;

fn on_gangplank_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_gangplank.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_gangplank_q(&mut commands, entity),
        SkillSlot::W => cast_gangplank_w(&mut commands, entity),
        SkillSlot::E => cast_gangplank_e(&mut commands, entity),
        SkillSlot::R => cast_gangplank_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_gangplank_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_Q_Cast"));

    // Q is targeted damage
    skill_damage(
        commands,
        entity,
        GANGPLANK_Q_KEY,
        DamageShape::Nearest {
            max_distance: 625.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Gangplank_Q_Hit")),
    );
}

fn cast_gangplank_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_W_Cast"));
    // W removes CC and heals
}

fn cast_gangplank_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_E_Cast"));
    // E places barrel
}

fn cast_gangplank_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Gangplank_R_Cast"));

    // R is global AoE
    skill_damage(
        commands,
        entity,
        GANGPLANK_R_KEY,
        DamageShape::Circle { radius: 20000.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Gangplank_R_Hit")),
    );
}

fn on_gangplank_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_gangplank: Query<(), With<Gangplank>>,
) {
    let source = trigger.source;
    if q_gangplank.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply passive
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffGangplankPassive::new());
}

fn add_skills(
    mut commands: Commands,
    q_gangplank: Query<Entity, (With<Gangplank>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_gangplank.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Gangplank/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Gangplank/Spells/GangplankPassive/GangplankPassive",
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
