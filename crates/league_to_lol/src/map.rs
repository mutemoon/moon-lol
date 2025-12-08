use bevy::prelude::*;

use league_loader::LeagueWadMapLoader;
use lol_config::ConfigNavigationGrid;

use crate::{
    get_bin_path, load_navigation_grid, save_struct_to_file, Error, CONFIG_PATH_MAP_NAV_GRID,
};

pub async fn save_navigation_grid(
    loader: &LeagueWadMapLoader,
) -> Result<ConfigNavigationGrid, Error> {
    let nav_grid = load_navigation_grid(loader).await?;
    let path = get_bin_path(CONFIG_PATH_MAP_NAV_GRID);
    save_struct_to_file(&path, &nav_grid).await?;
    Ok(nav_grid)
}
