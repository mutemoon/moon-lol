use bevy::prelude::*;
use lol_config::ConfigGame;

use crate::core::CommandSkinSpawn;

#[derive(Default)]
pub struct PluginGame;

impl Plugin for PluginGame {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedFrameCount>();

        app.add_systems(Startup, startup);
        app.add_systems(FixedLast, fixed_update_frame);
    }
}

#[derive(Resource, Default)]
pub struct FixedFrameCount(pub u32);

fn fixed_update_frame(mut frame: ResMut<FixedFrameCount>) {
    frame.0 += 1;
}

fn startup(mut commands: Commands, config_game: Res<ConfigGame>) {
    for (entity, skin) in config_game.legends.iter() {
        commands.trigger_targets(
            CommandSkinSpawn {
                skin_path: skin.clone(),
            },
            *entity,
        );
    }
}
