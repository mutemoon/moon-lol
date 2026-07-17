//! Camille E（钩索 / Hookshot）完整两段式实现。
//!
//! - **E1**：朝目标方向发射粘性飞弹，碰墙后拉向墙壁并记录墙壁锚点。
//! - **E2**：从墙壁朝目标方向冲刺，命中英雄附加眩晕与伤害，并获得攻速加成。
//!
//! ## 流程
//! 1. `on_camille_e` E1：设置重施窗口 + 发射粘性飞弹
//! 2. `on_camille_e_missile_hit`：飞弹碰墙 → 挂 `BuffCamilleWallCling` + 拉向墙壁
//! 3. 冲刺碰墙完成 → 玩家可施放 E2
//! 4. `on_camille_e` E2：读取墙壁锚点 → 扫描路径上最近敌人 → 冲刺（实体/Pointer）+ 攻速
//! 5. `on_camille_e_dash_end`：E2 冲刺结束 → 对目标施加眩晕 + 伤害

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::action::displace::{
    ActionDisplace, DisplaceEffect, DisplaceMotion, DisplaceTargetSelection,
};
use lol_core::attack::BuffAttack;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::damage::{Damage, DamageType};
use lol_core::life::Death;
use lol_core::missile::{CommandMissileCreate, EventMissileHit, MissileCollisionTarget};
use lol_core::movement::{EventMovementEnd, MovementSource};
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value,
    get_skill_value,
};
use lol_core::team::Team;

use crate::camille::buffs::{BuffCamilleWallCling, CamilleE2State};
use crate::camille::Camille;

// ── 常量 ──

const CAMILLE_E_RECAST_WINDOW: f32 = 1.0;
const CAMILLE_E_MISSILE_SPEED: f32 = 1400.0;
const CAMILLE_E_DASH_SPEED: f32 = 1050.0;
const CAMILLE_E2_STUN_DURATION: f32 = 0.75;

/// E 攻速加成持续时间（ron ASDuration = 5s）。
pub const CAMILLE_E_AS_DURATION: f32 = 5.0;

// ── 组件 ──

/// E 攻速加成计时器：到期回收 `BuffAttack`。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleE" })]
pub struct BuffCamilleE {
    pub timer: Timer,
}

