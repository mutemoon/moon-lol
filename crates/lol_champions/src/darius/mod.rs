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
mod tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;

#[derive(Default)]
pub struct PluginDarius;

impl Plugin for PluginDarius {
    fn build(&self, app: &mut App) {
        app.add_observer(q::on_darius_q);
        app.add_observer(w::on_darius_w);
        app.add_observer(e::on_darius_e);
        app.add_observer(r::on_darius_r);
        app.add_observer(passive::on_darius_damage_hit);
        app.add_systems(FixedUpdate, passive::update_darius_bleed);
        app.add_systems(FixedUpdate, passive::update_darius_might);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Darius"))]
#[reflect(Component)]
pub struct Darius;