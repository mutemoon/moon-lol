use crate::{combat::Lane, league::LeagueLoader};
use bevy::math::{Mat4, Vec2, Vec3};
use cdragon_prop::{BinList, BinMatrix, BinString, BinStruct, BinVec2, BinVec3};

#[derive(Debug)]
pub struct LeagueMinionPath {
    pub lane: Lane,
    pub path: Vec<Vec2>,
}

impl From<&BinStruct> for LeagueMinionPath {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .map(|v| Mat4::from_cols_array_2d(&v.0))
            .unwrap();

        let name = value
            .getv::<BinString>(LeagueLoader::hash_bin("name").into())
            .map(|v| v.0.as_str())
            .unwrap();

        let segments = value
            .getv::<BinList>(LeagueLoader::hash_bin("Segments").into())
            .iter()
            .filter_map(|v| v.downcast::<BinVec3>())
            .flat_map(|v| {
                v.iter()
                    .map(|v| Vec2::new(v.0 + transform.w_axis.x, -(v.2 + transform.w_axis.z)))
            })
            .collect();

        Self {
            lane: match name {
                "MinionPath_Top" => Lane::Top,
                "MinionPath_Mid" => Lane::Mid,
                "MinionPath_Bot" => Lane::Bot,
                _ => panic!("Unknown lane: {}", name),
            },
            path: segments,
        }
    }
}
