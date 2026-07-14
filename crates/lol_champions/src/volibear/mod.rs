pub mod buffs;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod p_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod r_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod w_tests;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::attack::{Attack, CommandAttackReset, EventAttackEnd};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitStun};
use lol_core::buffs::shield_white::BuffShieldWhite;
use lol_core::damage::{AbilityPower, CommandDamageCreate, Damage, DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::life::Health;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, delay_from_cast_frame,
    get_skill_cast_radius, get_skill_data_value, get_skill_value,
};
use lol_core::team::Team;

use crate::volibear::buffs::{DebuffVolibearWMark, VolibearPStacks};

// ── 被动（风暴之力）── 普攻叠加层数（上限 5），每层 +5% 攻速；满层触发连锁闪电；
//     脱战 6s 清零。
pub const VOLIBEAR_P_ATTACK_SPEED_PER_STACK: f32 = 0.05;
pub const VOLIBEAR_P_AP_RATIO: f32 = 0.45;
pub const VOLIBEAR_P_AD_RATIO: f32 = 0.2;
pub const VOLIBEAR_P_CHAIN_RADIUS: f32 = 300.0;

// ── W（疯狂撕咬）── W1 咬最近敌人 + 标记 8s；W2 重施对已标记目标 1.5x 伤害 + 治疗。
pub const VOLIBEAR_W_TAG: u32 = 1;
pub const VOLIBEAR_W_RECAST_WINDOW: f32 = 2.0;
pub const VOLIBEAR_W_MARK_DURATION: f32 = 8.0;

// ── E（落雷）── 地面靶向延迟 AoE 魔法伤害 + 减速（减速由 on_volibear_damage_hit 施加）。
pub const VOLIBEAR_E_SLOW_PERCENT: f32 = 0.4;
pub const VOLIBEAR_E_SLOW_DURATION: f32 = 2.0;

// ── R（风暴之怒）── 突进 + 落地 AoE 物理伤害 + 减速 + 增加最大生命。
pub const VOLIBEAR_R_TAG: u32 = 2;
pub const VOLIBEAR_R_LANDING_RADIUS: f32 = 300.0;
pub const VOLIBEAR_R_MAX_RANGE: f32 = 550.0;
pub const VOLIBEAR_R_DASH_SPEED: f32 = 750.0;

#[derive(Default)]
pub struct PluginVolibear;

impl Plugin for PluginVolibear {
    fn build(&self, app: &mut App) {
        app.add_observer(on_volibear_q);
        app.add_observer(on_volibear_w);
        app.add_observer(on_volibear_e);
        app.add_observer(on_volibear_r);
        app.add_observer(on_volibear_damage_hit);
        app.add_observer(on_volibear_attack_end);
        app.add_systems(FixedUpdate, update_volibear_p);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Volibear"), VolibearPStacks)]
#[reflect(Component)]
pub struct Volibear;

// ════════════════════════ Q ════════════════════════

fn on_volibear_q(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::Q) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    cast_volibear_q(&mut commands, entity, skill.level, spell_obj, ad);
}

/// Q：强化下次普攻--额外物理伤害（calculated_damage）+ 眩晕 + 移速。
fn cast_volibear_q(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    ad: f32,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAttackReset { entity });

    let bonus = get_skill_value(spell_obj, "calculated_damage", skill_level, |stat| {
        if stat == 2 { ad } else { 0.0 }
    })
    .unwrap_or(0.0);
    let ms_bonus = get_skill_data_value(spell_obj, "MaxSpeed", skill_level).unwrap_or(0.17);
    let duration = get_skill_data_value(spell_obj, "Duration", skill_level).unwrap_or(4.0);
    let stun_duration = get_skill_data_value(spell_obj, "StunDuration", skill_level).unwrap_or(1.0);

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOnHitCounter::new(1, duration))
        .with_related::<BuffOf>(BuffOnHitBonusDamage {
            flat: bonus,
            ratio: 0.0,
        })
        .with_related::<BuffOf>(BuffOnHitStun {
            duration: stun_duration,
        })
        .with_related::<BuffOf>(BuffMoveSpeed::new(ms_bonus, duration));
}

// ════════════════════════ W ════════════════════════

