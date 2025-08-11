use moon_lol::league::LeagueLoader;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
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

    let start = Instant::now();
    loader.save_configs().await.unwrap();
    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
