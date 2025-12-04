use bevy::prelude::*;
use binrw::BinRead;

use league_core::{MapContainer, MissileSpecificationBehaviors};
use league_file::AiMeshNGrid;
use league_loader::LeagueWadMapLoader;
use league_property::from_entry_unwrap;
use league_utils::hash_bin;
use lol_config::{ConfigNavigationGrid, ConfigNavigationGridCell};

use crate::Error;

pub async fn load_navigation_grid(
    loader: &LeagueWadMapLoader,
) -> Result<ConfigNavigationGrid, Error> {
    let entry = loader
        .materials_bin
        .iter_entry_by_class(hash_bin("MapContainer"))
        .next()
        .unwrap();

    let map_container = from_entry_unwrap::<MapContainer>(entry);

    let components = map_container.components;

    let map_nav_grid = components
        .iter()
        .filter_map(|v| match v {
            MissileSpecificationBehaviors::MapNavGrid(v) => Some(v),
            _ => None,
        })
        .next()
        .unwrap();

    let mut reader = loader
        .wad_loader
        .get_wad_entry_reader_by_path(&map_nav_grid.nav_grid_path)
        .unwrap();

    let nav_grid = AiMeshNGrid::read(&mut reader).unwrap();

    let min_bounds = nav_grid.header.min_bounds.xz();

    let min_position = vec2(min_bounds.x, min_bounds.y);

    let cell_size = nav_grid.header.cell_size;

    let x_len = nav_grid.header.x_cell_count as usize;
    let y_len = nav_grid.header.z_cell_count as usize;

    let mut cells: Vec<ConfigNavigationGridCell> = Vec::new();

    for (i, cell) in nav_grid.navigation_grid.iter().enumerate() {
        let cell = ConfigNavigationGridCell {
            heuristic: cell.heuristic,
            vision_pathing_flags: nav_grid.vision_pathing_flags[i],
            river_region_flags: nav_grid.other_flags[i].river_region_flags,
            jungle_quadrant_flags: nav_grid.other_flags[i].jungle_quadrant_flags,
            main_region_flags: nav_grid.other_flags[i].main_region_flags,
            nearest_lane_flags: nav_grid.other_flags[i].nearest_lane_flags,
            poi_flags: nav_grid.other_flags[i].poi_flags,
            ring_flags: nav_grid.other_flags[i].ring_flags,
            srx_flags: nav_grid.other_flags[i].srx_flags,
        };

        cells.push(cell);
    }

    Ok(ConfigNavigationGrid {
        min_position,
        cell_size,
        x_len,
        y_len,
        cells: cells.chunks(x_len).map(|v| v.to_vec()).collect(),
        height_x_len: nav_grid.height_samples.x_count as usize,
        height_y_len: nav_grid.height_samples.z_count as usize,
        height_samples: nav_grid
            .height_samples
            .samples
            .chunks(nav_grid.height_samples.x_count as usize)
            .map(|v| v.to_vec())
            .collect(),
        ..default()
    })
}
