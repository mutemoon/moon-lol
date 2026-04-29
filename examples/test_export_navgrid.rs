use bevy::prelude::Vec2;
use lol_base::grid::{
    ConfigNavigationGrid, ConfigNavigationGridCell, GridFlagsJungleQuadrant, GridFlagsMainRegion,
    GridFlagsNearestLane, GridFlagsPOI, GridFlagsRing, GridFlagsRiverRegion, GridFlagsSRX,
    GridFlagsVisionPathing,
};
use lol_base::map::MapPaths;

fn main() {
    let map_paths = MapPaths::new("test");

    println!("导出路径: {}", map_paths.navgrid_bin_export());

    println!("正在创建测试导航网格...");
    let nav_grid = make_test_grid();
    println!(
        "导航网格: {}x{} 格子, cell_size={}",
        nav_grid.x_len, nav_grid.y_len, nav_grid.cell_size
    );

    println!("正在导出到 {}...", map_paths.navgrid_bin_export());
    write_nav_grid(&nav_grid, &map_paths.navgrid_bin_export());

    println!("导出完成!");
}

fn make_test_grid() -> ConfigNavigationGrid {
    let cell = ConfigNavigationGridCell {
        heuristic: 1.0,
        vision_pathing_flags: GridFlagsVisionPathing::Walkable,
        river_region_flags: GridFlagsRiverRegion::empty(),
        jungle_quadrant_flags: GridFlagsJungleQuadrant::empty(),
        main_region_flags: GridFlagsMainRegion::from(0),
        nearest_lane_flags: GridFlagsNearestLane::from(0),
        poi_flags: GridFlagsPOI::from(0),
        ring_flags: GridFlagsRing::from(0),
        srx_flags: GridFlagsSRX::from(0),
    };

    ConfigNavigationGrid {
        min_position: Vec2::new(-2000.0, -2000.0),
        cell_size: 50.0,
        x_len: 100,
        y_len: 100,
        cells: vec![vec![cell; 100]; 100],
        height_x_len: 2,
        height_y_len: 2,
        height_samples: vec![vec![0.0; 100]; 100],
        occupied_cells: Default::default(),
        exclude_cells: Default::default(),
    }
}

/// 将导航网格写入 bin 文件
fn write_nav_grid(grid: &ConfigNavigationGrid, path: &str) {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("无法创建目录");
    }
    let data = bincode::serialize(grid).expect("无法序列化为二进制");
    std::fs::write(path, data).expect("无法写入文件");
}
