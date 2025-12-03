use bevy::prelude::*;
use lol_config::ConfigGame;

use crate::CommandSpawnCharacter;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameStartupSystems {
    SpawnChampion,
}

#[derive(Default)]
pub struct PluginGame;

impl Plugin for PluginGame {
    fn build(&self, app: &mut App) {
        app.init_resource::<FixedFrameCount>();

        app.add_systems(Startup, startup.in_set(GameStartupSystems::SpawnChampion));
        app.add_systems(FixedLast, fixed_update_frame);
    }
}

#[derive(Resource, Default)]
pub struct FixedFrameCount(pub u32);

fn fixed_update_frame(mut frame: ResMut<FixedFrameCount>) {
    frame.0 += 1;
}

fn startup(mut commands: Commands, config_game: Res<ConfigGame>) {
    // 使用 ConfigGame 中保存的 character_record 和 skin_path
    for (entity, skin, character_record) in config_game.legends.iter() {
        commands.trigger(CommandSpawnCharacter {
            entity: *entity,
            character_record_key: character_record.clone(),
            skin_path: skin.clone(),
        });
    }
}
