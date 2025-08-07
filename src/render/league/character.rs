use crate::render::{
    AnimationData, AnimationFile, LeagueLoader, LeagueSkeleton, LeagueSkinnedMesh,
    LeagueSkinnedMeshInternal,
};
use bevy::animation::{animated_field, AnimationTarget, AnimationTargetId};
use bevy::prelude::*;
use bevy::render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::{image::Image, math::Mat4};
use binrw::BinRead;
use cdragon_prop::{
    BinEmbed, BinEntry, BinFloat, BinHash, BinLink, BinMap, BinMatrix, BinString, BinStruct, BinU32,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LeagueBinCharacterRecord {
    pub character_name: Option<String>,
    pub fallback_character_name: Option<String>,
    pub base_hp: Option<f32>,
    pub base_static_hp_regen: Option<f32>,
    pub health_bar_height: Option<f32>,
    pub base_damage: Option<f32>,
    pub base_armor: Option<f32>,
    pub base_spell_block: Option<f32>,
    pub base_move_speed: Option<f32>,
    pub attack_range: Option<f32>,
    pub attack_speed: Option<f32>,
    pub attack_speed_ratio: Option<f32>,
    pub attack_speed_per_level: Option<f32>,
    pub exp_given_on_death: Option<f32>,
    pub gold_given_on_death: Option<f32>,
    pub local_gold_given_on_death: Option<f32>,
    pub global_gold_given_on_death: Option<f32>,
    pub display_name: Option<String>,
    pub hit_fx_scale: Option<f32>,
    pub selection_height: Option<f32>,
    pub selection_radius: Option<f32>,
    pub pathfinding_collision_radius: Option<f32>,
    pub gameplay_collision_radius: Option<f32>,
    pub unit_tags: Option<String>,
    pub description: Option<String>,
}

impl From<&BinEntry> for LeagueBinCharacterRecord {
    fn from(value: &BinEntry) -> Self {
        let character_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("mCharacterName").into())
            .map(|s| s.0.clone());
        let fallback_character_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("mFallbackCharacterName").into())
            .map(|s| s.0.clone());
        let base_hp = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseHP").into())
            .map(|f| f.0);
        let base_static_hp_regen = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseStaticHPRegen").into())
            .map(|f| f.0);
        let health_bar_height = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("healthBarHeight").into())
            .map(|f| f.0);
        let base_damage = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseDamage").into())
            .map(|f| f.0);
        let base_armor = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseArmor").into())
            .map(|f| f.0);
        let base_spell_block = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseSpellBlock").into())
            .map(|f| f.0);
        let base_move_speed = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("baseMoveSpeed").into())
            .map(|f| f.0);
        let attack_range = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackRange").into())
            .map(|f| f.0);
        let attack_speed = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeed").into())
            .map(|f| f.0);
        let attack_speed_ratio = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeedRatio").into())
            .map(|f| f.0);
        let attack_speed_per_level = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("attackSpeedPerLevel").into())
            .map(|f| f.0);
        let exp_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("expGivenOnDeath").into())
            .map(|f| f.0);
        let gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("goldGivenOnDeath").into())
            .map(|f| f.0);
        let local_gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("localGoldGivenOnDeath").into())
            .map(|f| f.0);
        let global_gold_given_on_death = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("globalGoldGivenOnDeath").into())
            .map(|f| f.0);
        let display_name = value
            .getv::<BinString>(LeagueLoader::hash_bin("name").into())
            .map(|s| s.0.clone());
        let hit_fx_scale = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("hitFxScale").into())
            .map(|f| f.0);
        let selection_height = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("selectionHeight").into())
            .map(|f| f.0);
        let selection_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("selectionRadius").into())
            .map(|f| f.0);
        let pathfinding_collision_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("pathfindingCollisionRadius").into())
            .map(|f| f.0);
        let gameplay_collision_radius = value
            .getv::<BinFloat>(LeagueLoader::hash_bin("overrideGameplayCollisionRadius").into())
            .map(|f| f.0);
        let unit_tags = value
            .getv::<BinString>(LeagueLoader::hash_bin("unitTagsString").into())
            .map(|s| s.0.clone());
        let description = value
            .getv::<BinString>(LeagueLoader::hash_bin("description").into())
            .map(|s| s.0.clone());

        LeagueBinCharacterRecord {
            character_name,
            fallback_character_name,
            base_hp,
            base_static_hp_regen,
            health_bar_height,
            base_damage,
            base_armor,
            base_spell_block,
            base_move_speed,
            attack_range,
            attack_speed,
            attack_speed_ratio,
            attack_speed_per_level,
            exp_given_on_death,
            gold_given_on_death,
            local_gold_given_on_death,
            global_gold_given_on_death,
            display_name,
            hit_fx_scale,
            selection_height,
            selection_radius,
            pathfinding_collision_radius,
            gameplay_collision_radius,
            unit_tags,
            description,
        }
    }
}

