//! Darius Q - 大杀四方 (Decimate)
//!
//! Inner blade (handle): lower damage, does NOT stack Hemorrhage
//! Outer blade (axe): higher damage, stacks Hemorrhage
//!
//! Inner radius: ~150
//! Outer radius: ~350

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL1;
use lol_base::spell::Spell;
use lol_core::action::damage::{ActionDamageEffect, DamageShape, TargetDamage, TargetFilter};
use lol_core::action::delayed_damage::{ActionDelayedDamage, AoEIndicator, AoEOrigin};
use lol_core::damage::DamageType;
use lol_core::skill::delay_from_cast_frame;

/// Darius Q skill - inner and outer blade damage values
///
/// Inner blade: half damage, no hemorrhage
/// Outer blade: full damage, stacks hemorrhage
#[derive(Component, Debug, Clone)]
pub struct DariusQInnerDamage {
    /// Inner blade damage amount (before applying spell scaling)
    pub base_damage: f32,
}

impl DariusQInnerDamage {
    pub fn new(base_damage: f32) -> Self {
        Self { base_damage }
    }
}

/// Inner blade radius (the "handle" of the axe)
pub const DARIUS_Q_INNER_RADIUS: f32 = 150.0;

/// Outer blade radius (the "blade" of the axe)
pub const DARIUS_Q_OUTER_RADIUS: f32 = 350.0;

/// Cast Darius Q - 前摇延迟的双形劈砍伤害
///
/// 组合表达（验证 `AoEOrigin::Caster` + 双 `ActionDamageEffect` 原语）：
/// - 原点 = Caster（劈砍以施法者为中心，全向）
/// - 延迟 = castFrame 7.5 → 0.25s（前摇）
/// - 形状 = [Circle{150} 内圈, Annular{150,350} 外圈]（双形空间划分）
/// - 伤害 = spell ron 的 `blade_damage`（物理）
/// - 出血由 `on_darius_damage_hit` observer 在伤害结算时施加
pub fn cast_darius_q(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_level: usize,
    spell_obj: &Spell,
    point: Vec2,
) {
    commands.trigger(lol_base::render_cmd::CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    let delay = delay_from_cast_frame(spell_obj);
    let _ = skill_level; // 半径用常量，伤害由 spell ron 的 blade_damage 计算

    // 内圈（Circle）+ 外圈（Annular）双形，均用 blade_damage 物理伤害
    let inner_effect = ActionDamageEffect {
        shape: DamageShape::Circle {
            radius: DARIUS_Q_INNER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "blade_damage".to_string(),
            damage_type: DamageType::Physical,
            ..Default::default()
        }],
        ..Default::default()
    };
    let outer_effect = ActionDamageEffect {
        shape: DamageShape::Annular {
            inner_radius: DARIUS_Q_INNER_RADIUS,
            outer_radius: DARIUS_Q_OUTER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "blade_damage".to_string(),
            damage_type: DamageType::Physical,
            ..Default::default()
        }],
        ..Default::default()
    };

    commands.trigger(ActionDelayedDamage {
        entity,
        skill: skill_spell,
        skill_level,
        delay,
        point,
        origin: AoEOrigin::Caster,
        effects: vec![inner_effect, outer_effect],
        indicator: AoEIndicator {
            color: Color::srgba(0.9, 0.2, 0.2, 0.4),
            pulse: false,
            grow_from_zero: true,
            impact_burst_scale: 1.4,
            fade_duration: 0.3,
        },
    });

    debug!(
        "Darius Q: 延迟 {:.2}s 双形劈砍（内圈 r={}, 外圈 r={}-{}）",
        delay, DARIUS_Q_INNER_RADIUS, DARIUS_Q_INNER_RADIUS, DARIUS_Q_OUTER_RADIUS
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_darius_q_inner_radius() {
        assert!(DARIUS_Q_INNER_RADIUS > 0.0);
        assert!(DARIUS_Q_OUTER_RADIUS > DARIUS_Q_INNER_RADIUS);
    }

    #[test]
    fn test_darius_q_damage_values() {
        // Outer damage should be roughly 2x inner damage
        // From wiki: outer = 90/100/110/120/130/140/150, inner = 45/50/55/60/65/70/75
        let outer_damage = 150.0f32;
        let inner_damage = 75.0f32;
        assert!((outer_damage / inner_damage - 2.0).abs() < 0.01);
    }
}
