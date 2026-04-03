use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill,
    SkillOf, SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffAnniePassive, BuffAnnieShield, BuffOf, DamageType, DebuffStun, PassiveSkillOf};

const ANNIE_Q_KEY: &str = "Characters/Annie/Spells/AnnieQ/AnnieQ";
const ANNIE_W_KEY: &str = "Characters/Annie/Spells/AnnieW/AnnieW";
#[allow(dead_code)]
const ANNIE_E_KEY: &str = "Characters/Annie/Spells/AnnieE/AnnieE";
const ANNIE_R_KEY: &str = "Characters/Annie/Spells/AnnieR/AnnieR";

#[derive(Default)]
pub struct PluginAnnie;

impl Plugin for PluginAnnie {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_annie_skill_cast);
        app.add_observer(on_annie_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Annie"))]
#[reflect(Component)]
pub struct Annie;

fn on_annie_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_annie.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_annie_q(&mut commands, entity),
        SkillSlot::W => cast_annie_w(&mut commands, entity),
        SkillSlot::E => cast_annie_e(&mut commands, entity),
        SkillSlot::R => cast_annie_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_annie_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Annie_Q_Cast"));

    // Q is targeted damage
    skill_damage(
        commands,
        entity,
        ANNIE_Q_KEY,
        DamageShape::Circle { radius: 625.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Annie_Q_Hit")),
    );

    // Increment passive stacks
    commands.entity(entity).with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn cast_annie_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Annie_W_Cast"));

    // W is a cone
    skill_damage(
        commands,
        entity,
        ANNIE_W_KEY,
        DamageShape::Sector { radius: 600.0, angle: 50.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Annie_W_Hit")),
    );

    commands.entity(entity).with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn cast_annie_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Annie_E_Cast"));

    // E grants shield
    commands.entity(entity).with_related::<BuffOf>(BuffAnnieShield::new());
}

fn cast_annie_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Annie_R_Cast"));

    // R summons Tibbers - area damage
    skill_damage(
        commands,
        entity,
        ANNIE_R_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Annie_R_Hit")),
    );

    commands.entity(entity).with_related::<BuffOf>(BuffAnniePassive::increment());
}

fn on_annie_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_annie: Query<(), With<Annie>>,
) {
    let source = trigger.source;
    if q_annie.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Check if Annie has 4 passive stacks for stun
    // For now, just stun
    commands.entity(target).with_related::<BuffOf>(DebuffStun::new(1.5));
}

fn add_skills(
    mut commands: Commands,
    q_annie: Query<Entity, (With<Annie>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_annie.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Annie/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Annie/Spells/AnniePassive/AnniePassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
