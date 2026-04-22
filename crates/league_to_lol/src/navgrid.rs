use bevy::math::Vec3Swizzles;
use league_core::grid::{
    JungleQuadrantFlags, MainRegionFlags, NearestLaneFlags, POIFlags, RingFlags, RiverRegionFlags,
    UnknownSRXFlags, VisionPathingFlags,
};
use league_file::grid::AiMeshNGrid;
use lol_base::grid::{
    ConfigNavigationGrid, ConfigNavigationGridCell, GridFlagsJungleQuadrant, GridFlagsMainRegion,
    GridFlagsNearestLane, GridFlagsPOI, GridFlagsRing, GridFlagsRiverRegion, GridFlagsSRX,
    GridFlagsVisionPathing,
};

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
            vision_pathing_flags: GridFlagsVisionPathing::from_bits(
                nav_grid.vision_pathing_flags[i].bits(),
            )
            .unwrap_or(GridFlagsVisionPathing::Walkable),
            river_region_flags: GridFlagsRiverRegion::from_bits(
                nav_grid.other_flags[i].river_region_flags.bits(),
            )
            .unwrap_or(GridFlagsRiverRegion::empty()),
            jungle_quadrant_flags: GridFlagsJungleQuadrant::from_bits(
                nav_grid.other_flags[i].jungle_quadrant_flags.bits(),
            )
            .unwrap_or(GridFlagsJungleQuadrant::empty()),
            main_region_flags: GridFlagsMainRegion::from(
                nav_grid.other_flags[i].main_region_flags as u8,
            ),
            nearest_lane_flags: GridFlagsNearestLane::from(
                nav_grid.other_flags[i].nearest_lane_flags as u8,
            ),
            poi_flags: GridFlagsPOI::from(nav_grid.other_flags[i].poi_flags as u8),
            ring_flags: GridFlagsRing::from(nav_grid.other_flags[i].ring_flags as u8),
            srx_flags: GridFlagsSRX::from(nav_grid.other_flags[i].srx_flags as u8),
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
