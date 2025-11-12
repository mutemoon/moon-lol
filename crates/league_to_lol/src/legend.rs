use std::collections::HashMap;

use bevy::prelude::*;

use league_core::VfxSystemDefinitionData;
use league_loader::LeagueWadLoader;

use crate::{save_character, save_wad_entry_to_file, Error};

pub async fn save_legends(
    root_dir: &str,
    champion: &str,
    skin: &str,
) -> Result<HashMap<u32, VfxSystemDefinitionData>, Error> {
    let wad_relative_path = format!("DATA/FINAL/Champions/{}.wad.client", champion);

    let loader = LeagueWadLoader::from_relative_path(&root_dir, &wad_relative_path)?;

    // let character_record_path = format!("Characters/{}/CharacterRecords/Root", character);

    let skin_path: String = format!("Characters/{}/Skins/{}", champion, skin);

    let character_record_path = format!("Characters/{}/CharacterRecords/Root", champion);

    let vfx_system_definition_datas = save_character(&loader, &skin_path, &character_record_path).await?;

    for (_, vfx_system_definition_data) in vfx_system_definition_datas.iter() {
        let Some(ref complex_emitter_definition_data) =
            vfx_system_definition_data.complex_emitter_definition_data
        else {
            continue;
        };

        for vfx_emitter_definition_data in complex_emitter_definition_data {
            if let Some(ref texture) = vfx_emitter_definition_data.texture {
                if !texture.is_empty() {
                    save_wad_entry_to_file(&loader, &texture).await?;
                }
            }
            if let Some(ref texture) = vfx_emitter_definition_data.particle_color_texture {
                if !texture.is_empty() {
                    save_wad_entry_to_file(&loader, &texture).await?;
                }
            }
            if let Some(ref texture) = vfx_emitter_definition_data.texture_mult {
                if let Some(ref texture) = texture.texture_mult {
                    if !texture.is_empty() {
                        save_wad_entry_to_file(&loader, &texture).await?;
                    }
                }
            }
        }
    }

    Ok(vfx_system_definition_datas)
}
