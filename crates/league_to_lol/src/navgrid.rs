use bevy::math::Vec3Swizzles;
use league_file::grid::AiMeshNGrid;
use lol_base::grid::{ConfigNavigationGrid, ConfigNavigationGridCell};

pub fn load_league_nav_grid(nav_grid: AiMeshNGrid) -> ConfigNavigationGrid {
    let min_bounds = nav_grid.header.min_bounds.xz();

    let min_position = bevy::prelude::vec2(min_bounds.x, min_bounds.y);

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

    ConfigNavigationGrid {
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
        ..Default::default()
    }
}
