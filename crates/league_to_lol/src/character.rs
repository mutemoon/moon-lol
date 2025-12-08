use std::collections::HashMap;

use bevy::prelude::*;

use league_core::{
    ResourceResolver, SkinCharacterDataProperties, VfxEmitterDefinitionDataPrimitive,
    VfxSystemDefinitionData,
};
use league_loader::{LeagueWadLoader, PropBinLoader};
use league_property::from_entry_unwrap;
use league_utils::hash_bin;
// use league_file::InibinFile;

use crate::{save_wad_entry_to_file, Error};

pub async fn save_character(
    loader: &LeagueWadLoader,
    skin: &str,
) -> Result<HashMap<u32, VfxSystemDefinitionData>, Error> {
    let skin_bin = loader.get_skin_bin_by_path(skin).unwrap();

    let skin_character_data_properties = from_entry_unwrap::<SkinCharacterDataProperties>(
        skin_bin
            .iter_entry_by_class(hash_bin("SkinCharacterDataProperties"))
            .next()
            .unwrap(),
    );

    let resource_resolver = skin_bin
        .iter_entry_by_class(hash_bin("ResourceResolver"))
        .next()
        .map(|v| from_entry_unwrap::<ResourceResolver>(v));

    let flat_map: HashMap<_, _> = skin_bin
        .links
        .iter()
        .map(|v| loader.get_prop_bin_by_path(&v.text).unwrap())
        .flat_map(|v| v.entries)
        .chain(skin_bin.entries.into_iter())
        .map(|v| (v.hash, v))
        .collect();

    let mut vfx_system_definition_datas = HashMap::new();
    if let Some(Some(resource_map)) = resource_resolver.map(|v| v.resource_map) {
        for (hash, link) in resource_map {
            let Some(entry_data) = flat_map.get(&link) else {
                continue;
            };
            let vfx_system_definition_data =
                from_entry_unwrap::<VfxSystemDefinitionData>(entry_data);

            if let Some(ref complex_emitter_definition_data) =
                vfx_system_definition_data.complex_emitter_definition_data
            {
                for v in complex_emitter_definition_data {
                    let Some(primitive) = &v.primitive else {
                        continue;
                    };

                    let VfxEmitterDefinitionDataPrimitive::VfxPrimitiveMesh(vfx_primitive_mesh) =
                        primitive
                    else {
                        continue;
                    };

                    let Some(m_mesh) = vfx_primitive_mesh.m_mesh.as_ref() else {
                        continue;
                    };

                    let Some(simple_mesh_name) = m_mesh.m_simple_mesh_name.as_ref() else {
                        continue;
                    };

                    save_wad_entry_to_file(loader, simple_mesh_name).await?;
                }
            };

            vfx_system_definition_datas.insert(hash, vfx_system_definition_data);
        }
    }

    for (_, vfx_system_definition_data) in vfx_system_definition_datas.iter() {
        let Some(ref complex_emitter_definition_data) =
            vfx_system_definition_data.complex_emitter_definition_data
        else {
            continue;
        };

        for vfx_emitter_definition_data in complex_emitter_definition_data {
            if let Some(ref texture) = vfx_emitter_definition_data.texture {
                if !texture.is_empty() {
                    save_wad_entry_to_file(loader, &texture).await?;
                }
            }
            if let Some(ref texture) = vfx_emitter_definition_data.particle_color_texture {
                if !texture.is_empty() {
                    save_wad_entry_to_file(loader, &texture).await?;
                }
            }
            if let Some(ref texture) = vfx_emitter_definition_data.texture_mult {
                if let Some(ref texture) = texture.texture_mult {
                    if !texture.is_empty() {
                        save_wad_entry_to_file(loader, &texture).await?;
                    }
                }
            }
        }
    }

    if let Some(icon_avatar) = skin_character_data_properties.icon_avatar.clone() {
        save_wad_entry_to_file(loader, &icon_avatar).await?;
    }

    // 保存动画文件
    // for (_, animation) in &animation_map {
    //     match animation {
    //         AnimationGraphDataMClipDataMap::AtomicClipData(AtomicClipData {
    //             m_animation_resource_data,
    //             ..
    //         }) => {
    //             let clip_path = &m_animation_resource_data.m_animation_file_path;
    //             let mut animation_file = loader.get_wad_entry_reader_by_path(&clip_path)?;
    //             let animation_file = AnimationFile::read(&mut animation_file)?;
    //             let animation_data = load_animation_file(animation_file);
    //             save_struct_to_file(&clip_path, &animation_data).await?;
    //         }
    //         _ => {}
    //     }
    // }

    Ok(vfx_system_definition_datas)
}
