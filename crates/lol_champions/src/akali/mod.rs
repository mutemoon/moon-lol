pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::akali::buffs::{BuffAkaliPassive, BuffAkaliStealth, BuffAkaliW};

const AKALI_Q_KEY: &str = "Characters/Akali/Spells/AkaliFivePointStrike/AkaliFivePointStrike";
#[allow(dead_code)]
const AKALI_W_KEY: &str = "Characters/Akali/Spells/AkaliSmokeBomb/AkaliSmokeBomb";
const AKALI_E_KEY: &str = "Characters/Akali/Spells/AkaliShurikenFlip/AkaliShurikenFlip";
const AKALI_R_KEY: &str = "Characters/Akali/Spells/AkaliShadowDance/AkaliShadowDance";

#[derive(Default)]
pub struct PluginAkali;

impl Plugin for PluginAkali {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_akali_skill_cast);
        app.add_observer(on_akali_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Akali"))]
#[reflect(Component)]
pub struct Akali;

fn on_akali_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_akali.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_akali_q(&mut commands, entity),
        SkillSlot::W => cast_akali_w(&mut commands, entity),
        SkillSlot::E => cast_akali_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_akali_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        _ => {}
    }
}

fn cast_akali_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Akali_Q_Cast"));

    // Q is a cone damage that slows distant enemies
    skill_damage(
        commands,
        entity,
        AKALI_Q_KEY,
        DamageShape::Sector {
            radius: 500.0,
            angle: 45.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Akali_Q_Hit")),
    );

    // Mark for passive ring
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliPassive::new());
}

fn cast_akali_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Akali_W_Cast"));

    // W drops a smoke bomb and grants stealth and move speed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliW::new());
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAkaliStealth::new());
}

fn cast_akali_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: throw shuriken and mark first enemy
        spawn_skill_particle(commands, entity, hash_bin("Akali_E_Cast"));
        skill_damage(
            commands,
            entity,
            AKALI_E_KEY,
            DamageShape::Sector {
                radius: 825.0,
                angle: 45.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            Some(hash_bin("Akali_E_Hit")),
        );
        // Mark for recast
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, 16.0));
    } else {
        // Second cast: dash to marked target
        spawn_skill_particle(commands, entity, hash_bin("Akali_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: AKALI_E_KEY.into(),
                move_type: DashMoveType::Pointer { max: 825.0 },
                damage: Some(DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Magic,
                    },
                }),
                speed: 1200.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    }
}

fn cast_akali_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell4"));

    if stage == 1 {
        // First cast: dash to target
        spawn_skill_particle(commands, entity, hash_bin("Akali_R_Cast"));
    } else {
        // Second cast: execute damage based on missing health
        spawn_skill_particle(commands, entity, hash_bin("Akali_R2_Cast"));
        skill_damage(
            commands,
            entity,
            AKALI_R_KEY,
            DamageShape::Circle { radius: 300.0 },
            vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            Some(hash_bin("Akali_R2_Hit")),
        );
    }

    // R is a dash that can be recast
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: AKALI_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 675.0 },
            damage: if stage == 1 {
                Some(DashDamage {
                    radius_end: 150.0,
                    damage: TargetDamage {
                        filter: TargetFilter::Champion,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Magic,
                    },
                })
            } else {
                None
            },
            speed: 900.0,
        },
    );

    if stage >= 2 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    } else {
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, 10.0));
    }
}

/// Listen for Akali damage hits
fn on_akali_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_akali: Query<(), With<Akali>>,
) {
    let source = trigger.source;
    if q_akali.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q slows distant enemies by 50%
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.0));
}

fn add_skills(
    mut commands: Commands,
    q_akali: Query<Entity, (With<Akali>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_akali.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Akali/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Akali/Spells/AkaliPassive/AkaliPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // E and R use manual cooldown mode for recast windows
            if index == 2 || index == 3 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
