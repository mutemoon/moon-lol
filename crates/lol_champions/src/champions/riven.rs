use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::buffs::riven_passive::BuffRivenPassive;

const RIVEN_Q_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave";
const RIVEN_W_KEY: &str = "Characters/Riven/Spells/RivenMartyrAbility/RivenMartyr";
const RIVEN_E_KEY: &str = "Characters/Riven/Spells/RivenFeintAbility/RivenFeint";
const RIVEN_Q_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_riven_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Riven"))]
#[reflect(Component)]
pub struct Riven;

fn on_riven_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_riven_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_riven_w(&mut commands, entity),
        SkillSlot::E => cast_riven_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_riven_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_riven_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|window| window.stage).unwrap_or(1);

    let (animation_hash, particle_hash) = match stage {
        1 => (hash_bin("Spell1A"), hash_bin("Riven_Q_01_Detonate")),
        2 => (hash_bin("Spell1B"), hash_bin("Riven_Q_02_Detonate")),
        _ => (hash_bin("Spell1C"), hash_bin("Riven_Q_03_Detonate")),
    };

    play_skill_animation(commands, entity, animation_hash);
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: RIVEN_Q_KEY.into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: Some(DashDamage {
                radius_end: 250.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("FirstSlashDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 1000.0,
        },
    );
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffRivenPassive);
    spawn_skill_particle(commands, entity, particle_hash);

    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    } else {
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            RIVEN_Q_RECAST_WINDOW,
        ));
    }
}

fn cast_riven_w(commands: &mut Commands, entity: Entity) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_W_Cast"));
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    skill_damage(
        commands,
        entity,
        RIVEN_W_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        None,
    );
}

fn cast_riven_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_E_Mis"));
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: RIVEN_E_KEY.into(),
            move_type: DashMoveType::Fixed(250.0),
            damage: None,
            speed: 1000.0,
        },
    );
}

fn cast_riven_r(commands: &mut Commands, entity: Entity) {
    spawn_skill_particle(commands, entity, hash_bin("Riven_R_Indicator_Ring"));
    spawn_skill_particle(commands, entity, hash_bin("Riven_R_ALL_Warning"));
}

fn add_skills(
    mut commands: Commands,
    q_riven: Query<Entity, (With<Riven>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_riven.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Riven/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Riven/Spells/RivenPassiveAbility/RivenPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            if index == 0 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
