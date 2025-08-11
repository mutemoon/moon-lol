use crate::core::{ConfigEnvironmentObject, ConfigGeometryObject, Configs};
use crate::league::LeagueLoader;
use bevy::animation::{AnimationTarget, AnimationTargetId};
use bevy::asset::uuid::Uuid;
use bevy::math::Mat4;
use bevy::prelude::*;
use bevy::render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
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

#[derive(Debug)]
pub struct AnimationGraphData {
    pub clip_data_map: HashMap<u32, AnimationClipData>,
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
    pub animation_file_path: String,
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

/// 从Config中的ConfigEnvironmentObject生成环境对象实体
pub fn spawn_environment_object(
    commands: &mut Commands,
    res_animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    res_materials: &mut ResMut<Assets<StandardMaterial>>,
    res_skinned_mesh_inverse_bindposes: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    config_env_object: &ConfigEnvironmentObject,
) -> Entity {
    // 加载纹理
    let texture_handle = asset_server.load(config_env_object.texture_path.clone());

    // 创建父实体
    let parent_entity = commands.spawn(transform).id();

    // 构建骨骼实体映射
    let mut index_to_entity = vec![Entity::PLACEHOLDER; config_env_object.joints.len()];
    let mut joint_inverse_matrix = vec![Mat4::default(); config_env_object.joints.len()];

    // 创建骨骼实体
    for (i, joint) in config_env_object.joints.iter().enumerate() {
        let joint_name_str = joint.name.clone();
        let name = Name::new(joint_name_str.clone());
        let hash = LeagueLoader::hash_joint(&joint.name);

        let ent = commands
            .spawn((
                joint.transform,
                name,
                AnimationTarget {
                    id: AnimationTargetId(Uuid::from_u128(hash as u128)),
                    player: parent_entity,
                },
            ))
            .id();
        index_to_entity[i] = ent;
        joint_inverse_matrix[i] = joint.inverse_bind_pose;
    }

    // 建立骨骼父子关系
    for (i, joint) in config_env_object.joints.iter().enumerate() {
        if joint.parent_index >= 0 {
            let parent_entity_joint = index_to_entity[joint.parent_index as usize];
            commands
                .entity(parent_entity_joint)
                .add_child(index_to_entity[i]);
        } else {
            commands.entity(parent_entity).add_child(index_to_entity[i]);
        }
    }

    // 处理动画（如果有）
    let mut animation_player = AnimationPlayer::default();
    if !config_env_object.animation_graph.clip_paths.is_empty() {
        // 加载第一个动画剪辑作为默认动画
        let animation_clips = config_env_object
            .animation_graph
            .clip_paths
            .iter()
            .map(|v| asset_server.load(v.clone()))
            .collect::<Vec<_>>();

        let (graph, animation_node_indices) = AnimationGraph::from_clips(animation_clips);
        let graph_handle = res_animation_graphs.add(graph);

        animation_player.play(animation_node_indices[0]).repeat();

        commands
            .entity(parent_entity)
            .insert((animation_player, AnimationGraphHandle(graph_handle)));
    } else {
        commands.entity(parent_entity).insert(animation_player);
    }

    // 加载和创建mesh实体
    for submesh_path in &config_env_object.submesh_paths {
        let mesh_handle = asset_server.load(submesh_path.clone());

        let child = commands
            .spawn((
                Transform::default(),
                Mesh3d(mesh_handle),
                MeshMaterial3d(res_materials.add(StandardMaterial {
                    base_color_texture: Some(texture_handle.clone()),
                    unlit: true,
                    cull_mode: None,
                    alpha_mode: AlphaMode::Opaque,
                    ..Default::default()
                })),
                SkinnedMesh {
                    inverse_bindposes: res_skinned_mesh_inverse_bindposes.add(
                        SkinnedMeshInverseBindposes::from(
                            config_env_object
                                .joint_influences_indices
                                .iter()
                                .map(|&v| joint_inverse_matrix[v as usize])
                                .collect::<Vec<_>>(),
                        ),
                    ),
                    joints: config_env_object
                        .joint_influences_indices
                        .iter()
                        .map(|&v| index_to_entity[v as usize])
                        .collect::<Vec<_>>(),
                },
            ))
            .id();
        commands.entity(parent_entity).add_child(child);
    }

    parent_entity
}

/// 从Configs批量生成所有环境对象
pub fn spawn_environment_objects_from_configs(
    commands: &mut Commands,
    res_animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    res_materials: &mut ResMut<Assets<StandardMaterial>>,
    res_skinned_mesh_inverse_bindposes: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
    asset_server: &Res<AssetServer>,
    configs: &Configs,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for (transform, config_env_object, _) in &configs.environment_objects {
        let entity = spawn_environment_object(
            commands,
            res_animation_graphs,
            res_materials,
            res_skinned_mesh_inverse_bindposes,
            asset_server,
            *transform,
            config_env_object,
        );
        entities.push(entity);
    }

    entities
}

/// 从Config中的ConfigGeometryObject生成几何对象实体
pub fn spawn_geometry_object(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    transform: Transform,
    config_geo_object: &ConfigGeometryObject,
) -> Entity {
    // 加载纹理
    let material_handle: Handle<StandardMaterial> =
        asset_server.load(config_geo_object.material_path.clone());

    // 加载网格
    let mesh_handle = asset_server.load(config_geo_object.mesh_path.clone());

    // 创建几何对象实体
    commands
        .spawn((
            transform,
            Mesh3d(mesh_handle),
            MeshMaterial3d(material_handle),
        ))
        .id()
}

/// 从Configs批量生成所有几何对象
pub fn spawn_geometry_objects_from_configs(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    configs: &Configs,
) -> Vec<Entity> {
    let mut entities = Vec::new();

    for config_geo_object in &configs.geometry_objects {
        let entity = spawn_geometry_object(
            commands,
            asset_server,
            Transform::default(), // 几何对象使用默认变换
            config_geo_object,
        );
        entities.push(entity);
    }

    entities
}
