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
use crate::{BuffOf, BuffSelfHeal, PassiveSkillOf};
use crate::DamageType;

const SYLAS_Q_KEY: &str = "Characters/Sylas/Spells/SylasQ/SylasQ";
const SYLAS_W_KEY: &str = "Characters/Sylas/Spells/SylasW/SylasW";
const SYLAS_E_KEY: &str = "Characters/Sylas/Spells/SylasE/SylasE";
const SYLAS_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginSylas;

impl Plugin for PluginSylas {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_sylas_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sylas"))]
#[reflect(Component)]
pub struct Sylas;

fn on_sylas_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_sylas_q(&mut commands, entity),
        SkillSlot::W => cast_sylas_w(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::E => cast_sylas_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_sylas_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}

fn cast_sylas_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Sylas_Q_Cast"));
    // Q is a lash that slows enemies in the center
    skill_damage(
        commands,
        entity,
        SYLAS_Q_KEY,
        DamageShape::Sector { radius: 350.0, angle: 60.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Sylas_Q_Hit")),
    );
    debug!("{:?} 的技能 {} 应对目标施加 {}",
        entity, "Sylas Q", "减速 DebuffSlow");
}

fn cast_sylas_w(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Sylas_W_Cast"));
    // W is a dash to target that deals damage and heals based on missing health
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: SYLAS_W_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 200.0 },
            damage: Some(crate::DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 900.0,
        },
    );
    // Heal based on missing health
    commands.entity(entity).with_related::<BuffOf>(BuffSelfHeal::new(60.0));
}

fn cast_sylas_e(
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
        // First cast: Throws chain toward enemy - damage in narrow cone
        spawn_skill_particle(commands, entity, hash_bin("Sylas_E_Cast"));
        skill_damage(
            commands,
            entity,
            SYLAS_E_KEY,
            DamageShape::Sector {
                radius: 400.0,
                angle: 20.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            Some(hash_bin("Sylas_E_Hit")),
        );
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, SYLAS_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash to enemy and pull
        spawn_skill_particle(commands, entity, hash_bin("Sylas_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &crate::ActionDash {
                skill: SYLAS_E_KEY.into(),
                move_type: crate::DashMoveType::Pointer { max: 300.0 },
                damage: Some(crate::DashDamage {
                    radius_end: 100.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Magic,
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
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Sylas E", stage);
    }
}

fn cast_sylas_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Sylas_R_Cast"));
    // R is stolen from enemy - deals damage based on enemy's ultimate
    debug!("{:?} R 偷取目标大招", entity);
}

fn add_skills(
    mut commands: Commands,
    q_sylas: Query<Entity, (With<Sylas>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_sylas.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Sylas/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Sylas/Spells/SylasPassiveAbility/SylasPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // E uses manual cooldown mode for recast window
            if index == 2 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
