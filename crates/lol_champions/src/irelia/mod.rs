pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod e_tests;
#[cfg(test)]
mod passive_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod r_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod w_tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;

// ── 伤害标签：区分 Irelia 不同伤害来源，供 on_irelia_damage_hit 识别 ──
pub const IRELIA_Q_DAMAGE_TAG: u32 = 1;
pub const IRELIA_E2_DAMAGE_TAG: u32 = 2;
pub const IRELIA_R_DAMAGE_TAG: u32 = 3;

// ── Q ──
/// Q 命中判定距离（ron `castRange`）
pub const IRELIA_Q_RANGE: f32 = 600.0;

// ── E2 命中附加 ──
/// E2 眩晕时长（ron `StunDuration`）
pub const IRELIA_E_STUN_DURATION: f32 = 0.75;
/// E2/R 不稳标记时长
pub const IRELIA_MARK_DURATION: f32 = 5.0;

// ── R 命中附加 ──
/// R 减速比例（ron `SlowAmount` 90%）
pub const IRELIA_R_SLOW_PERCENT: f32 = 0.9;
/// R 减速时长（ron `CCDuration`）
pub const IRELIA_R_SLOW_DURATION: f32 = 1.5;

#[derive(Default)]
pub struct PluginIrelia;

impl Plugin for PluginIrelia {
    fn build(&self, app: &mut App) {
        app.add_observer(q::on_irelia_q);
        app.add_observer(w::on_irelia_w);
        app.add_observer(e::on_irelia_e);
        app.add_observer(r::on_irelia_r);
        app.add_observer(passive::on_irelia_skill_cast_stack_passive);
        app.add_observer(passive::on_irelia_passive_attack_end);
        app.add_observer(passive::on_irelia_damage_hit);
        app.add_systems(FixedUpdate, buffs::update_irelia_unsteady);
        app.add_systems(FixedUpdate, passive::update_irelia_fervor);
        app.add_systems(FixedUpdate, w::update_irelia_w);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Irelia"))]
#[reflect(Component)]
pub struct Irelia;
