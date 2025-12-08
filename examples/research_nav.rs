use league_loader::LeagueWadMapLoader;
use league_to_lol::load_navigation_grid;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    #[cfg(unix)]
    let root_dir = r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game";
    #[cfg(windows)]
    let root_dir = r"C:\Program Files (x86)\WeGameApps\英雄联盟\game";

    let loader = LeagueWadMapLoader::from_loader(root_dir, "bloom").unwrap();

    let start = Instant::now();

    load_navigation_grid(&loader).await.unwrap();

    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
