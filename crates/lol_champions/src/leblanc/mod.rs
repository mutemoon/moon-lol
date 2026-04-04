pub mod buffs;

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

use crate::leblanc::buffs::{BuffLeBlancE, BuffLeBlancQ};

const LEBLANC_Q_KEY: &str = "Characters/LeBlanc/Spells/LeBlancQ/LeBlancQ";
const LEBLANC_W_KEY: &str = "Characters/LeBlanc/Spells/LeBlancW/LeBlancW";
const LEBLANC_E_KEY: &str = "Characters/LeBlanc/Spells/LeBlancE/LeBlancE";
const LEBLANC_R_KEY: &str = "Characters/LeBlanc/Spells/LeBlancR/LeBlancR";

#[derive(Default)]
pub struct PluginLeBlanc;

impl Plugin for PluginLeBlanc {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_leblanc_skill_cast);
        app.add_observer(on_leblanc_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("LeBlanc"))]
#[reflect(Component)]
pub struct LeBlanc;

fn on_leblanc_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leblanc: Query<(), With<LeBlanc>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_leblanc.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_leblanc_q(&mut commands, entity),
        SkillSlot::W => cast_leblanc_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_leblanc_e(&mut commands, entity),
        SkillSlot::R => cast_leblanc_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_leblanc_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("LeBlanc_Q_Cast"));

    // Q marks enemy
    skill_damage(
        commands,
        entity,
        LEBLANC_Q_KEY,
        DamageShape::Sector {
            radius: 700.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("LeBlanc_Q_Hit")),
    );
}

fn cast_leblanc_w(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("LeBlanc_W_Cast"));

    // W is a dash with damage
    skill_damage(
        commands,
        entity,
        LEBLANC_W_KEY,
        DamageShape::Circle { radius: 100.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("LeBlanc_W_Hit")),
    );
}

fn cast_leblanc_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("LeBlanc_E_Cast"));

    // E chains enemy
    skill_damage(
        commands,
        entity,
        LEBLANC_E_KEY,
        DamageShape::Sector {
            radius: 950.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("LeBlanc_E_Hit")),
    );
}

fn cast_leblanc_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("LeBlanc_R_Cast"));

    // R mimics the last used skill
    skill_damage(
        commands,
        entity,
        LEBLANC_R_KEY,
        DamageShape::Sector {
            radius: 700.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("LeBlanc_R_Hit")),
    );
}

fn on_leblanc_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_leblanc: Query<(), With<LeBlanc>>,
) {
    let source = trigger.source;
    if q_leblanc.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies mark
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLeBlancQ::new(100.0, 3.5));

    // E roots
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffLeBlancE::new(80.0, 1.5, 3.0));
}

fn add_skills(
    mut commands: Commands,
    q_leblanc: Query<Entity, (With<LeBlanc>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_leblanc.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/LeBlanc/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/LeBlanc/Spells/LeBlancPassive/LeBlancPassive",
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
