use league_to_lol::save_config_map;
use tokio::time::Instant;

#[tokio::main]
async fn main() {
    let root_dir = r"D:\Program Files\Riot Games\League of Legends\Game";

    let start = Instant::now();

    save_config_map(
        root_dir,
        "bloom",
        vec![("Fiora", "Skin22"), ("Hwei", "Skin0")],
    )
    .await
    .unwrap();

    let end = Instant::now();

    println!("Time taken: {:?}", end.duration_since(start));
}
