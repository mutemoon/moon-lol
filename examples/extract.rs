use tokio::time::Instant;

#[tokio::main]
async fn main() {
    // let root_dir = r"D:\Program Files\Riot Games\League of Legends\Game";

    let start = Instant::now();

    println!("Time taken: {:?}", start.elapsed());
}
