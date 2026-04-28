use std::collections::{HashMap, HashSet};

use bevy::asset::Asset;
use bevy::math::{Vec2, Vec3, vec2, vec3};
use bevy::reflect::TypePath;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// 表示格子不可通行的成本值
pub const CELL_COST_IMPASSABLE: f32 = f32::MAX;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GridFlagsVisionPathing: u16 {
        const Walkable = 0;

        const Brush = 1 << 0;
        const Wall = 1 << 1;
        const StructureWall = 1 << 2;
        const Unobserved8 = 1 << 3;

        const Unobserved16 = 1 << 4;
        const Unobserved32 = 1 << 5;
        const TransparentWall = 1 << 6;
        // marks the difference between two otherwise-equivalent cells, spread sporadically throughout the map, ignored for a cleaner image since it doesn't seem useful at all
        const Unknown128 = 1 << 7;

        const AlwaysVisible = 1 << 8;
        // only ever found on the original Nexus Blitz map, and it was only present in two sections of what would otherwise be normal wall
        const Unknown512 = 1 << 9;
        const BlueTeamOnly = 1 << 10;
        const RedTeamOnly = 1 << 11;

        // no bits observed past this point
        const NeutralZoneVisiblity = 1 << 12;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GridFlagsRiverRegion: u8 {
        const NonJungle = 0;

        const JungleQuadrant = 1 << 0;
        const BaronPit = 1 << 1;
        const Unobserved4 = 1 << 2;
        const Unobserved8 = 1 << 3;

        const River = 1 << 4;
        // only ever found on the original Nexus Blitz map, where it was instead used to represent the river (other flags were shuffled too)
        const Unknown32 = 1 << 5;
        // no bits observed past this point
        const RiverEntrance = 1 << 6;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct GridFlagsJungleQuadrant: u8 {
        const None = 0;

        const NorthJungleQuadrant = 1 << 0;
        const EastJungleQuadrant = 1 << 1;
        const WestJungleQuadrant = 1 << 2;
        const SouthJungleQuadrant = 1 << 3;

        const Unobserved8 = 1 << 4;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridFlagsMainRegion {
    Spawn = 0,
    Base = 1,

    TopLane = 2,
    MidLane = 3,
    BotLane = 4,

    TopSideJungle = 5,
    BotSideJungle = 6,

    TopSideRiver = 7,
    BotSideRiver = 8,

    TopSideBasePerimeter = 9,
    BotSideBasePerimeter = 10,

    TopSideLaneAlcove = 11,
    BotSideLaneAlcove = 12,
}

impl From<u8> for GridFlagsMainRegion {
    fn from(value: u8) -> Self {
        match value {
            0 => GridFlagsMainRegion::Spawn,
            1 => GridFlagsMainRegion::Base,
            2 => GridFlagsMainRegion::TopLane,
            3 => GridFlagsMainRegion::MidLane,
            4 => GridFlagsMainRegion::BotLane,
            5 => GridFlagsMainRegion::TopSideJungle,
            6 => GridFlagsMainRegion::BotSideJungle,
            7 => GridFlagsMainRegion::TopSideRiver,
            8 => GridFlagsMainRegion::BotSideRiver,
            9 => GridFlagsMainRegion::TopSideBasePerimeter,
            10 => GridFlagsMainRegion::BotSideBasePerimeter,
            11 => GridFlagsMainRegion::TopSideLaneAlcove,
            12 => GridFlagsMainRegion::BotSideLaneAlcove,
            _ => GridFlagsMainRegion::Spawn, // 默认值
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridFlagsNearestLane {
    BlueSideTopLane = 0,
    BlueSideMidLane = 1,
    BlueSideBotLane = 2,

    RedSideTopLane = 3,
    RedSideMidLane = 4,
    RedSideBotLane = 5,

    BlueSideTopNeutralZone = 6,
    BlueSideMidNeutralZone = 7,
    BlueSideBotNeutralZone = 8,

    RedSideTopNeutralZone = 9,
    RedSideMidNeutralZone = 10,
    RedSideBotNeutralZone = 11,
}

impl From<u8> for GridFlagsNearestLane {
    fn from(value: u8) -> Self {
        match value {
            0 => GridFlagsNearestLane::BlueSideTopLane,
            1 => GridFlagsNearestLane::BlueSideMidLane,
            2 => GridFlagsNearestLane::BlueSideBotLane,
            3 => GridFlagsNearestLane::RedSideTopLane,
            4 => GridFlagsNearestLane::RedSideMidLane,
            5 => GridFlagsNearestLane::RedSideBotLane,
            6 => GridFlagsNearestLane::BlueSideTopNeutralZone,
            7 => GridFlagsNearestLane::BlueSideMidNeutralZone,
            8 => GridFlagsNearestLane::BlueSideBotNeutralZone,
            9 => GridFlagsNearestLane::RedSideTopNeutralZone,
            10 => GridFlagsNearestLane::RedSideMidNeutralZone,
            11 => GridFlagsNearestLane::RedSideBotNeutralZone,
            _ => GridFlagsNearestLane::BlueSideTopLane, // 默认值
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridFlagsPOI {
    None = 0,

    NearTurret = 1,
    // note:  as of preseason 10, this flag now corresponds to cloud drake wind tunnels, and all following flags are removed
    CloudDrakeWindTunnelOrBaseGates = 2,

    BaronPit = 3,
    DragonPit = 4,

    CampRedBuff = 5,
    CampBlueBuff = 6,
    CampGromp = 7,
    CampKrugs = 8,
    CampRaptors = 9,
    CampMurkWolves = 10,
}

impl From<u8> for GridFlagsPOI {
    fn from(value: u8) -> Self {
        match value {
            0 => GridFlagsPOI::None,
            1 => GridFlagsPOI::NearTurret,
            2 => GridFlagsPOI::CloudDrakeWindTunnelOrBaseGates,
            3 => GridFlagsPOI::BaronPit,
            4 => GridFlagsPOI::DragonPit,
            5 => GridFlagsPOI::CampRedBuff,
            6 => GridFlagsPOI::CampBlueBuff,
            7 => GridFlagsPOI::CampGromp,
            8 => GridFlagsPOI::CampKrugs,
            9 => GridFlagsPOI::CampRaptors,
            10 => GridFlagsPOI::CampMurkWolves,
            _ => GridFlagsPOI::None, // 默认值
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridFlagsRing {
    BlueSpawnToNexus = 0,
    BlueNexusToInhib = 1,
    BlueInhibToInner = 2,
    BlueInnerToOuter = 3,
    BlueOuterToNeutral = 4,

    RedSpawnToNexus = 5,
    RedNexusToInhib = 6,
    RedInhibToInner = 7,
    RedInnerToOuter = 8,
    RedOuterToNeutral = 9,
}

impl From<u8> for GridFlagsRing {
    fn from(value: u8) -> Self {
        match value {
            0 => GridFlagsRing::BlueSpawnToNexus,
            1 => GridFlagsRing::BlueNexusToInhib,
            2 => GridFlagsRing::BlueInhibToInner,
            3 => GridFlagsRing::BlueInnerToOuter,
            4 => GridFlagsRing::BlueOuterToNeutral,
            5 => GridFlagsRing::RedSpawnToNexus,
            6 => GridFlagsRing::RedNexusToInhib,
            7 => GridFlagsRing::RedInhibToInner,
            8 => GridFlagsRing::RedInnerToOuter,
            9 => GridFlagsRing::RedOuterToNeutral,
            _ => GridFlagsRing::BlueSpawnToNexus, // 默认值
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridFlagsSRX {
    Walkable = 0,

    Wall = 1,
    TransparentWall = 2,
    Brush = 3,
    Unobserved4 = 4,

    TopSideOceanDrakePuddle = 5,
    BotSideOceanDrakePuddle = 6,
    BlueTeamOnly = 7,
    RedTeamOnly = 8,

    Unobserved9 = 9,
    Unobserved10 = 10,
    BlueTeamOnlyNeutralZoneVisibility = 11,
    RedTeamOnlyNeutralZoneVisibility = 12,

    BrushWall = 13,
}

impl From<u8> for GridFlagsSRX {
    fn from(value: u8) -> Self {
        match value {
            0 => GridFlagsSRX::Walkable,
            1 => GridFlagsSRX::Wall,
            2 => GridFlagsSRX::TransparentWall,
            3 => GridFlagsSRX::Brush,
            4 => GridFlagsSRX::Unobserved4,
            5 => GridFlagsSRX::TopSideOceanDrakePuddle,
            6 => GridFlagsSRX::BotSideOceanDrakePuddle,
            7 => GridFlagsSRX::BlueTeamOnly,
            8 => GridFlagsSRX::RedTeamOnly,
            9 => GridFlagsSRX::Unobserved9,
            10 => GridFlagsSRX::Unobserved10,
            11 => GridFlagsSRX::BlueTeamOnlyNeutralZoneVisibility,
            12 => GridFlagsSRX::RedTeamOnlyNeutralZoneVisibility,
            13 => GridFlagsSRX::BrushWall,
            _ => GridFlagsSRX::Walkable, // 默认值
        }
    }
}

#[derive(Asset, TypePath, Clone, Default, Serialize, Deserialize, Debug)]
pub struct ConfigNavigationGrid {
    pub min_position: Vec2,
    pub cell_size: f32,
    pub x_len: usize,
    pub y_len: usize,
    pub cells: Vec<Vec<ConfigNavigationGridCell>>,
    pub height_x_len: usize,
    pub height_y_len: usize,
    pub height_samples: Vec<Vec<f32>>,
    /// 动态障碍物的通行成本，值越大表示通行代价越高，CELL_COST_IMPASSABLE 表示不可通行
    #[serde(skip)]
    pub occupied_cells: HashMap<(usize, usize), f32>,
    #[serde(skip)]
    pub exclude_cells: HashSet<(usize, usize)>,
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

    /// 获取格子的动态障碍物通行成本，0.0 表示无额外成本
    pub fn get_cell_cost(&self, pos: (usize, usize)) -> f32 {
        if self.exclude_cells.contains(&pos) {
            return 0.0;
        }

        self.occupied_cells.get(&pos).copied().unwrap_or(0.0)
    }

    /// 判断格子是否可通行（静态墙体 + 动态障碍物成本检查）
    pub fn is_walkable_by_xy(&self, (x, y): (usize, usize)) -> bool {
        if x >= self.x_len || y >= self.y_len {
            return false;
        }
        if !self.get_cell_by_xy((x, y)).is_walkable() {
            return false;
        }
        if self.exclude_cells.contains(&(x, y)) {
            return true;
        }
        let cost = self.get_cell_cost((x, y));
        cost < CELL_COST_IMPASSABLE
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigNavigationGridCell {
    pub heuristic: f32,
    pub vision_pathing_flags: GridFlagsVisionPathing,
    pub river_region_flags: GridFlagsRiverRegion,
    pub jungle_quadrant_flags: GridFlagsJungleQuadrant,
    pub main_region_flags: GridFlagsMainRegion,
    pub nearest_lane_flags: GridFlagsNearestLane,
    pub poi_flags: GridFlagsPOI,
    pub ring_flags: GridFlagsRing,
    pub srx_flags: GridFlagsSRX,
}

impl ConfigNavigationGridCell {
    pub fn is_wall(&self) -> bool {
        self.vision_pathing_flags
            .contains(GridFlagsVisionPathing::Wall)
    }

    pub fn is_walkable(&self) -> bool {
        !self.is_wall()
    }
}
