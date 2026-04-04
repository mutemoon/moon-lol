use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::cc_debuffs::DebuffSlow;
use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::damage::{DamageType, EventDamageCreate};
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const BARD_Q_KEY: &str = "Characters/Bard/Spells/BardQ/BardQ";
#[allow(dead_code)]
const BARD_W_KEY: &str = "Characters/Bard/Spells/BardW/BardW";
#[allow(dead_code)]
const BARD_E_KEY: &str = "Characters/Bard/Spells/BardE/BardE";
const BARD_R_KEY: &str = "Characters/Bard/Spells/BardR/BardR";

#[derive(Default)]
pub struct PluginBard;

impl Plugin for PluginBard {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_bard_skill_cast);
        app.add_observer(on_bard_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Bard"))]
#[reflect(Component)]
pub struct Bard;

fn on_bard_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_bard.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_bard_q(&mut commands, entity),
        SkillSlot::W => cast_bard_w(&mut commands, entity),
        SkillSlot::E => cast_bard_e(&mut commands, entity),
        SkillSlot::R => cast_bard_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_bard_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Bard_Q_Cast"));

    // Q is a binding missile
    skill_damage(
        commands,
        entity,
        BARD_Q_KEY,
        DamageShape::Sector {
            radius: 850.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Bard_Q_Hit")),
    );
}

fn cast_bard_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Bard_W_Cast"));
    // W is a heal shrine - no direct damage
}

fn cast_bard_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Bard_E_Cast"));
    // E is a tunnel - no direct damage
}

fn cast_bard_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Bard_R_Cast"));
    // R is a global AoE stun
    skill_damage(
        commands,
        entity,
        BARD_R_KEY,
        DamageShape::Circle { radius: 3400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Bard_R_Hit")),
    );
}

fn on_bard_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_bard: Query<(), With<Bard>>,
) {
    let source = trigger.source;
    if q_bard.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_bard: Query<Entity, (With<Bard>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_bard.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Bard/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Bard/Spells/BardPassive/BardPassive",
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
