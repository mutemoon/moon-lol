use bevy::{
    ecs::resource::Resource,
    math::{vec2, vec3, Vec2, Vec3},
};
use league_core::{
    JungleQuadrantFlags, MainRegionFlags, NearestLaneFlags, POIFlags, RingFlags, RiverRegionFlags,
    UnknownSRXFlags, VisionPathingFlags,
};
use serde::{Deserialize, Serialize};

#[derive(Resource, Clone, Default, Serialize, Deserialize)]
pub struct ConfigNavigationGrid {
    pub min_position: Vec2,
    pub cell_size: f32,
    pub x_len: usize,
    pub y_len: usize,
    pub cells: Vec<Vec<ConfigNavigationGridCell>>,
    pub height_x_len: usize,
    pub height_y_len: usize,
    pub height_samples: Vec<Vec<f32>>,
}

impl ConfigNavigationGrid {
    pub fn get_width(&self) -> f32 {
        self.x_len as f32 * self.cell_size
    }

    pub fn get_height(&self) -> f32 {
        self.y_len as f32 * self.cell_size
    }

    pub fn get_height_by_position(&self, position: &Vec2) -> f32 {
        let x = (((position.x - self.min_position.x) / self.get_width())
            * (self.height_x_len - 1) as f32)
            .round() as usize;

        let y = (((position.y - self.min_position.y) / self.get_height())
            * (self.height_y_len - 1) as f32)
            .round() as usize;

        self.height_samples[y][x]
    }

    pub fn get_first_cell_center_position(&self) -> Vec2 {
        Vec2::new(
            self.min_position.x + self.cell_size / 2.0,
            self.min_position.y + self.cell_size / 2.0,
        )
    }

    pub fn get_cell_center_position_by_xy(&self, (x, y): (usize, usize)) -> Vec3 {
        let first_cell_center_position = self.get_first_cell_center_position();
        let cell_center_position = vec2(
            first_cell_center_position.x + x as f32 * self.cell_size,
            first_cell_center_position.y + y as f32 * self.cell_size,
        );
        vec3(
            cell_center_position.x,
            self.get_height_by_position(&cell_center_position),
            cell_center_position.y,
        )
    }

    pub fn get_cell_xy_by_position(&self, position: &Vec2) -> (usize, usize) {
        let x = ((position.x - self.min_position.x) / self.cell_size).floor() as usize;
        let y = ((position.y - self.min_position.y) / self.cell_size).floor() as usize;
        (x, y)
    }

    pub fn get_cell_by_xy(&self, (x, y): (usize, usize)) -> &ConfigNavigationGridCell {
        &self.cells[y.clamp(0, self.y_len - 1)][x.clamp(0, self.x_len - 1)]
    }

    pub fn get_cell_by_position(&self, pos: &Vec2) -> &ConfigNavigationGridCell {
        self.get_cell_by_xy(self.get_cell_xy_by_position(pos))
    }

    pub fn get_world_position_by_position(&self, position: &Vec2) -> Vec3 {
        vec3(
            position.x,
            self.get_height_by_position(position),
            position.y,
        )
    }

    pub fn get_position_by_float_xy(&self, pos: &Vec2) -> Vec2 {
        vec2(
            self.min_position.x + pos.x * self.cell_size,
            self.min_position.y + pos.y * self.cell_size,
        )
    }

    pub fn get_map_center_position(&self) -> Vec3 {
        self.get_world_position_by_position(&vec2(self.get_width() / 2.0, self.get_height() / 2.0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigNavigationGridCell {
    pub heuristic: f32,
    pub vision_pathing_flags: VisionPathingFlags,
    pub river_region_flags: RiverRegionFlags,
    pub jungle_quadrant_flags: JungleQuadrantFlags,
    pub main_region_flags: MainRegionFlags,
    pub nearest_lane_flags: NearestLaneFlags,
    pub poi_flags: POIFlags,
    pub ring_flags: RingFlags,
    pub srx_flags: UnknownSRXFlags,
}

impl ConfigNavigationGridCell {
    pub fn is_wall(&self) -> bool {
        self.vision_pathing_flags.contains(VisionPathingFlags::Wall)
    }

    pub fn is_walkable(&self) -> bool {
        !self.is_wall()
    }
}
