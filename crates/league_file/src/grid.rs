use bevy::math::Vec3;
use bevy::prelude::Resource;
use league_core::{
    JungleQuadrantFlags, MainRegionFlags, NearestLaneFlags, POIFlags, RingFlags, RiverRegionFlags,
    UnknownSRXFlags, VisionPathingFlags,
};
use nom::bytes::complete::take;
use nom::multi::count;
use nom::number::complete::{le_f32, le_i16, le_i32, le_u16, le_u32, le_u8};
use nom::{IResult, Parser};

#[derive(Debug, Resource, Clone)]
pub struct AiMeshNGrid {
    pub header: Header,
    pub navigation_grid: Vec<NavigationGridCell>,
    pub vision_pathing_flags: Vec<VisionPathingFlags>,
    pub other_flags: Vec<OtherFlags>,
    pub unknown_block: Vec<u8>,
    pub height_samples: HeightSamples,
    pub hint_nodes: HintNodes,
}

impl AiMeshNGrid {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, header) = Header::parse(input)?;
        let cell_count = (header.x_cell_count * header.z_cell_count) as usize;

        let (i, navigation_grid) = count(NavigationGridCell::parse, cell_count).parse(i)?;
        let (i, vision_pathing_flags_raw) = count(le_u16, cell_count).parse(i)?;
        let vision_pathing_flags = vision_pathing_flags_raw
            .into_iter()
            .map(|v| VisionPathingFlags::from_bits(v).unwrap())
            .collect();

        let (i, other_flags) = count(OtherFlags::parse, cell_count).parse(i)?;
        let (i, unknown_block_raw) = take(8usize * 132usize)(i)?;
        let unknown_block = unknown_block_raw.to_vec();

        let (i, height_samples) = HeightSamples::parse(i)?;
        let (i, hint_nodes) = HintNodes::parse(i)?;

        Ok((
            i,
            AiMeshNGrid {
                header,
                navigation_grid,
                vision_pathing_flags,
                other_flags,
                unknown_block,
                height_samples,
                hint_nodes,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct OtherFlags {
    pub river_region_flags: RiverRegionFlags,
    pub jungle_quadrant_flags: JungleQuadrantFlags,
    pub main_region_flags: MainRegionFlags,
    pub nearest_lane_flags: NearestLaneFlags,
    pub poi_flags: POIFlags,
    pub ring_flags: RingFlags,
    pub srx_flags: UnknownSRXFlags,
}

impl OtherFlags {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, river_region_raw) = le_u8(input)?;
        let (i, jungle_and_main) = le_u8(i)?;
        let (i, lane_and_poi) = le_u8(i)?;
        let (i, ring_and_srx) = le_u8(i)?;

        Ok((
            i,
            OtherFlags {
                river_region_flags: RiverRegionFlags::from_bits(river_region_raw).unwrap(),
                jungle_quadrant_flags: JungleQuadrantFlags::from_bits(jungle_and_main & 0x0f)
                    .unwrap(),
                main_region_flags: MainRegionFlags::from((jungle_and_main & 0xf0) >> 4),
                nearest_lane_flags: NearestLaneFlags::from(lane_and_poi & 0x0f),
                poi_flags: POIFlags::from((lane_and_poi & 0xf0) >> 4),
                ring_flags: RingFlags::from(ring_and_srx & 0x0f),
                srx_flags: UnknownSRXFlags::from((ring_and_srx & 0xf0) >> 4),
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub major_version: u8,
    pub minor_version: i16,
    pub min_bounds: Vec3,
    pub max_bounds: Vec3,
    pub cell_size: f32,
    pub x_cell_count: u32,
    pub z_cell_count: u32,
}

impl Header {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, major_version) = le_u8(input)?;
        let (i, minor_version) = le_i16(i)?;
        let (i, min_bounds) = parse_vec3(i)?;
        let (i, max_bounds) = parse_vec3(i)?;
        let (i, cell_size) = le_f32(i)?;
        let (i, x_cell_count) = le_u32(i)?;
        let (i, z_cell_count) = le_u32(i)?;

        Ok((
            i,
            Header {
                major_version,
                minor_version,
                min_bounds,
                max_bounds,
                cell_size,
                x_cell_count,
                z_cell_count,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct NavigationGridCell {
    pub center_height: f32,
    pub session_id: i32,
    pub arrival_cost: f32,
    pub is_open: i32,
    pub heuristic: f32,
    pub x: usize,
    pub z: usize,
    pub actor_list: i32,
    pub unknown1: i32,
    pub good_cell_session_id: i32,
    pub hint_weight: f32,
    pub unknown2: i16,
    pub arrival_direction: i16,
    pub hint_node1: i16,
    pub hint_node2: i16,
}

impl NavigationGridCell {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, center_height) = le_f32(input)?;
        let (i, session_id) = le_i32(i)?;
        let (i, arrival_cost) = le_f32(i)?;
        let (i, is_open) = le_i32(i)?;
        let (i, heuristic) = le_f32(i)?;
        let (i, x_raw) = le_i16(i)?;
        let (i, z_raw) = le_i16(i)?;
        let (i, actor_list) = le_i32(i)?;
        let (i, unknown1) = le_i32(i)?;
        let (i, good_cell_session_id) = le_i32(i)?;
        let (i, hint_weight) = le_f32(i)?;
        let (i, unknown2) = le_i16(i)?;
        let (i, arrival_direction) = le_i16(i)?;
        let (i, hint_node1) = le_i16(i)?;
        let (i, hint_node2) = le_i16(i)?;

        Ok((
            i,
            NavigationGridCell {
                center_height,
                session_id,
                arrival_cost,
                is_open,
                heuristic,
                x: x_raw as usize,
                z: z_raw as usize,
                actor_list,
                unknown1,
                good_cell_session_id,
                hint_weight,
                unknown2,
                arrival_direction,
                hint_node1,
                hint_node2,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HeightSamples {
    pub x_count: u32,
    pub z_count: u32,
    pub offset_x: f32,
    pub offset_z: f32,
    pub samples: Vec<f32>,
}

impl HeightSamples {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, x_count) = le_u32(input)?;
        let (i, z_count) = le_u32(i)?;
        let (i, offset_x) = le_f32(i)?;
        let (i, offset_z) = le_f32(i)?;
        let (i, samples) = count(le_f32, (x_count * z_count) as usize).parse(i)?;

        Ok((
            i,
            HeightSamples {
                x_count,
                z_count,
                offset_x,
                offset_z,
                samples,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HintNodes {
    pub distances: Vec<f32>,
    pub hint_coordinates: Vec<HintCoordinate>,
}

impl HintNodes {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, distances) = count(le_f32, 900usize * 900usize).parse(input)?;
        let (i, hint_coordinates) = count(HintCoordinate::parse, 900usize).parse(i)?;

        Ok((
            i,
            HintNodes {
                distances,
                hint_coordinates,
            },
        ))
    }
}

#[derive(Debug, Clone)]
pub struct HintCoordinate {
    pub x: i16,
    pub y: i16,
}

impl HintCoordinate {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, x) = le_i16(input)?;
        let (i, y) = le_i16(i)?;
        Ok((i, HintCoordinate { x, y }))
    }
}

fn parse_vec3(input: &[u8]) -> IResult<&[u8], Vec3> {
    let (i, x) = le_f32(input)?;
    let (i, y) = le_f32(i)?;
    let (i, z) = le_f32(i)?;
    Ok((i, Vec3::new(x, y, z)))
}
