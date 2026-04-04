pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_slot_from_index,
    spawn_skill_particle, CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot,
    Skills,
};

#[allow(dead_code)]
const JINX_Q_KEY: &str = "Characters/Jinx/Spells/JinxQ/JinxQ";
const JINX_W_KEY: &str = "Characters/Jinx/Spells/JinxW/JinxW";
const JINX_E_KEY: &str = "Characters/Jinx/Spells/JinxE/JinxE";
const JINX_R_KEY: &str = "Characters/Jinx/Spells/JinxR/JinxR";

#[derive(Default)]
pub struct PluginJinx;

impl Plugin for PluginJinx {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_jinx_skill_cast);
        app.add_observer(on_jinx_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Jinx"))]
#[reflect(Component)]
pub struct Jinx;

fn on_jinx_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_jinx.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_jinx_q(&mut commands, entity),
        SkillSlot::W => cast_jinx_w(&mut commands, entity),
        SkillSlot::E => cast_jinx_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_jinx_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_jinx_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_Q_Cast"));
    // Q switches between minigun and rocket launcher
    // Minigun gives attackspeed stacks, rocket deals AoE
    reset_skill_attack(commands, entity);
}

fn cast_jinx_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_W_Cast"));

    // W is a skillshot that slows
    skill_damage(
        commands,
        entity,
        JINX_W_KEY,
        DamageShape::Sector {
            radius: 1500.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jinx_W_Hit")),
    );
}

fn cast_jinx_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_E_Cast"));

    // E places traps that explode and knock up
    skill_damage(
        commands,
        entity,
        JINX_E_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Jinx_E_Hit")),
    );
}

fn cast_jinx_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Jinx_R_Cast"));

    // R is a global rocket with damage based on distance
    skill_damage(
        commands,
        entity,
        JINX_R_KEY,
        DamageShape::Sector {
            radius: 25000.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Jinx_R_Hit")),
    );
}

fn on_jinx_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_jinx: Query<(), With<Jinx>>,
) {
    let source = trigger.source;
    if q_jinx.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // W applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_jinx: Query<Entity, (With<Jinx>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_jinx.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Jinx/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Jinx/Spells/JinxPassive/JinxPassive",
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