fn on_volibear_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_transform: Query<&Transform>,
    q_team: Query<&Team>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_mark: Query<&DebuffVolibearWMark>,
    q_health: Query<&Health>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let caster_pos = q_transform
        .get(entity)
        .map(|t| t.translation.xz())
        .unwrap_or(Vec2::ZERO);
    let Ok(caster_team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let max_hp = q_health.get(entity).map(|h| h.max).unwrap_or(0.0);
    let current_hp = q_health.get(entity).map(|h| h.value).unwrap_or(0.0);

    cast_volibear_w(
        &mut commands,
        entity,
        skill.level,
        spell_obj,
        trigger.skill_entity,
        cooldown,
        recast,
        caster_pos,
        caster_team,
        ad,
        max_hp,
        current_hp,
        &q_enemies,
        &q_team,
        &q_buffs,
        &q_mark,
    );
}

/// W：W1 咬最近敌人（total_damage 物理 + 标记）；W2 重施对已标记目标 1.5x 伤害 + 治疗。
fn cast_volibear_w(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    caster_pos: Vec2,
    caster_team: &Team,
    ad: f32,
    max_hp: f32,
    current_hp: f32,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
    q_buffs: &Query<&Buffs>,
    q_mark: &Query<&DebuffVolibearWMark>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    let stage = recast.map(|w| w.stage).unwrap_or(1);
    let total_damage = get_skill_value(spell_obj, "total_damage", skill_level, |stat| {
        if stat == 2 {
            ad
        } else if stat == 12 {
            max_hp
        } else {
            0.0
        }
    })
    .unwrap_or(0.0);
    let cast_range = get_skill_cast_radius(spell_obj, skill_level).unwrap_or(325.0);

    if stage == 1 {
        // W1：开启重施窗口 + 咬最近敌人 + 标记
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            VOLIBEAR_W_RECAST_WINDOW,
        ));
        if let Some(enemy) = nearest_enemy(caster_pos, cast_range, caster_team, q_enemies, q_team) {
            if total_damage > 0.0 {
                commands.entity(enemy).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: entity,
                    damage_type: DamageType::Physical,
                    amount: total_damage,
                    tag: Some(VOLIBEAR_W_TAG),
                });
            }
            // 直接标记（不依赖伤害命中 observer 的二次延迟）
            commands
                .entity(enemy)
                .with_related::<BuffOf>(DebuffVolibearWMark::new(entity, VOLIBEAR_W_MARK_DURATION));
        }
    } else {
        // W2：移除重施 + 重置冷却 + 咬最近敌人（已标记则 1.5x + 治疗）
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        });

        let w2_mult =
            get_skill_data_value(spell_obj, "W2DamageMultiplier", skill_level).unwrap_or(1.5);
        let base_heal = get_skill_data_value(spell_obj, "BaseHeal", skill_level).unwrap_or(5.0);
        let heal_percent =
            get_skill_data_value(spell_obj, "HealPercent", skill_level).unwrap_or(0.05);

        if let Some(enemy) = nearest_enemy(caster_pos, cast_range, caster_team, q_enemies, q_team) {
            let marked = is_marked_by(enemy, entity, q_buffs, q_mark);
            let dmg = total_damage * if marked { w2_mult } else { 1.0 };
            if dmg > 0.0 {
                commands.entity(enemy).trigger(|e| CommandDamageCreate {
                    entity: e,
                    source: entity,
                    damage_type: DamageType::Physical,
                    amount: dmg,
                    tag: Some(VOLIBEAR_W_TAG),
                });
            }
            // 仅对已标记目标治疗（BaseHeal + HealPercent * 已损生命）
            if marked {
                let missing = (max_hp - current_hp).max(0.0);
                let heal = base_heal + heal_percent * missing;
                if heal > 0.0 {
                    commands.entity(entity).queue(move |mut e: EntityWorldMut| {
                        if let Some(mut health) = e.get_mut::<Health>() {
                            health.value = (health.value + heal).min(health.max);
                        }
                    });
                }
            }
        }
    }
}

// ════════════════════════ E ════════════════════════

