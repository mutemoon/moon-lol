use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::buffs::lulu_buffs::{BuffLuluE, BuffLuluR, BuffLuluW};

const LULU_Q_KEY: &str = "Characters/Lulu/Spells/LuluQ/LuluQ";
#[allow(dead_code)]
const LULU_W_KEY: &str = "Characters/Lulu/Spells/LuluW/LuluW";
const LULU_E_KEY: &str = "Characters/Lulu/Spells/LuluE/LuluE";
const LULU_R_KEY: &str = "Characters/Lulu/Spells/LuluR/LuluR";

#[derive(Default)]
pub struct PluginLulu;

impl Plugin for PluginLulu {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_lulu_skill_cast);
        app.add_observer(on_lulu_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Lulu"))]
#[reflect(Component)]
pub struct Lulu;

fn on_lulu_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_lulu.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_lulu_q(&mut commands, entity),
        SkillSlot::W => cast_lulu_w(&mut commands, entity),
        SkillSlot::E => cast_lulu_e(&mut commands, entity),
        SkillSlot::R => cast_lulu_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_lulu_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Lulu_Q_Cast"));

    // Q is a bolt that slows
    skill_damage(
        commands,
        entity,
        LULU_Q_KEY,
        DamageShape::Sector {
            radius: 950.0,
            angle: 15.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lulu_Q_Hit")),
    );
}

fn cast_lulu_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Lulu_W_Cast"));

    // W polymorphs enemy or buffs ally
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluW::new(false, 0.3, 0.25, 2.5));
}

fn cast_lulu_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Lulu_E_Cast"));

    // E shields ally or damages enemy
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluE::new(80.0, 2.5));

    skill_damage(
        commands,
        entity,
        LULU_E_KEY,
        DamageShape::Circle { radius: 650.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lulu_E_Hit")),
    );
}

fn cast_lulu_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Lulu_R_Cast"));

    // R knocks up nearby enemies and grants bonus health
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffLuluR::new(400.0, true, 0.5, 4.0));

    skill_damage(
        commands,
        entity,
        LULU_R_KEY,
        DamageShape::Circle { radius: 900.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Lulu_R_Hit")),
    );
}

fn on_lulu_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_lulu: Query<(), With<Lulu>>,
) {
    let source = trigger.source;
    if q_lulu.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.8, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_lulu: Query<Entity, (With<Lulu>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_lulu.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Lulu/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Lulu/Spells/LuluPassive/LuluPassive",
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
