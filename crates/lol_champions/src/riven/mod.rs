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

#[derive(Default)]
pub struct PluginRiven;

impl Plugin for PluginRiven {
    fn build(&self, app: &mut App) {
        app.add_observer(q::on_riven_q);
        app.add_observer(w::on_riven_w);
        app.add_observer(e::on_riven_e);
        app.add_observer(r::on_riven_r);
        app.add_observer(passive::on_riven_skill_cast_charge_passive);
        app.add_observer(passive::on_damage_create_trigger_bonus);
        app.add_observer(q::on_riven_dash_end);
        app.add_systems(FixedUpdate, r::update_riven_buffs);
        app.add_systems(FixedUpdate, passive::update_riven_passive_timer);
        app.add_systems(FixedUpdate, buffs::update_shield_visuals);
        app.add_systems(FixedUpdate, buffs::cleanup_shield_visuals);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Riven"))]
#[reflect(Component)]
pub struct Riven;
