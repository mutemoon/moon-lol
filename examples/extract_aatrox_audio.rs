//! 临时示例：独立验证 Aatrox 音效提取管线（bnk/wpk 解析 + ww2ogg 转码 + ConfigAudio 生成），
//! 不经过 GLB/VFX 导出，快速验证 assets/characters/aatrox/skins/skin0_audio.ron 与 sounds/*.ogg。

use league_core::extract::SkinCharacterDataProperties;
use league_property::extract::get_hashes;
use league_loader::game::{Data, LeagueLoader};
use league_to_lol::extract::audio::export_audio_for_skin;
use league_to_lol::extract::{extract_phase_1_create_loader, extract_spells_for_champion};
use league_to_lol::extract::utils::write_to_file;
use lol_base::audio::ConfigAudio;
use ron::ser::{PrettyConfig, to_string_pretty};

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";
    let hashes_dir = "assets/CommunityDragon-Data/hashes/lol";

    let loader: LeagueLoader = extract_phase_1_create_loader(game_path);

    let hash_paths = vec![
        format!("{}/hashes.binentries.txt", hashes_dir),
        format!("{}/hashes.binfields.txt", hashes_dir),
        format!("{}/hashes.binhashes.txt", hashes_dir),
        format!("{}/hashes.bintypes.txt", hashes_dir),
    ];
    let hashes = get_hashes(&hash_paths.iter().map(|s| s.as_str()).collect::<Vec<_>>());

    let character_name = "Aatrox";
    let skin_bin_path = format!("data/characters/{}/skins/skin0.bin", character_name);

    let skin_prop_group = loader
        .get_prop_group_by_paths(vec![&skin_bin_path])
        .expect("无法加载 skin bin");
    let skin_data = skin_prop_group
        .get_by_class::<SkinCharacterDataProperties>()
        .expect("无法获取 SkinCharacterDataProperties");

    let all_spell_names = extract_spells_for_champion(&loader, character_name, &skin_prop_group, &hashes);
    let basic_attack_names = vec![format!("{}BasicAttack", character_name)];

    let config: ConfigAudio = export_audio_for_skin(
        &loader,
        character_name,
        "skin0",
        &skin_data,
        &all_spell_names,
        &basic_attack_names,
    );

    let output_audio_path = format!("characters/{}/skins/skin0_audio.ron", character_name);
    let serialized = to_string_pretty(&config, PrettyConfig::default()).unwrap();
    write_to_file(&output_audio_path, &serialized);

    println!("[DONE] 音频配置已写入 {}", output_audio_path);
    println!("[AUDIO] 普攻 cast={} hit={}", config.basic_attack.on_cast.len(), config.basic_attack.on_hit.len());
    for (name, cue) in &config.spells {
        println!("[AUDIO] 技能 {}: cast={} hit={}", name, cue.on_cast.len(), cue.on_hit.len());
    }
}
