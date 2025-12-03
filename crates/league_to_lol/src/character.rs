use std::collections::HashMap;

use bevy::prelude::*;
use binrw::BinRead;

use league_core::{
    AnimationGraphDataMClipDataMap, AtomicClipData, CharacterRecord, SpellObject,
    VfxEmitterDefinitionDataPrimitive, VfxSystemDefinitionData,
};
use league_file::{AnimationFile, LeagueSkeleton, LeagueSkinnedMesh};
use league_loader::{LeagueWadLoader, PropBinLoader};
use league_property::from_entry_unwrap;
use league_utils::{hash_bin, hash_joint};
use lol_config::{
    ConfigCharacterSkin, ConfigJoint, ConfigSkinnedMeshInverseBindposes, LeagueMaterial,
};
// use league_file::InibinFile;

use crate::{
    get_bin_path, get_character_record_save_path, get_character_spell_objects_save_path,
    load_animation_file, load_animation_map, save_struct_to_file, save_wad_entry_to_file,
    skinned_mesh_to_intermediate, Error,
};

pub async fn save_character(
    _data_loader: &LeagueWadLoader,
    loader: &LeagueWadLoader,
    skin: &str,
    character_record_path: &str,
) -> Result<HashMap<u32, VfxSystemDefinitionData>, Error> {
    let (skin_character_data_properties, resource_resolver, flat_map) =
        loader.load_character_skin(&skin);

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

    let skin_mesh_properties = &skin_character_data_properties.skin_mesh_properties.unwrap();

    let texture = skin_mesh_properties.texture.clone().unwrap();
    save_wad_entry_to_file(loader, &texture).await?;

    let material = LeagueMaterial {
        texture_path: texture.clone(),
    };
    let material_path = get_bin_path(&format!("ASSETS/{}/material", skin));
    save_struct_to_file(&material_path, &material).await?;

    let skeleton = skin_mesh_properties.skeleton.clone().unwrap();
    save_wad_entry_to_file(loader, &skeleton).await?;

    let league_skeleton = loader
        .get_wad_entry_reader_by_path(&skeleton)
        .map(|mut v| LeagueSkeleton::read(&mut v).unwrap())
        .unwrap();

    let simple_skin = skin_mesh_properties.simple_skin.clone().unwrap();
    let mut reader = loader
        .get_wad_entry_no_seek_reader_by_path(&simple_skin)
        .unwrap();
    let league_simple_mesh = LeagueSkinnedMesh::read(&mut reader).unwrap();

    let (animation_map, blend_data) = load_animation_map(
        flat_map
            .get(
                &skin_character_data_properties
                    .skin_animation_properties
                    .animation_graph_data,
            )
            .unwrap(),
    )?;

    // 保存动画文件
    for (_, animation) in &animation_map {
        match animation {
            AnimationGraphDataMClipDataMap::AtomicClipData(AtomicClipData {
                m_animation_resource_data,
                ..
            }) => {
                let clip_path = &m_animation_resource_data.m_animation_file_path;
                let mut animation_file = loader.get_wad_entry_reader_by_path(&clip_path)?;
                let animation_file = AnimationFile::read(&mut animation_file)?;
                let animation_data = load_animation_file(animation_file);
                save_struct_to_file(&clip_path, &animation_data).await?;
            }
            _ => {}
        }
    }

    let mut submesh_paths = Vec::new();

    for (i, range) in league_simple_mesh.ranges.iter().enumerate() {
        let mesh = skinned_mesh_to_intermediate(&league_simple_mesh, i);
        let mesh_path = format!("ASSETS/{}/meshes/{}.mesh", skin, &range.name);
        save_struct_to_file(&mesh_path, &mesh).await?;

        submesh_paths.push(mesh_path);
    }

    let inverse_bind_poses = league_skeleton
        .modern_data
        .influences
        .iter()
        .map(|&v| league_skeleton.modern_data.joints[v as usize].inverse_bind_transform)
        .collect::<Vec<_>>();

    let inverse_bind_pose_path = get_bin_path(&format!("ASSETS/{}/inverse_bind_pose", skin));
    save_struct_to_file(
        &inverse_bind_pose_path,
        &ConfigSkinnedMeshInverseBindposes {
            inverse_bindposes: inverse_bind_poses,
        },
    )
    .await?;

    let path = get_bin_path(&format!("ASSETS/{}/config_character_skin", skin));
    let config_character_skin = ConfigCharacterSkin {
        skin_scale: skin_mesh_properties.skin_scale,
        material_path,
        submesh_paths,
        joint_influences_indices: league_skeleton.modern_data.influences,
        inverse_bind_pose_path,
        joints: league_skeleton
            .modern_data
            .joints
            .iter()
            .map(|joint| ConfigJoint {
                hash: hash_joint(&joint.name),
                transform: Transform::from_matrix(joint.local_transform),
                parent_index: joint.parent_index,
            })
            .collect(),
        animation_map,
        blend_data,
        icon_avatar_path: skin_character_data_properties.icon_avatar.clone(),
    };
    save_struct_to_file(&path, &config_character_skin).await?;

    let name = character_record_path.split("/").nth(1).unwrap();

    let character_bin_path = format!("data/characters/{0}/{0}.bin", name);

    let character_bin = loader.get_prop_bin_by_path(&character_bin_path).unwrap();

    let character_record: CharacterRecord = from_entry_unwrap(
        character_bin
            .entries
            .iter()
            .find(|v| v.hash == hash_bin(&character_record_path))
            .unwrap(),
    );

    // 保存 character_record
    let character_record_save_path =
        get_bin_path(&get_character_record_save_path(&character_record_path));
    save_struct_to_file(&character_record_save_path, &character_record).await?;

    let spell_objects = character_bin
        .iter_entry_by_class(hash_bin(&"SpellObject"))
        .map(|v| (v.hash, from_entry_unwrap::<SpellObject>(v)))
        .collect::<HashMap<_, _>>();

    for spell_object in spell_objects.values() {
        let Some(m_img_icon_name) = spell_object
            .m_spell
            .as_ref()
            .and_then(|v| v.m_img_icon_name.as_ref())
            .and_then(|v| v.get(0))
        else {
            continue;
        };

        if m_img_icon_name.is_empty() {
            continue;
        }

        save_wad_entry_to_file(loader, m_img_icon_name).await?;

        // let Some(missile_effect_name) = spell_object
        //     .m_spell
        //     .as_ref()
        //     .and_then(|v| v.m_missile_effect_name.as_ref())
        // else {
        //     continue;
        // };

        // let path = format!("data/particles/{}bin", missile_effect_name);

        // let mut reader = data_loader.get_wad_entry_reader_by_path(&path).unwrap();
        // let inibin = InibinFile::read(&mut reader).unwrap();
    }

    let spell_objects_save_path = get_bin_path(&get_character_spell_objects_save_path(&name));
    save_struct_to_file(&spell_objects_save_path, &spell_objects).await?;

    Ok(vfx_system_definition_datas)
}
