use crate::render::LeagueLoader;
use bevy::math::Mat4;
use cdragon_prop::{BinMatrix, BinStruct};

#[derive(Debug)]
pub struct LeagueBarrack {
    pub transform: Mat4,
}

impl From<&BinStruct> for LeagueBarrack {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .unwrap();

        let mut transform = Mat4::from_cols_array_2d(&transform.0);
        transform.w_axis.z = -transform.w_axis.z;

        Self { transform }
    }
}
