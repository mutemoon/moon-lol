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

use crate::sett::e::on_sett_e;
use crate::sett::passive::{
    on_sett_attack_end, on_sett_damage_hit, on_sett_damage_taken, update_sett_grit,
};
use crate::sett::q::on_sett_q;
use crate::sett::r::{on_sett_r, on_sett_r_dash_end};
use crate::sett::w::{on_sett_w, update_sett_w_shield};

#[derive(Default)]
pub struct PluginSett;

impl Plugin for PluginSett {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sett_q);
        app.add_observer(on_sett_w);
        app.add_observer(on_sett_e);
        app.add_observer(on_sett_r);
        app.add_observer(on_sett_r_dash_end);
        app.add_observer(on_sett_damage_hit);
        app.add_observer(on_sett_attack_end);
        app.add_observer(on_sett_damage_taken);
        app.add_systems(FixedUpdate, update_sett_grit);
        app.add_systems(FixedUpdate, update_sett_w_shield);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Sett"))]
#[reflect(Component)]
pub struct Sett;
