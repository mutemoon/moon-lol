use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::shyvana_buffs::BuffShyvanaE;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const SHYVANA_Q_KEY: &str = "Characters/Shyvana/Spells/ShyvanaQ/ShyvanaQ";
const SHYVANA_W_KEY: &str = "Characters/Shyvana/Spells/ShyvanaW/ShyvanaW";
const SHYVANA_E_KEY: &str = "Characters/Shyvana/Spells/ShyvanaE/ShyvanaE";
#[allow(dead_code)]
const SHYVANA_R_KEY: &str = "Characters/Shyvana/Spells/ShyvanaR/ShyvanaR";

#[derive(Default)]
pub struct PluginShyvana;

impl Plugin for PluginShyvana {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_shyvana_skill_cast);
        app.add_observer(on_shyvana_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shyvana"))]
#[reflect(Component)]
pub struct Shyvana;

fn on_shyvana_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shyvana.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_shyvana_q(&mut commands, entity),
        SkillSlot::W => cast_shyvana_w(&mut commands, entity),
        SkillSlot::E => cast_shyvana_e(&mut commands, entity),
        SkillSlot::R => cast_shyvana_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shyvana_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_Q_Cast"));

    // Q is twin bite - damage
    skill_damage(
        commands,
        entity,
        SHYVANA_Q_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Shyvana_Q_Hit")),
    );
}

fn cast_shyvana_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_W_Cast"));

    // W is flame breath - damage over time
    skill_damage(
        commands,
        entity,
        SHYVANA_W_KEY,
        DamageShape::Sector {
            radius: 600.0,
            angle: 25.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shyvana_W_Hit")),
    );
}

fn cast_shyvana_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_E_Cast"));

    // E is dragon descent - knockup
    skill_damage(
        commands,
        entity,
        SHYVANA_E_KEY,
        DamageShape::Circle { radius: 450.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shyvana_E_Hit")),
    );
}

fn cast_shyvana_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Shyvana_R_Cast"));

    // R is shape shift - transformation
}

fn on_shyvana_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shyvana: Query<(), With<Shyvana>>,
) {
    let source = trigger.source;
    if q_shyvana.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E stuns
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShyvanaE::new(0.5, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_shyvana: Query<Entity, (With<Shyvana>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_shyvana.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Shyvana/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Shyvana/Spells/ShyvanaPassive/ShyvanaPassive",
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
