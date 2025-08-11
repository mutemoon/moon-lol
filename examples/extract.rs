use moon_lol::league::LeagueLoader;

fn main() {
    #[cfg(unix)]
    let loader = LeagueLoader::new(
        r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
        r"DATA/FINAL/Maps/Shipping/Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )
    .unwrap();
    #[cfg(windows)]
    let loader = LeagueLoader::new(
        r"C:\Program Files (x86)\WeGameApps\英雄联盟\game",
        r"DATA\FINAL\Maps\Shipping\Map11.wad.client",
        r"data/maps/mapgeometry/map11/bloom.mapgeo",
    )
    .unwrap();

    loader.to_configs();
}
