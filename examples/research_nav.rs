use moon_lol::league::LeagueLoader;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    #[cfg(unix)]
    let loader = LeagueLoader::new(
        r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
        "bloom",
    )
    .unwrap();
    #[cfg(windows)]
    let loader =
        LeagueLoader::new(r"C:\Program Files (x86)\WeGameApps\英雄联盟\game", "bloom").unwrap();

    let start = Instant::now();

    loader.map_loader.load_navigation_grid().await.unwrap();

    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
