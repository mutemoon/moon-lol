//! 一次性提取所有英雄的音效配置与音频文件（bnk/wpk 解析 + ww2ogg 转码 + ConfigAudio 生成），
//! 并自动向各英雄的 skin0.ron 挂载 AudioBank 组件。

use std::fs;

use league_core::extract::SkinCharacterDataProperties;
use league_loader::game::{Data, LeagueLoader};
use league_to_lol::extract::audio::export_audio_for_skin;
use league_to_lol::extract::extract_phase_1_create_loader;
use league_to_lol::extract::utils::write_to_file;
use lol_base::audio::ConfigAudio;
use ron::ser::{PrettyConfig, to_string_pretty};

fn main() {
    let game_path = r"D:\WeGameApps\英雄联盟\Game";

    println!("[AUDIO] 初始化资源加载器...");
    let loader: LeagueLoader = extract_phase_1_create_loader(game_path);

    // 扫描 assets/characters 下所有英雄目录
    let mut character_dirs = Vec::new();
    if let Ok(entries) = fs::read_dir("assets/characters") {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                // 排除非英雄目录（如地图单位/建筑/野怪等）
                if !name.starts_with("SRU")
                    && !name.starts_with("sru")
                    && name != "Locke"
                    && name != "inhibitor"
                    && name != "nexus"
                    && name != "turret"
                    && name != "tftchampion"
                {
                    character_dirs.push(name);
                }
            }
        }
    }
    character_dirs.sort();

    println!(
        "[AUDIO] 共找到 {} 个英雄目录，准备批量提取音效...",
        character_dirs.len()
    );

    let mut success_count = 0;
    for (idx, champ_dir) in character_dirs.iter().enumerate() {
        let lower_name = champ_dir.to_lowercase();
        // 格式化为 首字母大写（如 aatrox -> Aatrox），以便匹配技能/事件名前缀
        let display_name = format!("{}{}", lower_name[..1].to_uppercase(), &lower_name[1..]);

        println!(
            "\n[{}/{}] 正在处理音效: {}",
            idx + 1,
            character_dirs.len(),
            display_name
        );
        let skin_bin_path = format!("data/characters/{}/skins/skin0.bin", lower_name);

        let Ok(skin_prop_group) = loader.get_prop_group_by_paths(vec![&skin_bin_path]) else {
            eprintln!("  [SKIP] 无法加载 skin bin: {}", skin_bin_path);
            continue;
        };
        let Some(skin_data) = skin_prop_group.get_by_class::<SkinCharacterDataProperties>() else {
            eprintln!(
                "  [SKIP] 无法获取 SkinCharacterDataProperties: {}",
                display_name
            );
            continue;
        };

        let config: ConfigAudio =
            export_audio_for_skin(&loader, &display_name, "skin0", &skin_data);

        let output_audio_path = format!("characters/{}/skins/skin0_audio.ron", champ_dir);
        let serialized = to_string_pretty(&config, PrettyConfig::default()).unwrap();
        write_to_file(&output_audio_path, &serialized);

        // 确保 skin0.ron 中已挂载 AudioBank 组件
        ensure_audio_bank_in_skin0(champ_dir);

        success_count += 1;
        println!(
            "  [DONE] 音频配置已写入 {} (包含 {} 个音效事件)",
            output_audio_path,
            config.events.len()
        );
    }

    println!(
        "\n[ALL DONE] 完成全英雄音效提取: 成功 {}/{}",
        success_count,
        character_dirs.len()
    );
}

fn ensure_audio_bank_in_skin0(champ_dir: &str) {
    let skin0_path = format!("assets/characters/{}/skins/skin0.ron", champ_dir);
    let Ok(content) = fs::read_to_string(&skin0_path) else {
        return;
    };
    if content.contains("AudioBank") {
        return;
    }
    let bank_str = format!(
        "        \"lol_base::audio::AudioBank\": (Path(\"characters/{}/skins/skin0_audio.ron\")),\n",
        champ_dir
    );
    if let Some(pos) = content.find("components: {") {
        let insert_pos = pos + "components: {\n".len();
        let mut new_content = content.clone();
        new_content.insert_str(insert_pos, &bank_str);
        let _ = fs::write(&skin0_path, new_content);
    }
}
