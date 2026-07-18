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

use crate::aatrox::buffs::AatroxPassiveState;
use crate::aatrox::e::on_aatrox_e;
use crate::aatrox::passive::{on_aatrox_attack_end, update_aatrox_passive};
use crate::aatrox::q::on_aatrox_q;
use crate::aatrox::r::{on_aatrox_r, update_aatrox_r};
use crate::aatrox::w::{on_aatrox_w, update_aatrox_w_marks};

#[derive(Default)]
pub struct PluginAatrox;

impl Plugin for PluginAatrox {
    fn build(&self, app: &mut App) {
        app.add_observer(on_aatrox_q);
        app.add_observer(on_aatrox_w);
        app.add_observer(on_aatrox_e);
        app.add_observer(on_aatrox_r);
        app.add_observer(on_aatrox_attack_end);
        app.add_systems(FixedUpdate, update_aatrox_passive);
        app.add_systems(FixedUpdate, update_aatrox_w_marks);
        app.add_systems(FixedUpdate, update_aatrox_r);
    }
}

#[derive(Component, Reflect, Default)]
#[require(Champion, Name = Name::new("Aatrox"), AatroxPassiveState)]
#[reflect(Component)]
pub struct Aatrox;