fn on_volibear_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    cast_volibear_e(
        &mut commands,
        entity,
        skill.spell.clone(),
        skill.level,
        spell_obj,
        trigger.point,
    );
}

/// E：地面靶向延迟 AoE（落雷）--施法点为圆心，延迟后造成魔法伤害 + 落地护盾。
///    减速由 on_volibear_damage_hit 在伤害结算时施加（验证延迟伤害仍触发 observer）。
fn cast_volibear_e(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    let radius = get_skill_cast_radius(spell_obj, skill_level).unwrap_or(325.0);

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill_spell,
        skill_level,
        delay: delay_from_cast_frame(spell_obj),
        point,
        origin: AoEOrigin::CastPoint,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "calculated_damage".to_string(),
                damage_type: DamageType::Magic,
                ..Default::default()
            }],
            ..Default::default()
        }],
        indicator: AoEIndicator {
            color: Color::srgba(0.4, 0.6, 1.0, 0.4),
            pulse: false,
            grow_from_zero: true,
            impact_burst_scale: 1.5,
            fade_duration: 0.3,
        },
    });

    // 落地护盾
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
}

// ════════════════════════ R ════════════════════════

fn on_volibear_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_skill: Query<&Skill>,
    q_enemies: Query<(Entity, &Transform), With<Champion>>,
    q_team: Query<&Team>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }
    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    cast_volibear_r(
        &mut commands,
        entity,
        skill.level,
        spell_obj,
        trigger.point,
        &q_enemies,
        &q_team,
        &q_damage,
    );
}

/// R：向指针方向突进（最大 550），落地 AoE 物理伤害 + 减速 + 增加最大生命。
fn cast_volibear_r(
    commands: &mut Commands,
    entity: Entity,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
    q_damage: &Query<&Damage>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // 突进（位移简化：直接冲向指针点，最大 550）
    commands.trigger(ActionDash {
        entity,
        point,
        move_type: DashMoveType::Pointer {
            max: VOLIBEAR_R_MAX_RANGE,
        },
        speed: VOLIBEAR_R_DASH_SPEED,
    });

    let Ok(team) = q_team.get(entity) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let damage = get_skill_value(
        spell_obj,
        "sweet_spot_damage_tooltip",
        skill_level,
        |stat| {
            if stat == 2 { ad } else { 0.0 }
        },
    )
    .unwrap_or(0.0);
    let slow_percent = get_skill_data_value(spell_obj, "SlowAmount", skill_level).unwrap_or(0.5);
    let slow_duration = get_skill_data_value(spell_obj, "SlowDuration", skill_level).unwrap_or(1.0);

    // 落地 AoE（以施法点为圆心）
    let mut hit = 0u32;
    for (enemy, enemy_transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == team {
            continue;
        }
        let dist = enemy_transform.translation.xz().distance(point);
        if dist > VOLIBEAR_R_LANDING_RADIUS {
            continue;
        }
        if damage > 0.0 {
            commands.entity(enemy).trigger(|e| CommandDamageCreate {
                entity: e,
                source: entity,
                damage_type: DamageType::Physical,
                amount: damage,
                tag: Some(VOLIBEAR_R_TAG),
            });
        }
        commands
            .entity(enemy)
            .with_related::<BuffOf>(DebuffSlow::new(slow_percent, slow_duration));
        hit += 1;
    }

    // 增加最大生命（HealthAmount，当前生命同步增加）
    let bonus_hp = get_skill_data_value(spell_obj, "HealthAmount", skill_level).unwrap_or(0.0);
    if bonus_hp > 0.0 {
        commands.entity(entity).queue(move |mut e: EntityWorldMut| {
            if let Some(mut health) = e.get_mut::<Health>() {
                health.max += bonus_hp;
                health.value = (health.value + bonus_hp).min(health.max);
            }
        });
    }

    debug!(
        "Volibear R: 风暴之怒，突进并砸中 {} 个敌人 + 减速 + 增加生命",
        hit
    );
}

// ════════════════════════ 被动 / 标签副作用 ════════════════════════

