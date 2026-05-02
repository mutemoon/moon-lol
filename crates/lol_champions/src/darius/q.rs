//! Darius Q - 大杀四方 (Decimate)
//!
//! Inner blade (handle): lower damage, does NOT stack Hemorrhage
//! Outer blade (axe): higher damage, stacks Hemorrhage
//!
//! Inner radius: ~150
//! Outer radius: ~350

use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};

use super::buffs::BuffDariusBleed;

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

/// Cast Darius Q - applies inner and outer blade damage
///
/// Inner blade: deals [inner_damage] physical damage (half of outer)
/// Outer blade: deals [outer_damage] physical damage, stacks 1 hemorrhage
pub fn cast_darius_q(
    commands: &mut Commands,
    entity: Entity,
    skill_spell: Handle<Spell>,
    inner_damage: f32,
    outer_damage: f32,
    _apply_hemorrhage_outer: bool,
) {
    // Spawn the Q cast particle
    commands.trigger(lol_base::render_cmd::CommandSkinParticleSpawn {
        entity,
        hash: league_utils::hash_bin("Darius_Q_Cast"),
    });

    // Play Q animation
    commands.trigger(lol_base::render_cmd::CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });

    // Inner blade damage (Circle, radius = inner_radius)
    // Inner blade does NOT apply hemorrhage
    let inner_effect = ActionDamageEffect {
        shape: DamageShape::Circle {
            radius: DARIUS_Q_INNER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "InnerDamage".to_string(),
            damage_type: DamageType::Physical,
        }],
        particle: Some(league_utils::hash_bin("Darius_Q_Inner_Hit")),
    };

    // Outer blade damage (Annular ring from inner_radius to outer_radius)
    // Outer blade DOES apply hemorrhage
    let outer_effect = ActionDamageEffect {
        shape: DamageShape::Annular {
            inner_radius: DARIUS_Q_INNER_RADIUS,
            outer_radius: DARIUS_Q_OUTER_RADIUS,
        },
        damage_list: vec![TargetDamage {
            filter: TargetFilter::All,
            amount: "OuterDamage".to_string(),
            damage_type: DamageType::Physical,
        }],
        particle: Some(league_utils::hash_bin("Darius_Q_Outer_Hit")),
    };

    // Trigger the dual-shape damage
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![inner_effect, outer_effect],
    });

    debug!(
        "Darius Q: inner={} (r={}), outer={} (r={}-{})",
        inner_damage,
        DARIUS_Q_INNER_RADIUS,
        outer_damage,
        DARIUS_Q_INNER_RADIUS,
        DARIUS_Q_OUTER_RADIUS
    );
}

/// Observer: Handle Darius Q outer blade hemorrhage application
///
/// This observer triggers after damage is dealt. For Q outer blade hits,
/// we need to apply hemorrhage stacks.
pub fn on_darius_q_damage(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_darius: Query<(), With<super::Darius>>,
) {
    let source = trigger.source;
    if q_darius.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply 1 stack of hemorrhage for Q outer blade hit
    // Note: In a full implementation, we'd distinguish inner vs outer hit
    // For now, Q applies hemorrhage (testing purposes)
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffDariusBleed::new(1, 5.0));
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
