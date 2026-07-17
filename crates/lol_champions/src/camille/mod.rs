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
pub struct PluginCamille;

impl Plugin for PluginCamille {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                passive::update_camille_passive,
                e::update_camille_e,
                r::update_camille_r_mark,
            ),
        );
        app.add_observer(q::on_camille_q);
        app.add_observer(w::on_camille_w);
        app.add_observer(e::on_camille_e);
        app.add_observer(e::on_camille_e_missile_hit);
        app.add_observer(e::on_camille_e_dash_end);
        app.add_observer(r::on_camille_r);
        app.add_observer(r::on_camille_r_arrival);
        app.add_observer(passive::on_camille_damage_hit);
        app.add_observer(passive::on_camille_attack_end);
        app.add_observer(r::on_camille_r_attack_end);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Camille"))]
#[reflect(Component)]
pub struct Camille;