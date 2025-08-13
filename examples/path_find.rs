use std::fs::File;

use bevy::{math::vec3, scene::ron::de::from_reader};
use moon_lol::{
    core::{find_path, Configs},
    logging::setup_file_logging,
};

fn main() {
    let log_path = "moon_lol.log".to_string().into();

    // Set up file logging
    setup_file_logging(&log_path);

    let configs: Configs = from_reader(File::open("assets/configs.ron").unwrap()).unwrap();
    let path = find_path(
        &configs,
        configs.navigation_grid.get_center_pos(),
        configs.navigation_grid.get_center_pos() + vec3(500.0, 0.0, -500.0),
    );

    println!("{:?}", path);
}
