use binrw::BinRead;
use moon_lol::render::LeagueMapGeo;
use std::fs::File;

fn main() {
    let path = "assets/extract_data/base_srx.mapgeo";

    let Ok(mut reader) = File::open(path) else {
        return;
    };

    let start = std::time::Instant::now();

    match LeagueMapGeo::read(&mut reader) {
        Ok(_) => {
            println!("loaded in {:?}", start.elapsed());
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    };
}
