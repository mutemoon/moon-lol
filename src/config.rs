use std::collections::HashMap;

use bevy::{ecs::resource::Resource, math::Vec2};
use serde::{Deserialize, Serialize};

use crate::combat::Lane;

#[derive(Resource, Serialize, Deserialize)]
pub struct GameConfig {
    pub environments: Vec<EnvironmentItem>,
    pub minion_paths: HashMap<Lane, Vec<Vec2>>,
}

#[derive(Serialize, Deserialize)]
pub struct EnvironmentItem {
    pub name: String,
    pub mesh_path: String,
    pub texture_path: String,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use bevy::scene::ron;

    use super::*;

    #[test]
    fn test_environment_item() {
        let environment_item = EnvironmentItem {
            name: "Test Environment".to_string(),
            mesh_path: "test_mesh".to_string(),
            texture_path: "test_texture".to_string(),
        };

        let writer = File::create("assets/test.ron").unwrap();

        let result = ron::ser::to_writer(writer, &environment_item);
        assert_eq!(result.is_ok(), true);

        let mut reader = BufReader::new(File::open("assets/test.ron").unwrap());
        let environment_item: EnvironmentItem = ron::de::from_reader(&mut reader).unwrap();

        assert_eq!(environment_item.name, "Test Environment");
        assert_eq!(environment_item.mesh_path, "test_mesh");
        assert_eq!(environment_item.texture_path, "test_texture");
    }
}