#[derive(Debug)]
pub struct LeagueBinMaybeCharacterMapRecord {
    pub transform: Mat4,
    pub name: u32,
    pub definition: CharacterMapRecordDefinition,
}

impl From<&BinStruct> for LeagueBinMaybeCharacterMapRecord {
    fn from(value: &BinStruct) -> Self {
        let transform = value
            .getv::<BinMatrix>(LeagueLoader::hash_bin("transform").into())
            .unwrap();

        let name = value
            .getv::<BinHash>(LeagueLoader::hash_bin("name").into())
            .unwrap();

        let definition = value
            .getv::<BinStruct>(LeagueLoader::hash_bin("definition").into())
            .unwrap();

        Self {
            transform: Mat4::from_cols_array_2d(&transform.0),
            name: name.0.hash,
            definition: definition.into(),
        }
    }
}

#[derive(Debug)]
pub struct CharacterMapRecordDefinition {
    pub team: Option<u32>,
    pub character_record: String,
    pub skin: String,
}

impl From<&BinStruct> for CharacterMapRecordDefinition {
    fn from(value: &BinStruct) -> Self {
        let team = value
            .getv::<BinU32>(LeagueLoader::hash_bin("Team").into())
            .map(|v| v.0);

        let character_record = value
            .getv::<BinString>(LeagueLoader::hash_bin("CharacterRecord").into())
            .unwrap();

        let skin = value
            .getv::<BinString>(LeagueLoader::hash_bin("Skin").into())
            .unwrap();

        Self {
            team,
            character_record: character_record.0.clone(),
            skin: skin.0.clone(),
        }
    }
}

pub struct SkinCharacterDataProperties {
    pub skin_animation_properties: SkinAnimationProperties,
    pub skin_mesh_properties: SkinMeshDataProperties,
}

impl From<&BinEntry> for SkinCharacterDataProperties {
    fn from(value: &BinEntry) -> Self {
        let skin_animation_properties = value
            .getv::<BinEmbed>(LeagueLoader::hash_bin("skinAnimationProperties").into())
            .unwrap();

        let skin_mesh_properties = value
            .getv::<BinEmbed>(LeagueLoader::hash_bin("skinMeshProperties").into())
            .unwrap();

        Self {
            skin_animation_properties: skin_animation_properties.into(),
            skin_mesh_properties: skin_mesh_properties.into(),
        }
    }
}

pub struct SkinMeshDataProperties {
    pub skeleton: String,
    pub simple_skin: String,
    pub texture: String,
}

impl From<&BinEmbed> for SkinMeshDataProperties {
    fn from(value: &BinEmbed) -> Self {
        let skeleton = value
            .getv::<BinString>(LeagueLoader::hash_bin("skeleton").into())
            .map(|v| v.0.clone())
            .unwrap();

        let simple_skin = value
            .getv::<BinString>(LeagueLoader::hash_bin("simpleSkin").into())
            .unwrap();

        let texture = value
            .getv::<BinString>(LeagueLoader::hash_bin("texture").into())
            .unwrap();

        Self {
            skeleton,
            simple_skin: simple_skin.0.clone(),
            texture: texture.0.clone(),
        }
    }
}

pub struct SkinAnimationProperties {
    pub animation_graph_data: u32,
}

impl From<&BinEmbed> for SkinAnimationProperties {
    fn from(value: &BinEmbed) -> Self {
        let animation_graph_data = value
            .getv::<BinLink>(LeagueLoader::hash_bin("animationGraphData").into())
            .map(|v| v.0.hash)
            .unwrap();

        Self {
            animation_graph_data,
        }
    }
}

pub fn load_character_record(
    loader: &LeagueLoader,
    character_record: &str,
) -> LeagueBinCharacterRecord {
    let name = character_record.split("/").nth(1).unwrap();

    let path = format!("data/characters/{0}/{0}.bin", name);

    let character_bin = loader.get_prop_bin_by_path(&path).unwrap();

    let character_record = character_bin
        .entries
        .iter()
        .find(|v| v.path.hash == LeagueLoader::hash_bin(character_record))
        .unwrap();

    return character_record.into();
}

