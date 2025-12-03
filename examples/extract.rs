use std::collections::HashSet;

use tokio::time::Instant;

use league_core::{
    UiElementEffectAnimationData, UiElementEffectAnimationDataTextureData,
    UiElementGroupButtonData, UiElementIconData, UiElementRegionData,
};
use league_loader::{LeagueWadGroupLoader, PropBinLoader};
use league_property::from_entry;
use league_to_lol::{
    get_bin_path, save_config_map, save_struct_to_file, save_wad_entry_to_file, CONFIG_UI,
};
use lol_config::ConfigUi;

#[tokio::main]
async fn main() {
    let root_dir = r"D:\Program Files\Riot Games\League of Legends\Game";

    let start = Instant::now();

    let mut config_ui = ConfigUi::default();

    let mut textures = HashSet::new();

    let ui_loader = LeagueWadGroupLoader::from_relative_path(
        root_dir,
        vec!["DATA/FINAL/UI.wad.client", "DATA/FINAL/UI.zh_MY.wad.client"],
    );

    for wad_entry in ui_loader.iter_wad_entries() {
        let Ok(prop_bin) = ui_loader.get_prop_bin_by_hash(*wad_entry.0) else {
            continue;
        };

        for entry in prop_bin.entries.iter() {
            if let Ok(ui_element_icon_data) = from_entry::<UiElementIconData>(entry) {
                if let Some(texture_data) = ui_element_icon_data.texture_data.as_ref() {
                    match texture_data {
                        UiElementEffectAnimationDataTextureData::AtlasData(atlas_data) => {
                            if !textures.contains(&atlas_data.m_texture_name) {
                                textures.insert(atlas_data.m_texture_name.clone());
                            }
                        }
                        _ => {}
                    }
                }
                config_ui
                    .ui_elements
                    .insert(entry.hash, ui_element_icon_data);
            }

            if let Ok(ui_element_effect_animation_data) =
                from_entry::<UiElementEffectAnimationData>(entry)
            {
                match &ui_element_effect_animation_data.texture_data {
                    UiElementEffectAnimationDataTextureData::AtlasData(atlas_data) => {
                        if !textures.contains(&atlas_data.m_texture_name) {
                            textures.insert(atlas_data.m_texture_name.clone());
                        }
                    }
                    _ => {}
                }

                config_ui
                    .ui_animations
                    .insert(entry.hash, ui_element_effect_animation_data);
            }

            if let Ok(ui_element_group_button_data) = from_entry::<UiElementGroupButtonData>(entry)
            {
                config_ui
                    .ui_button_group
                    .insert(entry.hash, ui_element_group_button_data);
            }

            if let Ok(ui_element_region_data) = from_entry::<UiElementRegionData>(entry) {
                config_ui
                    .ui_region
                    .insert(entry.hash, ui_element_region_data);
            }
        }
    }

    for texture in textures {
        match save_wad_entry_to_file(&ui_loader, &texture).await {
            Ok(_) => {}
            Err(e) => println!("{}: {:?}", texture, e),
        };
    }

    save_struct_to_file(&get_bin_path(CONFIG_UI), &config_ui)
        .await
        .unwrap();

    save_config_map(
        root_dir,
        "base_srx",
        vec![("Fiora", "Skin22"), ("Hwei", "Skin0")],
    )
    .await
    .unwrap();

    let end = Instant::now();

    println!("Time taken: {:?}", end.duration_since(start));
}
