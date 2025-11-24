use std::collections::HashMap;

use bevy::prelude::*;

use league_core::VfxSystemDefinitionData;
use league_loader::LeagueWadLoader;

use crate::{save_character, Error};

pub async fn save_legends(
    root_dir: &str,
    champion: &str,
    skin: &str,
    data_loader: &LeagueWadLoader,
) -> Result<HashMap<u32, VfxSystemDefinitionData>, Error> {
    let wad_relative_path = format!("DATA/FINAL/Champions/{}.wad.client", champion);

    let loader = LeagueWadLoader::from_relative_path(&root_dir, &wad_relative_path)?;

    let skin_path: String = format!("Characters/{}/Skins/{}", champion, skin);

    let character_record_path = format!("Characters/{}/CharacterRecords/Root", champion);

    let vfx_system_definition_datas =
        save_character(&data_loader, &loader, &skin_path, &character_record_path).await?;

    Ok(vfx_system_definition_datas)
}