pub fn load_character_skin(
    loader: &LeagueLoader,
    skin: &str,
) -> (SkinCharacterDataProperties, HashMap<u32, BinEntry>) {
    let skin_path = format!("data/{}.bin", skin);

    let skin_bin = loader.get_prop_bin_by_path(&skin_path).unwrap();

    let skin_mesh_properties = skin_bin
        .entries
        .iter()
        .find(|v| v.ctype.hash == LeagueLoader::hash_bin("SkinCharacterDataProperties"))
        .unwrap();

    let flat_map: HashMap<_, _> = skin_bin
        .linked_files
        .iter()
        .map(|v| loader.get_prop_bin_by_path(v).unwrap())
        .flat_map(|v| v.entries)
        .map(|v| (v.path.hash, v))
        .collect();

    (skin_mesh_properties.into(), flat_map)
}

pub fn spawn_character(
    commands: &mut Commands,
    res_animation_clips: &mut ResMut<Assets<AnimationClip>>,
    res_animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    res_image: &mut ResMut<Assets<Image>>,
    res_materials: &mut ResMut<Assets<StandardMaterial>>,
    res_meshes: &mut ResMut<Assets<Mesh>>,
    res_skinned_mesh_inverse_bindposes: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,

    loader: &LeagueLoader,
    transform: Mat4,
    skin: &str,
) {
    let (skin_character_data_properties, flat_map) = load_character_skin(loader, &skin);

    let texture = loader
        .get_image_by_texture_path(&skin_character_data_properties.skin_mesh_properties.texture)
        .unwrap();

    let mut reader = loader
        .get_wad_entry_no_seek_reader_by_path(
            &skin_character_data_properties
                .skin_mesh_properties
                .simple_skin,
        )
        .unwrap();

    let league_skinned_mesh =
        LeagueSkinnedMesh::from(LeagueSkinnedMeshInternal::read(&mut reader).unwrap());

    let league_skeleton = loader
        .get_wad_entry_reader_by_path(&skin_character_data_properties.skin_mesh_properties.skeleton)
        .map(|mut v| LeagueSkeleton::read(&mut v).unwrap())
        .unwrap();

    let gr_da: AnimationGraphData = flat_map
        .get(
            &skin_character_data_properties
                .skin_animation_properties
                .animation_graph_data,
        )
        .unwrap()
        .into();

    let idle_path = gr_da
        .clip_data_map
        .get(&0x35f43992)
        .iter()
        .filter_map(|v| match v {
            AnimationClipData::AtomicClipData {
                animation_resource_data,
            } => Some(animation_resource_data.animation_file_path.clone()),
            AnimationClipData::Unknown => None,
        })
        .collect::<Vec<_>>();
    let idle_path = idle_path.first();

    let animation_data = idle_path.map(|v| {
        loader
            .get_wad_entry_reader_by_path(v)
            .map(|mut v| AnimationData::from(AnimationFile::read(&mut v).unwrap()))
            .unwrap()
    });

    let mut clip = AnimationClip::default();

    let mut index_to_entity = vec![Entity::PLACEHOLDER; league_skeleton.modern_data.joints.len()];
    let mut joint_inverse_matrix = vec![Mat4::default(); league_skeleton.modern_data.joints.len()];

    let mut transform = Transform::from_matrix(transform);

    transform.translation.z = -transform.translation.z;

    let player_entity = commands.spawn(transform).id();

    let sphere = res_meshes.add(Sphere::new(50.0));

    let mat = res_materials.add(Color::srgb(1.0, 0.2, 0.2));

    for (i, joint) in league_skeleton.modern_data.joints.iter().enumerate() {
        let joint_name_str = joint.name.clone();
        let name = Name::new(joint_name_str.clone());
        let hash = LeagueLoader::hash_joint(&joint.name);

        let target_id = AnimationTargetId::from_name(&name);

        match animation_data {
            Some(ref animation_data) => {
                if let Some(anim_track_index) =
                    animation_data.joint_hashes.iter().position(|v| *v == hash)
                {
                    if let Some(data) = animation_data.translates.get(anim_track_index) {
                        clip.add_curve_to_target(
                            target_id,
                            AnimatableCurve::new(
                                animated_field!(Transform::translation),
                                AnimatableKeyframeCurve::new(data.clone().into_iter()).unwrap(),
                            ),
                        );
                    }

                    if let Some(data) = animation_data.rotations.get(anim_track_index) {
                        clip.add_curve_to_target(
                            target_id,
                            AnimatableCurve::new(
                                animated_field!(Transform::rotation),
                                AnimatableKeyframeCurve::new(data.clone().into_iter()).unwrap(),
                            ),
                        );
                    }

                    if let Some(data) = animation_data.scales.get(anim_track_index) {
                        clip.add_curve_to_target(
                            target_id,
                            AnimatableCurve::new(
                                animated_field!(Transform::scale),
                                AnimatableKeyframeCurve::new(data.clone().into_iter()).unwrap(),
                            ),
                        );
                    }
                }
            }
            None => {}
        }

        let ent = commands
            .spawn((
                // Mesh3d(sphere.clone()),
                // MeshMaterial3d(mat.clone()),
                Transform::from_matrix(joint.local_transform),
                name,
                AnimationTarget {
                    id: target_id,
                    player: player_entity,
                },
            ))
            .id();
        index_to_entity[i] = ent;
        joint_inverse_matrix[i] = joint.inverse_bind_transform;
    }

    for (i, joint) in league_skeleton.modern_data.joints.iter().enumerate() {
        if joint.parent_id >= 0 {
            let parent_entity = index_to_entity[joint.parent_id as usize];
            commands.entity(parent_entity).add_child(index_to_entity[i]);
        } else {
            commands.entity(player_entity).add_child(index_to_entity[i]);
        }
    }

    let texu = res_image.add(texture);

    let clip_handle = res_animation_clips.add(clip);

    let (graph, animation_node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = res_animation_graphs.add(graph);

    let mut player = AnimationPlayer::default();
    player.play(animation_node_index).repeat();

    commands
        .entity(player_entity)
        .insert((player, AnimationGraphHandle(graph_handle)));

    for i in 0..league_skinned_mesh.ranges.len() {
        let mesh = league_skinned_mesh.to_bevy_mesh(i).unwrap();

        let child = commands
            .spawn((
                Transform::default(),
                Mesh3d(res_meshes.add(mesh)),
                MeshMaterial3d(res_materials.add(StandardMaterial {
                    base_color_texture: Some(texu.clone()),
                    unlit: true,
                    cull_mode: None,
                    alpha_mode: AlphaMode::Opaque,
                    ..Default::default()
                })),
                SkinnedMesh {
                    inverse_bindposes: res_skinned_mesh_inverse_bindposes.add(
                        SkinnedMeshInverseBindposes::from(
                            league_skeleton
                                .modern_data
                                .influences
                                .iter()
                                .map(|v| joint_inverse_matrix[*v as usize])
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    joints: league_skeleton
                        .modern_data
                        .influences
                        .iter()
                        .map(|v| index_to_entity[*v as usize])
                        .collect::<Vec<_>>(),
                },
            ))
            .id();
        commands.entity(player_entity).add_child(child);
    }
}

#[derive(Debug)]
pub struct AnimationGraphData {
    clip_data_map: HashMap<u32, AnimationClipData>,
}

impl From<&BinEntry> for AnimationGraphData {
    fn from(value: &BinEntry) -> Self {
        let clip_data_map = value
            .getv::<BinMap>(LeagueLoader::hash_bin("mClipDataMap").into())
            .unwrap()
            .downcast::<BinHash, BinStruct>()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.0.hash, v.into()))
            .collect();

        Self { clip_data_map }
    }
}

#[derive(Debug)]
pub enum AnimationClipData {
    AtomicClipData {
        animation_resource_data: AnimationResourceData,
    },
    Unknown,
}

impl From<&BinStruct> for AnimationClipData {
    fn from(value: &BinStruct) -> Self {
        let hash = LeagueLoader::hash_bin("AtomicClipData");

        if value.ctype.hash == hash {
            AnimationClipData::AtomicClipData {
                animation_resource_data: value
                    .getv::<BinEmbed>(LeagueLoader::hash_bin("mAnimationResourceData").into())
                    .unwrap()
                    .into(),
            }
        } else {
            AnimationClipData::Unknown
        }
    }
}

#[derive(Debug)]
pub struct AnimationResourceData {
    animation_file_path: String,
}

impl From<&BinEmbed> for AnimationResourceData {
    fn from(value: &BinEmbed) -> Self {
        let animation_file_path = value
            .getv::<BinString>(LeagueLoader::hash_bin("mAnimationFilePath").into())
            .map(|v| v.0.clone())
            .unwrap();

        AnimationResourceData {
            animation_file_path,
        }
    }
}
