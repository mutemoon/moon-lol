pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::tristana::buffs::BuffTristanaW;

const TRISTANA_Q_KEY: &str = "Characters/Tristana/Spells/TristanaQ/TristanaQ";
const TRISTANA_W_KEY: &str = "Characters/Tristana/Spells/TristanaW/TristanaW";
const TRISTANA_E_KEY: &str = "Characters/Tristana/Spells/TristanaE/TristanaE";
const TRISTANA_R_KEY: &str = "Characters/Tristana/Spells/TristanaR/TristanaR";

#[derive(Default)]
pub struct PluginTristana;

impl Plugin for PluginTristana {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_tristana_skill_cast);
        app.add_observer(on_tristana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Tristana"))]
#[reflect(Component)]
pub struct Tristana;

fn on_tristana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_tristana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_tristana_q(&mut commands, entity),
        SkillSlot::W => cast_tristana_w(&mut commands, entity),
        SkillSlot::E => cast_tristana_e(&mut commands, entity),
        SkillSlot::R => cast_tristana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_tristana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_Q_Cast"));
}

fn cast_tristana_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_W_Cast"));

    skill_damage(
        commands,
        entity,
        TRISTANA_W_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Tristana_W_Hit")),
    );
}

fn cast_tristana_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_E_Cast"));

    skill_damage(
        commands,
        entity,
        TRISTANA_E_KEY,
        DamageShape::Nearest {
            max_distance: 700.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Tristana_E_Hit")),
    );
}

fn cast_tristana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Tristana_R_Cast"));

    skill_damage(
        commands,
        entity,
        TRISTANA_R_KEY,
        DamageShape::Nearest {
            max_distance: 700.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Tristana_R_Hit")),
    );
}

fn on_tristana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_tristana: Query<(), With<Tristana>>,
) {
    let source = trigger.source;
    if q_tristana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTristanaW::new(0.5, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_tristana: Query<Entity, (With<Tristana>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_tristana.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Tristana/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Tristana/Spells/TristanaPassive/TristanaPassive",
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
