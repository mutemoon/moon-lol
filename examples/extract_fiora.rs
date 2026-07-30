//! 临时示例：精准重提 Fiora skin0，用于验证 skinN.ron + skinN_vfx.ron 新管线

use league_property::extract::get_hashes;
use league_to_lol::extract::{extract_character_from_record, extract_phase_1_create_loader};

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";
    let hashes_dir = "assets/CommunityDragon-Data/hashes/lol";

    let loader = extract_phase_1_create_loader(game_path);

    let hash_paths = vec![
        format!("{}/hashes.binentries.txt", hashes_dir),
        format!("{}/hashes.binfields.txt", hashes_dir),
        format!("{}/hashes.binhashes.txt", hashes_dir),
        format!("{}/hashes.bintypes.txt", hashes_dir),
    ];
    let hashes = get_hashes(&hash_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let character_name = "Fiora";
    let skin_bin_path = format!("data/characters/{}/skins/skin0.bin", character_name);
    let success = extract_character_from_record(
        &loader,
        character_name,
        true,
        None,
        Some(&skin_bin_path),
        &hashes,
    );
    println!("[DONE] Fiora 提取结果: {}", success);
}
