use crate::league::{
    AnimationClipData, AnimationData, AnimationFile, AnimationGraphData, LeagueLoader,
    LeagueSkeleton, SkinCharacterDataProperties,
};
use bevy::prelude::*;
use bevy::render::mesh::skinning::SkinnedMeshInverseBindposes;
use binrw::BinRead;
use cdragon_prop::BinEntry;
use std::collections::HashMap;

/// 缓存的角色资源数据
pub struct CachedCharacterResources {
    pub texture_handle: Handle<Image>,
    pub mesh_handles: Vec<Handle<Mesh>>,
    pub material_handle: Handle<StandardMaterial>,
    pub skeleton_data: LeagueSkeleton,
    pub animation_clip_handle: Option<Handle<AnimationClip>>,
    pub animation_graph_handle: Option<Handle<AnimationGraph>>,
    pub inverse_bindposes_handle: Handle<SkinnedMeshInverseBindposes>,
}

/// 全局资源缓存，按皮肤路径存储
#[derive(Resource, Default)]
pub struct CharacterResourceCache {
    cache: HashMap<String, CachedCharacterResources>,
}

impl CharacterResourceCache {
    /// 获取或创建角色资源
    // pub fn get_or_create_character_resources(
    //     &mut self,
    //     skin: &str,
    //     loader: &LeagueLoader,
    //     res_animation_clips: &mut ResMut<Assets<AnimationClip>>,
    //     res_animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    //     res_image: &mut ResMut<Assets<Image>>,
    //     res_materials: &mut ResMut<Assets<StandardMaterial>>,
    //     res_meshes: &mut ResMut<Assets<Mesh>>,
    //     res_skinned_mesh_inverse_bindposes: &mut ResMut<Assets<SkinnedMeshInverseBindposes>>,
    // ) -> &CachedCharacterResources {
    //
    // }

    fn create_animation_resources(
        skin_character_data_properties: &SkinCharacterDataProperties,
        flat_map: &HashMap<u32, BinEntry>,
        league_skeleton: &LeagueSkeleton,
        loader: &LeagueLoader,
        res_animation_clips: &mut ResMut<Assets<AnimationClip>>,
        res_animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    ) -> (
        Option<Handle<AnimationClip>>,
        Option<Handle<AnimationGraph>>,
    ) {
        use bevy::animation::{animated_field, AnimationTargetId};

        let animation_graph_data: AnimationGraphData = flat_map
            .get(
                &skin_character_data_properties
                    .skin_animation_properties
                    .animation_graph_data,
            )
            .map(|entry| entry.into())
            .unwrap_or_else(|| AnimationGraphData {
                clip_data_map: HashMap::new(),
            });

        let idle_path = animation_graph_data
            .clip_data_map
            .get(&0x35f43992)
            .and_then(|v| match v {
                AnimationClipData::AtomicClipData {
                    animation_resource_data,
                } => Some(&animation_resource_data.animation_file_path),
                AnimationClipData::Unknown => None,
            });

        let animation_data = idle_path.and_then(|v| {
            loader
                .get_wad_entry_reader_by_path(v)
                .map(|mut reader| AnimationData::from(AnimationFile::read(&mut reader).unwrap()))
                .ok()
        });

        if let Some(animation_data) = animation_data {
            let mut clip = AnimationClip::default();

            for (_i, joint) in league_skeleton.modern_data.joints.iter().enumerate() {
                let joint_name_str = joint.name.clone();
                let name = Name::new(joint_name_str.clone());
                let hash = LeagueLoader::hash_joint(&joint.name);
                let target_id = AnimationTargetId::from_name(&name);

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

            let clip_handle = res_animation_clips.add(clip);
            let (graph, _animation_node_index) = AnimationGraph::from_clip(clip_handle.clone());
            let graph_handle = res_animation_graphs.add(graph);

            (Some(clip_handle), Some(graph_handle))
        } else {
            (None, None)
        }
    }
}
