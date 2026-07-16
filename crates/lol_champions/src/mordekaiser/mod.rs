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

use crate::mordekaiser::e::on_mordekaiser_e;
use crate::mordekaiser::passive::{
    on_mordekaiser_damage_hit, on_mordekaiser_damage_taken, update_mordekaiser_passive,
};
use crate::mordekaiser::q::on_mordekaiser_q;
use crate::mordekaiser::r::{on_mordekaiser_r, update_mordekaiser_realm};
use crate::mordekaiser::w::{on_mordekaiser_w, update_mordekaiser_w_shield};

#[derive(Default)]
pub struct PluginMordekaiser;

impl Plugin for PluginMordekaiser {
    fn build(&self, app: &mut App) {
        app.add_observer(on_mordekaiser_q);
        app.add_observer(on_mordekaiser_w);
        app.add_observer(on_mordekaiser_e);
        app.add_observer(on_mordekaiser_r);
        app.add_observer(on_mordekaiser_damage_hit);
        app.add_observer(on_mordekaiser_damage_taken);
        app.add_systems(FixedUpdate, update_mordekaiser_passive);
        app.add_systems(FixedUpdate, update_mordekaiser_w_shield);
        app.add_systems(FixedUpdate, update_mordekaiser_realm);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Mordekaiser"))]
#[reflect(Component)]
pub struct Mordekaiser;