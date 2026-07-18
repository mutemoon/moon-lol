pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

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
use lol_core::entities::champion::Champion;

use crate::volibear::buffs::VolibearPStacks;
use crate::volibear::e::on_volibear_e;
use crate::volibear::passive::{on_volibear_attack_end, on_volibear_damage_hit, update_volibear_p};
use crate::volibear::q::on_volibear_q;
use crate::volibear::r::{on_volibear_r, on_volibear_r_dash_end};
use crate::volibear::w::on_volibear_w;

#[derive(Default)]
pub struct PluginVolibear;

impl Plugin for PluginVolibear {
    fn build(&self, app: &mut App) {
        app.add_observer(on_volibear_q);
        app.add_observer(on_volibear_w);
        app.add_observer(on_volibear_e);
        app.add_observer(on_volibear_r);
        app.add_observer(on_volibear_r_dash_end);
        app.add_observer(on_volibear_damage_hit);
        app.add_observer(on_volibear_attack_end);
        app.add_systems(FixedUpdate, update_volibear_p);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Volibear"), VolibearPStacks)]
#[reflect(Component)]
pub struct Volibear;
