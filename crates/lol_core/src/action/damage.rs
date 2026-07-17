use bevy::prelude::*;
use lol_base::spell::Spell;

use crate::action::delayed_damage::DelayedDamageInstance;
use crate::damage::{AbilityPower, CommandDamageCreate, Damage, DamageType};
use crate::entities::champion::Champion;
use crate::entities::minion::Minion;
use crate::skill::{Skill, Skills, get_skill_data_value, get_skill_value};
use crate::team::Team;

#[derive(Debug, Clone)]
pub enum DamageShape {
    Circle {
        radius: f32,
    },
    Sector {
        radius: f32,
        angle: f32,
    },
    Annular {
        inner_radius: f32,
        outer_radius: f32,
    },
    Nearest {
        max_distance: f32,
    },
    /// 面朝方向的矩形：沿 forward 从 start_distance 延伸到 start_distance + length，左右各 width/2
    Rectangle {
        width: f32,
        length: f32,
        start_distance: f32,
    },
}

impl Default for DamageShape {
    fn default() -> Self {
        Self::Circle { radius: 300.0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum TargetFilter {
    #[default]
    All,
    Champion,
    Minion,
}

/// 单条伤害的修饰器：在伤害结算时按目标聚合状态调整
#[derive(Debug, Clone)]
pub enum DamageModifier {
    None,
    /// 孤立增伤：当该 effect 形状内（排除后）仅命中 1 个目标时，
    /// 从 spell data values 读取标量值乘算伤害（如铁男 Q 的 IsolationScalar）
    Isolation {
        scalar_data_value: String,
    },
}

impl Default for DamageModifier {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct TargetDamage {
    pub filter: TargetFilter,
    pub amount: String,
    pub damage_type: DamageType,
    pub modifier: DamageModifier,
}

#[derive(Debug, Clone, Default)]
pub struct ActionDamageEffect {
    pub shape: DamageShape,
    pub damage_list: Vec<TargetDamage>,
    /// 排除区：在此子形状内的目标跳过本 effect（用于空间分区，如万豪 W 中心/两边）
    pub exclude: Vec<DamageShape>,
    /// 伤害来源标记：透传至本 effect 所有 `CommandDamageCreate.tag`，供全局伤害
    /// 观察者区分一次伤害来自哪个技能/形状分区（如 Darius Q 内圈不叠层、仅外圈减速）。
    /// 粒度为 effect：同一 effect 内的多条 damage 共享同一来源标记。
    pub tag: Option<u32>,
}

#[derive(Debug, Clone, EntityEvent)]
pub struct ActionDamage {
    pub entity: Entity,
    pub skill: Handle<Spell>,
    pub effects: Vec<ActionDamageEffect>,
}

/// 判定目标是否在形状内（相对 origin/forward）
///
/// forward 为 XZ 平面单位方向向量（扇形/矩形以此定向）。
pub fn is_in_shape(target_pos: Vec3, origin: Vec3, forward: Vec2, shape: &DamageShape) -> bool {
    match shape {
        DamageShape::Circle { radius } => target_pos.distance(origin) <= *radius,
        DamageShape::Sector { radius, angle } => {
            let diff = (target_pos - origin).xz();
            let distance = diff.length();
            if distance > *radius {
                return false;
            }
            // 距离为 0（目标在施法者身上）视为命中扇形顶点
            if distance == 0.0 {
                return true;
            }
            let half_angle = angle.to_radians() / 2.0;
            let target_dir = diff.normalize();
            forward.dot(target_dir).acos() <= half_angle
        }
        DamageShape::Annular {
            inner_radius,
            outer_radius,
        } => {
            let d = target_pos.distance(origin);
            d >= *inner_radius && d <= *outer_radius
        }
        DamageShape::Nearest { max_distance } => target_pos.distance(origin) <= *max_distance,
        DamageShape::Rectangle {
            width,
            length,
            start_distance,
        } => {
            // 以 origin 为起点、forward 方向延伸的矩形（从 start_distance 到 start_distance+length）
            let diff = (target_pos - origin).xz();
            let along = forward.dot(diff); // 沿 forward 投影
            if along < *start_distance || along > *start_distance + *length {
                return false;
            }
            // forward 的左侧法向量 (-y, x)
            let lateral = -forward.y * diff.x + forward.x * diff.y;
            lateral.abs() <= *width / 2.0
        }
    }
}

/// 在形状内收集敌方目标（同队跳过）。Nearest 仅返回最近单体。
pub fn collect_targets_in_shape(
    origin: Vec3,
    forward: Vec2,
    shape: &DamageShape,
    team: &Team,
    q_target: &Query<
        (
            Entity,
            &Team,
            Option<&Champion>,
            Option<&Minion>,
            &Transform,
        ),
        Without<DelayedDamageInstance>,
    >,
) -> Vec<Entity> {
    let mut targets = Vec::new();
    match shape {
        DamageShape::Nearest { max_distance } => {
            let mut min_dist = *max_distance;
            let mut nearest = None;
            for (target, target_team, _, _, target_transform) in q_target.iter() {
                if target_team == team {
                    continue;
                }
                let distance = target_transform.translation.distance(origin);
                if distance < min_dist {
                    min_dist = distance;
                    nearest = Some(target);
                }
            }
            if let Some(target) = nearest {
                targets.push(target);
            }
        }
        _ => {
            for (target, target_team, _, _, target_transform) in q_target.iter() {
                if target_team == team {
                    continue;
                }
                if is_in_shape(target_transform.translation, origin, forward, shape) {
                    targets.push(target);
                }
            }
        }
    }
    targets
}

/// 对一组 effect 执行伤害结算（瞬发 ActionDamage 与延迟 DelayedDamageInstance 共用）。
///
/// - `origin`/`forward` 为伤害快照位置与朝向（延迟伤害为施法瞬间快照）
/// - 对每个 effect：先收集形状内目标，再剔除 exclude 区内目标，
///   最后对每条 damage 按 filter 与 modifier（Isolation 仅当唯一目标）结算
/// - 返回每个 effect 命中的目标列表（用于 AoE 命中报告）
pub fn apply_damage_effects(
    commands: &mut Commands,
    caster: Entity,
    origin: Vec3,
    forward: Vec2,
    team: &Team,
    effects: &[ActionDamageEffect],
    skill_object: &Spell,
    skill_level: usize,
    q_target: &Query<
        (
            Entity,
            &Team,
            Option<&Champion>,
            Option<&Minion>,
            &Transform,
        ),
        Without<DelayedDamageInstance>,
    >,
    q_damage: &Query<&Damage>,
    q_ap: &Query<&AbilityPower>,
) -> Vec<Vec<Entity>> {
    let mut hit_per_effect = Vec::with_capacity(effects.len());

    for effect in effects {
        let mut targets = collect_targets_in_shape(origin, forward, &effect.shape, team, q_target);

        // 排除区：在任意 exclude 子形状内的目标跳过本 effect
        if !effect.exclude.is_empty() {
            targets.retain(|t| {
                let Ok((_, _, _, _, tf)) = q_target.get(*t) else {
                    return true;
                };
                !effect
                    .exclude
                    .iter()
                    .any(|ex| is_in_shape(tf.translation, origin, forward, ex))
            });
        }

        let isolated = targets.len() == 1;

        // 记录本次 effect 实际收到伤害的目标
        let mut actually_hit: Vec<Entity> = Vec::new();

        for target_entity in &targets {
            let Ok((_, _, champion, minion, _)) = q_target.get(*target_entity) else {
                continue;
            };

            for damage in &effect.damage_list {
                let apply = match damage.filter {
                    TargetFilter::All => true,
                    TargetFilter::Champion => champion.is_some(),
                    TargetFilter::Minion => minion.is_some(),
                };
                if !apply {
                    continue;
                }

                let mut damage_amount =
                    get_skill_value(&skill_object, &damage.amount, skill_level, |stat| {
                        // stat==2 -> AD（物理攻击力），stat==0/None -> AP（法术强度）
                        if stat == 2 {
                            if let Ok(damage) = q_damage.get(caster) {
                                return damage.0;
                            }
                        }
                        if stat == 0 {
                            if let Ok(ap) = q_ap.get(caster) {
                                return ap.0;
                            }
                        }
                        0.0
                    })
                    .unwrap();

                // 修饰器：孤立增伤
                if let DamageModifier::Isolation { scalar_data_value } = &damage.modifier {
                    if isolated {
                        let scalar =
                            get_skill_data_value(skill_object, scalar_data_value, skill_level)
                                .unwrap_or(1.0);
                        damage_amount *= scalar;
                    }
                }

                commands
                    .entity(*target_entity)
                    .trigger(|e| CommandDamageCreate {
                        entity: e,
                        source: caster,
                        damage_type: damage.damage_type,
                        amount: damage_amount,
                        tag: effect.tag,
                    });

                if !actually_hit.contains(target_entity) {
                    actually_hit.push(*target_entity);
                }
            }
        }

        hit_per_effect.push(actually_hit);
    }

    hit_per_effect
}

pub fn on_action_damage(
    event: On<ActionDamage>,
    mut commands: Commands,
    res_assets_spell_object: Res<Assets<Spell>>,
    q_transform: Query<&Transform>,
    q_target: Query<
        (
            Entity,
            &Team,
            Option<&Champion>,
            Option<&Minion>,
            &Transform,
        ),
        Without<DelayedDamageInstance>,
    >,
    q_team: Query<&Team>,
    q_skills: Query<&Skills>,
    q_skill: Query<&Skill>,
    q_damage: Query<&Damage>,
    q_ap: Query<&AbilityPower>,
) {
    let entity = event.event_target();

    let Ok(team) = q_team.get(entity) else {
        return;
    };
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Some(skill_object) = res_assets_spell_object.get(&event.skill) else {
        return;
    };
    let Ok(skills) = q_skills.get(entity) else {
        return;
    };
    let skill = skills
        .iter()
        .filter_map(|v| q_skill.get(v).ok())
        .find(|s| s.spell == event.skill)
        .or_else(|| skills.iter().filter_map(|v| q_skill.get(v).ok()).next());

    let Some(skill) = skill else {
        return;
    };

    let origin = transform.translation;
    let forward = transform.forward().xz();

    let _ = apply_damage_effects(
        &mut commands,
        entity,
        origin,
        forward,
        team,
        &event.effects,
        skill_object,
        skill.level,
        &q_target,
        &q_damage,
        &q_ap,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORWARD_X: Vec2 = Vec2::new(1.0, 0.0);
    const ORIGIN: Vec3 = Vec3::ZERO;

    #[test]
    fn rectangle_includes_target_along_forward() {
        let shape = DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 0.0,
        };
        // 正前方 300 单位，居中
        assert!(is_in_shape(
            Vec3::new(300.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }

    #[test]
    fn rectangle_excludes_target_outside_length() {
        let shape = DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 0.0,
        };
        // 超出长度
        assert!(!is_in_shape(
            Vec3::new(700.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 身后
        assert!(!is_in_shape(
            Vec3::new(-50.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }

    #[test]
    fn rectangle_excludes_target_outside_width() {
        let shape = DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 0.0,
        };
        // 前方 300 但横向偏移 100（> 80 半宽）
        assert!(!is_in_shape(
            Vec3::new(300.0, 0.0, 100.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 横向偏移 70（< 80 半宽）应命中
        assert!(is_in_shape(
            Vec3::new(300.0, 0.0, 70.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }

    #[test]
    fn rectangle_respects_forward_rotation() {
        let shape = DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 0.0,
        };
        // forward 朝 +Z
        let forward_z = Vec2::new(0.0, 1.0);
        assert!(is_in_shape(
            Vec3::new(0.0, 0.0, 300.0),
            ORIGIN,
            forward_z,
            &shape,
        ));
        assert!(!is_in_shape(
            Vec3::new(300.0, 0.0, 0.0),
            ORIGIN,
            forward_z,
            &shape,
        ));
    }

    #[test]
    fn rectangle_respects_start_distance() {
        // 起始距离 400，长度 625 -> 命中区间 [400, 1025]
        let shape = DamageShape::Rectangle {
            width: 160.0,
            length: 625.0,
            start_distance: 400.0,
        };
        // 死区（< 400）不命中
        assert!(!is_in_shape(
            Vec3::new(300.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 区间内（600）命中
        assert!(is_in_shape(
            Vec3::new(600.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 超出（1100）不命中
        assert!(!is_in_shape(
            Vec3::new(1100.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }

    #[test]
    fn sector_includes_within_angle() {
        let shape = DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        };
        // 正前方
        assert!(is_in_shape(
            Vec3::new(200.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 边界 45° 内（扇形半角 45°）
        let edge = Vec3::new(200.0, 0.0, 200.0); // 45°
        assert!(is_in_shape(edge, ORIGIN, FORWARD_X, &shape));
    }

    #[test]
    fn sector_excludes_outside_angle_or_radius() {
        let shape = DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        };
        // 超出半径
        assert!(!is_in_shape(
            Vec3::new(400.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 角度过大（~90°，超出 45° 半角）
        assert!(!is_in_shape(
            Vec3::new(10.0, 0.0, 200.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }

    #[test]
    fn annular_excludes_inner_circle() {
        let shape = DamageShape::Annular {
            inner_radius: 150.0,
            outer_radius: 350.0,
        };
        // 内圈
        assert!(!is_in_shape(
            Vec3::new(100.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 环内
        assert!(is_in_shape(
            Vec3::new(250.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
        // 外圈外
        assert!(!is_in_shape(
            Vec3::new(400.0, 0.0, 0.0),
            ORIGIN,
            FORWARD_X,
            &shape,
        ));
    }
}
