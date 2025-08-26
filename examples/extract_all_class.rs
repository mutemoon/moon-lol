use moon_lol::league::{get_hashes_u64, LeagueLoader};
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

    let hash_paths = vec![
        "assets/hashes.binentries.txt",
        "assets/hashes.binfields.txt",
        "assets/hashes.binhashes.txt",
        "assets/hashes.bintypes.txt",
    ];

    let file_paths = vec!["assets/hashes.game.txt.0", "assets/hashes.game.txt.1"];

    let hashes = get_hashes_u64(&file_paths);

    let paths = hashes
        .iter()
        .filter(|(_hash, path)| {
            path.ends_with(".bin")
                && (path.contains("data/characters/") || path.contains("map11/bloom.materials.bin"))
        })
        .map(|v| v.1.as_str())
        .collect::<Vec<_>>();

    let rust_code = loader
        .extract_all_classes(&paths, &hash_paths)
        .await
        .unwrap();

    std::fs::write("league.rs", rust_code).unwrap();

    let end = Instant::now();
    println!("Time taken: {:?}", end.duration_since(start));
}
