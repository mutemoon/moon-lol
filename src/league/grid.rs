use crate::league::BinVec3;
use bevy::prelude::*;
use binrw::binread;

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct Header {
    pub major_version: u8,
    pub minor_version: i16,

    #[br(if(major_version > 0))]
    pub min_grid_pos: Option<BinVec3>,

    #[br(if(major_version > 0))]
    pub max_grid_pos: Option<BinVec3>,

    #[br(if(major_version > 0))]
    pub cell_size: Option<f32>,

    #[br(if(major_version > 0))]
    pub x_cell_count: Option<u32>,

    #[br(if(major_version > 0))]
    pub y_cell_count: Option<u32>,
}

impl Header {
    fn get_x_cell_count(&self) -> u32 {
        self.x_cell_count.unwrap_or(294)
    }

    fn get_y_cell_count(&self) -> u32 {
        self.y_cell_count.unwrap_or(295)
    }
}

// 注意：为了在 Bevy 中使用，所有 Cell 和 Enum 都需要 Clone
#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct Cell3 {
    pub height: f32,
    pub unk1: u32,
    pub arrival: f32,
    pub open: u8,
    pub heuristic: f32,
    pub actors: u32,
    pub x: i16,
    pub z: i16,
    pub unk2: f32,
    pub unk3: f32,
    pub unk4: u32,
    pub session_id_related: u32,
    pub ref_value: f32,
    pub arrival_direction: i16,
    pub flags: i16,
    pub ref_nodes: [i16; 2],
    pub unk5: [u8; 3],
}

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct Cell5 {
    pub height: f32,
    pub unk1: u32,
    pub arrival: f32,
    pub open: u8,
    pub heuristic: f32,
    pub actors: u32,
    pub x: i16,
    pub z: i16,
    pub unk2: f32,
    pub unk3: f32,
    pub unk4: u32,
    pub session_id_related: u32,
    pub ref_value: f32,
    pub arrival_direction: i16,
    pub flags: i16,
    pub ref_nodes: [i16; 2],
    pub unk5: [u8; 3],
}

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
#[br(import(version: u8))]
pub enum NavigationGridCell {
    #[br(pre_assert(version == 3))]
    Version3(Cell3),
    #[br(pre_assert(version == 5))]
    Version5(Cell5),
    #[br(pre_assert(version == 7))]
    Version7(Cell7),
}

impl NavigationGridCell {
    pub fn get_height(&self) -> f32 {
        match self {
            NavigationGridCell::Version3(cell) => cell.height,
            NavigationGridCell::Version5(cell) => cell.height,
            NavigationGridCell::Version7(cell) => cell.height,
        }
    }

    pub fn get_x(&self) -> usize {
        match self {
            NavigationGridCell::Version3(cell) => cell.x as usize,
            NavigationGridCell::Version5(cell) => cell.x as usize,
            NavigationGridCell::Version7(cell) => cell.x as usize,
        }
    }

    pub fn get_z(&self) -> usize {
        match self {
            NavigationGridCell::Version3(cell) => cell.z as usize,
            NavigationGridCell::Version5(cell) => cell.z as usize,
            NavigationGridCell::Version7(cell) => cell.z as usize,
        }
    }

    pub fn get_heuristic(&self) -> f32 {
        match self {
            NavigationGridCell::Version3(cell) => cell.heuristic,
            NavigationGridCell::Version5(cell) => cell.heuristic,
            NavigationGridCell::Version7(cell) => cell.heuristic,
        }
    }
}

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct FloatMap {
    pub x_count: u32,
    pub y_count: u32,
    pub unk1: f32,
    pub unk2: f32,
    #[br(count = x_count * y_count)]
    pub unk3: Vec<f32>,
    #[br(count = 810899)]
    pub unk4: Vec<f32>,
    pub unk5: i16,
    pub unk6: i16,
}

#[binread]
#[br(little)]
#[derive(Debug, Resource, Clone)]
pub struct AiMeshNGrid {
    pub header: Header,
    #[br(count = header.get_x_cell_count() * header.get_y_cell_count())]
    #[br(args { inner: (header.major_version,) })]
    pub navigation_grid: Vec<NavigationGridCell>,
    #[br(if(header.major_version == 5))]
    #[br(count = header.get_x_cell_count() * header.get_y_cell_count())]
    pub version5_unk1: Option<Vec<u16>>,
    #[br(if(header.major_version == 5))]
    #[br(count = 528)]
    pub version5_unk2: Option<Vec<u8>>,
    #[br(if(header.major_version == 7))]
    #[br(count = header.get_x_cell_count() * header.get_y_cell_count())]
    pub version7_unk1: Option<Vec<u16>>,
    #[br(if(header.major_version == 7))]
    #[br(count = header.get_x_cell_count() * header.get_y_cell_count())]
    pub version7_unk2: Option<Vec<u16>>,
    #[br(if(header.major_version == 7))]
    #[br(count = header.get_x_cell_count() * header.get_y_cell_count())]
    pub version7_unk3: Option<Vec<u16>>,
    #[br(if(header.major_version == 7))]
    #[br(count = 1056)]
    pub version7_unk4: Option<Vec<u8>>,
    pub float_map: FloatMap,
}

#[binread]
#[br(little)]
#[derive(Debug, Clone)]
pub struct Cell7 {
    pub height: f32,
    pub unk1: u16,
    pub min_height: f32,
    pub unk2: u16,
    pub heuristic: f32,
    pub actors: u32,
    pub x: i16,
    pub z: i16,
    pub unk3: f32,
    pub unk4: f32,
    pub session_id_related: u32,
    pub ref_value: f32,
    pub arrival_direction: i16,
    pub flags: i16,
    pub ref_nodes: [i16; 2],
}