/// 监听 Volibear 造成的伤害：仅 None（E 延迟伤害 / Q 强化普攻）施加减速。
/// W（W_TAG）标记在 cast_volibear_w 直接施加；R（R_TAG）减速在 cast_volibear_r 直接施加。
fn on_volibear_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
) {
    let source = trigger.source;
    if q_volibear.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    match trigger.event().tag {
        None => {
            commands
                .entity(target)
                .with_related::<BuffOf>(DebuffSlow::new(
                    VOLIBEAR_E_SLOW_PERCENT,
                    VOLIBEAR_E_SLOW_DURATION,
                ));
        }
        _ => {}
    }
}

/// 被动（风暴之力）：普攻命中叠加层数（上限 5），每层 +5% 攻速；满层触发连锁闪电。
fn on_volibear_attack_end(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    mut q_volibear: Query<
        (
            &mut VolibearPStacks,
            &mut Attack,
            &Transform,
            &Team,
            &Damage,
        ),
        With<Volibear>,
    >,
    q_ap: Query<&AbilityPower>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
) {
    let attacker = trigger.event_target();
    let Ok((mut stacks, mut attack, transform, team, damage)) = q_volibear.get_mut(attacker) else {
        return;
    };

    let new_count = (stacks.count + 1).min(VolibearPStacks::MAX);
    stacks.count = new_count;
    stacks.timer.reset();
    attack.buff_bonus_attack_speed = new_count as f32 * VOLIBEAR_P_ATTACK_SPEED_PER_STACK;

    // 满层触发连锁闪电（附近敌人受魔法伤害）
    if new_count >= VolibearPStacks::MAX {
        let ad = damage.0;
        let ap = q_ap.get(attacker).map(|a| a.0).unwrap_or(0.0);
        let chain_dmg = VOLIBEAR_P_AP_RATIO * ap + VOLIBEAR_P_AD_RATIO * ad;
        if chain_dmg > 0.0 {
            let pos = transform.translation.xz();
            for (enemy, enemy_transform, enemy_team) in q_enemies.iter() {
                if enemy_team == team {
                    continue;
                }
                let dist = enemy_transform.translation.xz().distance(pos);
                if dist <= VOLIBEAR_P_CHAIN_RADIUS {
                    commands.entity(enemy).trigger(|e| CommandDamageCreate {
                        entity: e,
                        source: attacker,
                        damage_type: DamageType::Magic,
                        amount: chain_dmg,
                        tag: None,
                    });
                }
            }
        }
    }
}

/// 被动脱战：6s 计时器到期后清零层数与攻速加成。
fn update_volibear_p(
    time: Res<Time>,
    mut q_volibear: Query<(&mut VolibearPStacks, &mut Attack), With<Volibear>>,
) {
    for (mut stacks, mut attack) in q_volibear.iter_mut() {
        stacks.timer.tick(time.delta());
        if stacks.timer.just_finished() {
            stacks.count = 0;
            attack.buff_bonus_attack_speed = 0.0;
        }
    }
}

// ════════════════════════ 辅助 ════════════════════════

/// 在 caster 周围 range 内寻找最近的敌方英雄。
fn nearest_enemy(
    caster_pos: Vec2,
    range: f32,
    caster_team: &Team,
    q_enemies: &Query<(Entity, &Transform), With<Champion>>,
    q_team: &Query<&Team>,
) -> Option<Entity> {
    let mut best: Option<(Entity, f32)> = None;
    for (enemy, transform) in q_enemies.iter() {
        let Ok(enemy_team) = q_team.get(enemy) else {
            continue;
        };
        if enemy_team == caster_team {
            continue;
        }
        let dist = transform.translation.xz().distance(caster_pos);
        if dist <= range && best.map(|(_, d)| dist < d).unwrap_or(true) {
            best = Some((enemy, dist));
        }
    }
    best.map(|(e, _)| e)
}

/// enemy 是否挂有来自 source 的 W 标记。
fn is_marked_by(
    enemy: Entity,
    source: Entity,
    q_buffs: &Query<&Buffs>,
    q_mark: &Query<&DebuffVolibearWMark>,
) -> bool {
    let Ok(buffs) = q_buffs.get(enemy) else {
        return false;
    };
    buffs
        .iter()
        .find_map(|b| q_mark.get(b).ok())
        .map(|mark| mark.source == source)
        .unwrap_or(false)
}
