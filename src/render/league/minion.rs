use crate::render::LeagueLoader;
use bevy::math::{Mat4, Vec3};
use cdragon_prop::{BinList, BinMatrix, BinString, BinStruct, BinVec3};

#[derive(Debug)]
pub struct LeagueMinionPath {
    pub transform: Mat4,
    pub name: String,
    pub segments: Vec<Vec3>,
}

impl From<&BinStruct> for LeagueMinionPath {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .map(|v| Mat4::from_cols_array_2d(&v.0))
            .unwrap();

        let name = value
            .getv::<BinString>(LeagueLoader::hash_bin("name").into())
            .map(|v| v.0.clone())
            .unwrap();

        let segments = value
            .getv::<BinList>(LeagueLoader::hash_bin("Segments").into())
            .iter()
            .filter_map(|v| v.downcast::<BinVec3>())
            .flat_map(|v| v.iter().map(|v| Vec3::new(v.0, v.1, v.2)))
            .collect();

        Self {
            transform,
            name,
            segments,
        }
    }
}
