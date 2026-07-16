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
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<passive::FioraVitalLastDirection>();
        app.add_systems(
            FixedUpdate,
            (
                passive::attach_fiora_passive_ability,
                passive::update_add_vital,
                passive::update_remove_vital,
                r::fixed_update,
                r::update_fiora_r_heal,
                e::update_fiora_e_buff,
                passive::update_vital_visuals,
                w::update_fiora_w,
            ),
        );
        app.add_observer(q::on_fiora_q);
        app.add_observer(w::on_fiora_w);
        app.add_observer(e::on_fiora_e);
        app.add_observer(r::on_fiora_r);
        app.add_observer(q::on_fiora_q_dash_end);
        app.add_observer(passive::on_passive_damage_create);
        app.add_observer(e::on_event_attack_end);
        app.add_observer(r::on_r_damage_create);
        app.add_observer(w::on_fiora_w_parried_cc);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;