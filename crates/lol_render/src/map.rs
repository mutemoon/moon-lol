use bevy::prelude::*;
use lol_core::action::{Action, CommandAction};

use crate::controller::Controller;

#[derive(Default)]
pub struct PluginRenderMap;

impl Plugin for PluginRenderMap {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
    }
}

#[derive(Component)]
pub struct Map;

pub fn on_click_map(
    click: On<Pointer<Press>>,
    mut commands: Commands,
    q_move: Query<Entity, With<Controller>>,
    // q_map_geo: Query<&MapGeometry>,
) {
    let Some(position) = click.hit.position else {
        return;
    };
    let targets = q_move.iter().collect::<Vec<Entity>>();

    // let map_geo_entity = click.entity;
    // if let Ok(map_geo) = q_map_geo.get(map_geo_entity) {
    //     println!("map_geo: {:?}", map_geo.config);
    // } else {
    //     println!("map_geo_entity: {:?}", map_geo_entity);
    // }

    for entity in targets {
        commands.trigger(CommandAction {
            entity,
            action: Action::Move(position.xz()),
        });
    }
}
