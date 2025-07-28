use bevy::asset::RenderAssetUsages;
use moon_lol::render::LeagueLoader;

fn main() {
    let loader = LeagueLoader::new(
        r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
        r"DATA/FINAL/Maps/Shipping/Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )
    .unwrap();

    let res = loader.get_texture_by_hash(0x85d9a737ddd65640).unwrap();

    res.to_bevy_image(RenderAssetUsages::default()).unwrap();
}
