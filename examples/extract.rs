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

    loader.map_loader.save_config_map().await.unwrap();

    loader.map_loader.save_navigation_grid().await.unwrap();

    match loader.save_legends("Fiora", "Skin44").await {
        Ok(_) => println!("Legends saved"),
        Err(e) => println!("Error saving legends: {:?}", e),
    }

    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
