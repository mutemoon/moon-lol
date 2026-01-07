use bevy::math::Vec3;
use bitflags::bitflags;
use league_utils::BoundingBox;
use nom::bytes::complete::{tag, take};
use nom::multi::count;
use nom::number::complete::{le_f32, le_i32, le_u16, le_u32, le_u8};
use nom::{IResult, Parser};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct StaticMeshFlags: u32 {
        const HasVcp = 1;
        const HasLocalOriginLocatorAndPivot = 2;
    }
}

#[derive(Debug, Clone)]
pub struct StaticMeshFace {
    pub material: String,
    pub indices: [u16; 3],
    pub uvs: [[f32; 2]; 3],
    pub color0: [u8; 3],
    pub color1: [u8; 3],
    pub color2: [u8; 3],
}

#[derive(Debug)]
pub struct LeagueMeshStatic {
    pub major: u16,
    pub minor: u16,
    pub name: String,
    pub bounding_box: BoundingBox,
    pub has_vertex_colors: bool,
    pub vertices: Vec<[f32; 3]>,
    pub vertex_colors: Option<Vec<[u8; 4]>>,
    pub central_point: Vec3,
    pub faces: Vec<StaticMeshFace>,
}

impl LeagueMeshStatic {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, _) = tag(&b"r3d2Mesh"[..])(input)?;
        let (i, major) = le_u16(i)?;
        let (i, minor) = le_u16(i)?;

        if !((major == 2 || major == 3) || minor == 1) {
            panic!("Invalid file version: {}.{}", major, minor);
        }

        let (i, name) = parse_padded_string(i, 128)?;
        let (i, vertex_count) = le_i32(i)?;
        let (i, face_count) = le_i32(i)?;
        let (i, flags_raw) = le_u32(i)?;
        let flags = StaticMeshFlags::from_bits_truncate(flags_raw);

        let (i, bounding_box) = BoundingBox::parse(i)?;

        let mut has_vertex_colors = false;
        let mut current_i = i;
        if major >= 3 && minor >= 2 {
            let (i_next, has_vertex_colors_raw) = le_u32(current_i)?;
            has_vertex_colors = has_vertex_colors_raw == 1;
            current_i = i_next;
        }

        let (i, vertices_raw) = count(count(le_f32, 3), vertex_count as usize).parse(current_i)?;
        let vertices = vertices_raw
            .into_iter()
            .map(|v| [v[0], v[1], v[2]])
            .collect();

        let mut vertex_colors = None;
        let mut current_i = i;
        if has_vertex_colors {
            let (i_next, colors_raw) =
                count(count(le_u8, 4), vertex_count as usize).parse(current_i)?;
            vertex_colors = Some(
                colors_raw
                    .into_iter()
                    .map(|v| [v[0], v[1], v[2], v[3]])
                    .collect(),
            );
            current_i = i_next;
        }

        let (i, central_point_raw) = count(le_f32, 3).parse(current_i)?;
        let central_point = Vec3::from_slice(&central_point_raw);

        let (i, faces_on_disk) = count(StaticMeshFaceDisk::parse, face_count as usize).parse(i)?;

        let mut face_colors_data = None;
        let mut current_i = i;
        if flags.contains(StaticMeshFlags::HasVcp) {
            let (i_next, colors) =
                count(FaceColors::parse, face_count as usize).parse(current_i)?;
            face_colors_data = Some(colors);
            current_i = i_next;
        }

        let faces = Self::build_faces(faces_on_disk, face_colors_data);

        Ok((
            current_i,
            LeagueMeshStatic {
                major,
                minor,
                name,
                bounding_box,
                has_vertex_colors,
                vertices,
                vertex_colors,
                central_point,
                faces,
            },
        ))
    }

    fn build_faces(
        disk_data: Vec<StaticMeshFaceDisk>,
        colors_data: Option<Vec<FaceColors>>,
    ) -> Vec<StaticMeshFace> {
        let default_color = [255, 255, 255];
        let faces_iter = disk_data.into_iter().map(|disk_face| {
            let uvs = [
                [disk_face.uv_x[0], disk_face.uv_y[0]],
                [disk_face.uv_x[1], disk_face.uv_y[1]],
                [disk_face.uv_x[2], disk_face.uv_y[2]],
            ];
            let indices = [
                disk_face.indices[0] as u16,
                disk_face.indices[1] as u16,
                disk_face.indices[2] as u16,
            ];
            StaticMeshFace {
                material: disk_face.material,
                indices,
                uvs,
                color0: default_color,
                color1: default_color,
                color2: default_color,
            }
        });

        if let Some(colors) = colors_data {
            faces_iter
                .zip(colors.into_iter())
                .map(|(mut face, color_data)| {
                    face.color0 = color_data.color0;
                    face.color1 = color_data.color1;
                    face.color2 = color_data.color2;
                    face
                })
                .collect()
        } else {
            faces_iter.collect()
        }
    }
}

struct StaticMeshFaceDisk {
    pub indices: [u32; 3],
    pub material: String,
    pub uv_x: [f32; 3],
    pub uv_y: [f32; 3],
}

impl StaticMeshFaceDisk {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, indices_raw) = count(le_u32, 3).parse(input)?;
        let (i, material) = parse_padded_string(i, 64)?;
        let (i, uv_x_raw) = count(le_f32, 3).parse(i)?;
        let (i, uv_y_raw) = count(le_f32, 3).parse(i)?;

        Ok((
            i,
            StaticMeshFaceDisk {
                indices: indices_raw.try_into().unwrap(),
                material,
                uv_x: uv_x_raw.try_into().unwrap(),
                uv_y: uv_y_raw.try_into().unwrap(),
            },
        ))
    }
}

struct FaceColors {
    pub color0: [u8; 3],
    pub color1: [u8; 3],
    pub color2: [u8; 3],
}

impl FaceColors {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, c0) = count(le_u8, 3).parse(input)?;
        let (i, c1) = count(le_u8, 3).parse(i)?;
        let (i, c2) = count(le_u8, 3).parse(i)?;
        Ok((
            i,
            FaceColors {
                color0: c0.try_into().unwrap(),
                color1: c1.try_into().unwrap(),
                color2: c2.try_into().unwrap(),
            },
        ))
    }
}

fn parse_padded_string(input: &[u8], size: usize) -> IResult<&[u8], String> {
    let (i, bytes) = take(size)(input)?;
    let null_pos = bytes.iter().position(|&c| c == 0).unwrap_or(bytes.len());
    Ok((i, String::from_utf8_lossy(&bytes[..null_pos]).to_string()))
}
