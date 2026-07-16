pub mod buffs;
pub mod e;
pub mod q;
pub mod r;
pub mod w;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;

use crate::renekton::e::on_renekton_e;
use crate::renekton::q::on_renekton_q;
use crate::renekton::r::on_renekton_r;
use crate::renekton::w::on_renekton_w;

#[derive(Default)]
pub struct PluginRenekton;

impl Plugin for PluginRenekton {
    fn build(&self, app: &mut App) {
        app.add_observer(on_renekton_q);
        app.add_observer(on_renekton_w);
        app.add_observer(on_renekton_e);
        app.add_observer(on_renekton_r);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Renekton"))]
#[reflect(Component)]
pub struct Renekton;