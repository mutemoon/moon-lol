use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventSkillCast, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::PassiveSkillOf;
use crate::DamageType;

const LEESIN_Q_KEY: &str = "Characters/LeeSin/Spells/LeeSinQ/LeeSinQ";
const LEESIN_W_KEY: &str = "Characters/LeeSin/Spells/LeeSinW/LeeSinW";
const LEESIN_E_KEY: &str = "Characters/LeeSin/Spells/LeeSinE/LeeSinE";
const LEESIN_R_KEY: &str = "Characters/LeeSin/Spells/LeeSinR/LeeSinR";
const LEESIN_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginLeeSin;

impl Plugin for PluginLeeSin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_leesin_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("LeeSin"))]
#[reflect(Component)]
pub struct LeeSin;

fn on_leesin_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_leesin: Query<(), With<LeeSin>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_leesin.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_leesin_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_leesin_w(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::E => cast_leesin_e(&mut commands, entity, trigger.skill_entity, cooldown, recast),
        SkillSlot::R => cast_leesin_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_leesin_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell1"));

    if stage == 1 {
        // First cast: Sonic Wave - skillshot that marks enemy
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_Q_Cast"));
        skill_damage(
            commands,
            entity,
            LEESIN_Q_KEY,
            DamageShape::Sector {
                radius: 400.0,
                angle: 30.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("LeeSin_Q_Hit")),
        );
        // Insert recast window for second cast (Resonating Strike)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Resonating Strike - dash to marked enemy
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_Q2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: LEESIN_Q_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 500.0 },
                damage: Some(crate::DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 800.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Lee Sin Q", stage);
    }
}

fn cast_leesin_w(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell2"));

    if stage == 1 {
        // First cast: Safeguard - dash to ally/windwall
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_W_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: LEESIN_W_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 300.0 },
                damage: None,
                speed: 700.0,
            },
        );
        // Insert recast window for second cast (Iron Will)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Iron Will - lifesteal and attack speed buff
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_W2_Cast"));
        // Iron Will doesn't deal damage, just grants lifesteal and attack speed
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Lee Sin W", stage);
    }
}

fn cast_leesin_e(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Tempest - AoE damage and slow
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_E_Cast"));
        skill_damage(
            commands,
            entity,
            LEESIN_E_KEY,
            DamageShape::Circle { radius: 250.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("LeeSin_E_Hit")),
        );
        // Insert recast window for second cast (Cripple)
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, LEESIN_RECAST_WINDOW));
    } else {
        // Second cast: Cripple - slow enemies already affected by Tempest
        spawn_skill_particle(commands, entity, hash_bin("LeeSin_E2_Cast"));
        // TODO: Add cripple debuff (movement speed reduction)
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Lee Sin E", stage);
    }
}

fn cast_leesin_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("LeeSin_R_Cast"));
    // R is a knockback that hits enemy into terrain
    skill_damage(
        commands,
        entity,
        LEESIN_R_KEY,
        DamageShape::Nearest {
            max_distance: 150.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("LeeSin_R_Hit")),
    );
    // TODO: Knockback enemy into terrain and stun
}

fn add_skills(
    mut commands: Commands,
    q_leesin: Query<Entity, (With<LeeSin>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_leesin.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/LeeSin/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/LeeSin/Spells/LeeSinPassiveAbility/LeeSinPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // Q, W, E all use manual cooldown mode for recast windows
            if index < 3 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
