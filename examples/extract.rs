use league_to_lol::save_config_map;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    #[cfg(unix)]
    let root_dir = r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game";
    #[cfg(windows)]
    let root_dir = r"C:\Riot Games\League of Legends\Game";

    let start = Instant::now();

    save_config_map(root_dir, "bloom", vec![("Fiora", "Skin0")])
        .await
        .unwrap();

    let end = Instant::now();

    println!("Time taken: {:?}", end.duration_since(start));
}