impl BuffCamilleE {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

// ── 辅助函数 ──

/// 施放 E2 攻速加成 + 计时 buff。
pub fn apply_camille_e_as(commands: &mut Commands, entity: Entity, as_percent: f32, duration: f32) {
    commands.entity(entity).insert(BuffAttack {
        bonus_attack_speed: as_percent,
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCamilleE::new(duration));
}

/// 读取技能数据中的命名数值。
fn read_e_data(spell_obj: &Spell, level: usize, name: &str, default: f32) -> f32 {
    get_skill_data_value(spell_obj, name, level).unwrap_or(default)
}

// ── 主观察者：E 技能施放 ──

pub fn on_camille_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_damage: Query<&Damage>,
    q_wall_cling_buff: Query<(Entity, &BuffOf, &BuffCamilleWallCling)>,
    q_enemies: Query<
        (Entity, &Team, &Transform),
        (Without<Camille>, Without<Death>),
    >,
    q_team: Query<&Team>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;

    let as_buff = read_e_data(spell_obj, level, "ASBuff", 0.35);
    let as_duration = read_e_data(spell_obj, level, "ASDuration", 5.0);
    let e2_long_range = read_e_data(spell_obj, level, "E2LongDashRange", 800.0);
    let e2_short_range = read_e_data(spell_obj, level, "E2ShortDashRange", 400.0);
    let e2_collision_range = read_e_data(spell_obj, level, "ECollisionRange", 130.0);
    let dash_speed = read_e_data(spell_obj, level, "DashSpeed", CAMILLE_E_DASH_SPEED);

    // 播放动画
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let stage = recast.map(|w| w.stage).unwrap_or(1);

    if stage == 1 {
        // ── E1：发射粘性飞弹 + 重施窗口 ──
        commands.trigger(CommandMissileCreate {
            entity,
            target: None,
            destination: Some(Vec3::new(trigger.point.x, 0.0, trigger.point.y)),
            spell: skill.spell.clone(),
            damage: 0.0,
            speed: Some(CAMILLE_E_MISSILE_SPEED),
            particle_hash: None,
            sticky: true,
            pass_through: false,
            collision_target: MissileCollisionTarget::WallOnly,
            missing_hp_scaling: None,
        });

        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
    } else {
        // ── E2：从墙壁冲刺 ──

        // 读取墙壁锚点位置（若无锚点则用 Champion 当前位置）
        let wall_cling = q_wall_cling_buff.iter().next();
        let origin = wall_cling
            .as_ref()
            .map(|(_, _, cling)| cling.wall_point)
            .unwrap_or_else(|| {
                q_transform
                    .get(entity)
                    .map(|t| t.translation)
                    .unwrap_or_default()
            });
        let cast_target = Vec3::new(trigger.point.x, 0.0, trigger.point.y);
        let cast_dir = (cast_target - origin).xz().normalize_or_zero();

        // 寻找路径上最近的敌方英雄
        let Ok(team) = q_team.get(entity) else {
            return;
        };
        let mut nearest_target: Option<(Entity, f32)> = None;

        for (enemy, enemy_team, enemy_tf) in q_enemies.iter() {
            if *enemy_team == *team {
                continue;
            }
            let to_enemy = enemy_tf.translation - origin;
            let projection = to_enemy.xz().dot(cast_dir);
            if projection <= 0.0 || projection > e2_long_range {
                continue;
            }
            let perpendicular = (to_enemy.xz() - cast_dir * projection).length();
            if perpendicular <= e2_collision_range {
                if nearest_target.map_or(true, |(_, d)| projection < d) {
                    nearest_target = Some((enemy, projection));
                }
            }
        }

        // 读取伤害值
        let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
        let e2_damage =
            get_skill_value(spell_obj, "total_damage", level, |stat| {
                if stat == 2 {
                    ad
                } else {
                    0.0
                }
            })
            .unwrap_or(0.0);

        // 设置冲刺类型与状态
        let target_entity = nearest_target.map(|(e, _)| e);
        let move_type = if let Some(target) = target_entity {
            commands.entity(entity).insert(CamilleE2State {
                target: Some(target),
                stun_duration: CAMILLE_E2_STUN_DURATION,
                damage: e2_damage,
            });
            DashMoveType::Entity {
                target,
                stop_radius: e2_collision_range,
            }
        } else {
            DashMoveType::Pointer {
                max: e2_short_range,
            }
        };

        // 冲刺沿途伤害
        commands.entity(entity).insert(DashDamageIntent {
            damage: DashDamage {
                radius_end: 150.0,
                damage: lol_core::action::damage::TargetDamage {
                    filter: lol_core::action::damage::TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: DamageType::Physical,
                    ..Default::default()
                },
            },
            skill: skill.spell.clone(),
        });

        // 触发冲刺
        commands.trigger(ActionDash {
            entity,
            point: trigger.point,
            move_type,
            speed: dash_speed,
        });

        // 攻击速度加成
        apply_camille_e_as(&mut commands, entity, as_buff, as_duration);

        // 移除墙壁锚点（子实体）
        for (e, _, _) in q_wall_cling_buff.iter() {
            commands.entity(e).despawn();
        }

        // 清理重施窗口并设置冷却
        commands
            .entity(trigger.skill_entity)
            .remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert((
            CoolDown {
                duration: cooldown.duration,
                timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
            },
        ));
    }
}

// ── 观察者：粘性飞弹碰墙 ──

/// E1 粘性飞弹碰墙：挂墙壁锚点 + 拉向墙壁。
pub fn on_camille_e_missile_hit(
    trigger: On<EventMissileHit>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let wall_point = trigger.point;

    // 挂墙壁锚点
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCamilleWallCling { wall_point });

    // 拉向墙壁
    commands.trigger(ActionDash {
        entity,
        point: wall_point.xz(),
        move_type: DashMoveType::WorldPoint(wall_point.xz()),
        speed: CAMILLE_E_DASH_SPEED,
    });
}

// ── 观察者：冲刺结束（E2 落地） ──

/// E2 冲刺结束：对目标施加眩晕 + 伤害。
pub fn on_camille_e_dash_end(
    trigger: On<EventMovementEnd>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_e2_state: Query<&CamilleE2State>,
) {
    if trigger.event().source != MovementSource::Dash {
        return;
    }

    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok(e2_state) = q_e2_state.get(entity) else {
        return;
    };

    // 对目标施加眩晕 + 伤害
    if let Some(target) = e2_state.target {
        commands.trigger(ActionDisplace {
            entity,
            targets: DisplaceTargetSelection::Explicit(vec![target]),
            motion: DisplaceMotion::None,
            effects: vec![
                DisplaceEffect::Stun {
                    duration: e2_state.stun_duration,
                },
                DisplaceEffect::Damage {
                    amount: e2_state.damage,
                    damage_type: DamageType::Physical,
                    tag: None,
                },
            ],
            cone_hit_policy: None,
        });
    }

    commands.entity(entity).remove::<CamilleE2State>();
}

// ── 系统：攻速计时 ──

/// E 攻速计时：到期移除 `BuffAttack` 与计时 buff。
pub fn update_camille_e(
    mut commands: Commands,
    mut q: Query<(Entity, &BuffOf, &mut BuffCamilleE)>,
    time: Res<Time<Fixed>>,
) {
    for (e, bo, mut buff) in q.iter_mut() {
        buff.timer.tick(time.delta());
        if !buff.timer.is_finished() {
            continue;
        }
        commands.entity(bo.0).remove::<BuffAttack>();
        commands.entity(e).despawn();
    }
}