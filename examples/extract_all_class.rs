use moon_lol::league::LeagueLoader;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    #[cfg(unix)]
    let loader = LeagueLoader::new(r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game").unwrap();
    #[cfg(windows)]
    let loader = LeagueLoader::new(r"C:\Program Files (x86)\WeGameApps\英雄联盟\game").unwrap();

    let map_loader = loader.get_map_loader("bloom").unwrap();

    let start = Instant::now();

    let hash_paths = vec![
        "assets/hashes.binentries.txt",
        "assets/hashes.binfields.txt",
        "assets/hashes.binhashes.txt",
        "assets/hashes.bintypes.txt",
    ];

    let rust_code = map_loader
        .extract_all_map_classes(&hash_paths)
        .await
        .unwrap();

    std::fs::write("map.rs", rust_code).unwrap();

    let rust_code = map_loader.extract_all_classes(&hash_paths).await.unwrap();

    std::fs::write("character.rs", rust_code).unwrap();

    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
