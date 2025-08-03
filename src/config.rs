use bevy::ecs::resource::Resource;

use crate::render::LeagueMinionPath;

#[derive(Resource)]
pub struct GameConfig {
    pub minion_paths: Vec<LeagueMinionPath>,
}
