pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;

use crate::garen::e::on_garen_e;
use crate::garen::passive::on_garen_attack_end_silence;
use crate::garen::q::on_garen_q;
use crate::garen::r::on_garen_r;
use crate::garen::w::on_garen_w;

#[derive(Default)]
pub struct PluginGaren;

impl Plugin for PluginGaren {
    fn build(&self, app: &mut App) {
        app.add_observer(on_garen_q);
        app.add_observer(on_garen_w);
        app.add_observer(on_garen_e);
        app.add_observer(on_garen_r);
        app.add_observer(on_garen_attack_end_silence);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Garen"))]
#[reflect(Component)]
pub struct Garen;