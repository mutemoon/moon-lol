//! examples/league_animation.rs
//! 读取 League 骨骼 + 动画，并用 Bevy 驱动

use bevy::prelude::*;
use moon_lol::league::LeagueLoader;

fn main() {
    let hash = LeagueLoader::hash_bin("MinionPath");

    println!("{:x}", hash);

    App::new().add_plugins(DefaultPlugins).run();
}
