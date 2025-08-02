//! examples/league_animation.rs
//! 读取 League 骨骼 + 动画，并用 Bevy 驱动

use bevy::{
    animation::{animated_field, AnimationTarget, AnimationTargetId},
    asset::RenderAssetUsages,
    log::info,
    pbr::MeshMaterial3d,
    prelude::*,
    render::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
};
use binrw::BinRead;
use moon_lol::render::{
    AnimationData, AnimationFile, Joint, LeagueLoader, LeagueSkeleton, LeagueSkinnedMesh,
    LeagueSkinnedMeshInternal, LeagueTexture,
};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            |mut commands: Commands,
             mut res_meshes: ResMut<Assets<Mesh>>,
             mut res_materials: ResMut<Assets<StandardMaterial>>,

             mut skinned_mesh_inverse_bindposes: ResMut<Assets<SkinnedMeshInverseBindposes>>,
             mut animation_clips: ResMut<Assets<AnimationClip>>,
             mut res_image: ResMut<Assets<Image>>,
             mut animation_graphs: ResMut<Assets<AnimationGraph>>| {
                info!("--- 开始加载骨骼 ---");
                let skl_path = "assets/turret_skin44.bloom_sr_act2.skl";
                let skeleton = {
                    let file = File::open(skl_path).expect("cannot open .skl");
                    let mut reader = BufReader::new(file);
                    LeagueSkeleton::read(&mut reader).expect("parse .skl failed")
                };
                let joints = &skeleton.modern_data.joints;
                info!("成功从 '{}' 解析到 {} 个骨骼关节。", skl_path, joints.len());

                print_joint_tree(joints);

                info!("--- 开始加载动画 ---");
                let anm_path = "assets/respawn1.bloom_structure.anm";
                let animation_data: AnimationData = {
                    let file = File::open(anm_path).expect("cannot open .anm");
                    let mut reader = BufReader::new(file);
                    let anim_file = AnimationFile::read(&mut reader).expect("parse .anm failed");
                    anim_file.into()
                };
                info!(
                    "成功从 '{}' 解析到动画，包含 {} 个关节轨道。",
                    anm_path,
                    animation_data.joint_hashes.len()
                );

                let player_entity = commands
                    .spawn((Transform::from_translation(vec3(100.0, 0.0, 0.0))))
                    .id();
                info!("创建了根实体/动画播放器，ID: {:?}", player_entity);

                let mut reader =
                    BufReader::new(File::open("assets/turret_skin44.bloom_sr_act2.skn").unwrap());

                let skinned_mesh =
                    LeagueSkinnedMesh::from(LeagueSkinnedMeshInternal::read(&mut reader).unwrap());

                let mut reader = BufReader::new(
                    File::open("assets/turret_skin44_tx_cm.bloom_structure.tex").unwrap(),
                );

                let image = LeagueTexture::read(&mut reader)
                    .unwrap()
                    .to_bevy_image(RenderAssetUsages::default())
                    .unwrap();

                let texu = res_image.add(image);

                let sphere = res_meshes.add(Sphere::new(5.0));
                let mat = res_materials.add(Color::srgb(1.0, 0.2, 0.2));

                let mut index_to_entity = vec![Entity::PLACEHOLDER; joints.len()];
                let mut joint_inverse_matrix = vec![Mat4::default(); joints.len()];

                let mut clip = AnimationClip::default();

                for (i, joint) in joints.iter().enumerate() {
                    let joint_name_str = joint.name.clone();
                    let name = Name::new(joint_name_str.clone());
                    let hash = LeagueLoader::compute_joint_hash(&joint.name);

                    let target_id = AnimationTargetId::from_name(&name);

                    if let Some(anim_track_index) =
                        animation_data.joint_hashes.iter().position(|v| *v == hash)
                    {
                        info!("为关节 '{}' 添加动画曲线", joint_name_str);

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

                    let ent = commands
                        .spawn((
                            Mesh3d(sphere.clone()),
                            MeshMaterial3d(mat.clone()),
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

                for (i, joint) in joints.iter().enumerate() {
                    if joint.parent_id >= 0 {
                        let parent_entity = index_to_entity[joint.parent_id as usize];
                        commands.entity(parent_entity).add_child(index_to_entity[i]);
                    } else {
                        commands.entity(player_entity).add_child(index_to_entity[i]);
                    }
                }

                info!("已生成实体并建立父子关系。");

                let clip_handle = animation_clips.add(clip);

                let (graph, animation_node_index) = AnimationGraph::from_clip(clip_handle);
                let graph_handle = animation_graphs.add(graph);

                let mut player = AnimationPlayer::default();
                player.play(animation_node_index).repeat();

                commands
                    .entity(player_entity)
                    .insert((player, AnimationGraphHandle(graph_handle)));

                for i in 0..skinned_mesh.ranges.len() {
                    let mesh = skinned_mesh.to_bevy_mesh(i).unwrap();

                    let child = commands
                        .spawn((
                            Transform::default(),
                            Mesh3d(res_meshes.add(mesh)),
                            // Mesh3d(res_meshes.add(Sphere::new(100.0))),
                            MeshMaterial3d(res_materials.add(StandardMaterial {
                                base_color_texture: Some(texu.clone()),
                                unlit: true,
                                cull_mode: None,
                                alpha_mode: AlphaMode::Opaque,
                                ..Default::default()
                            })),
                            SkinnedMesh {
                                inverse_bindposes: skinned_mesh_inverse_bindposes.add(
                                    SkinnedMeshInverseBindposes::from(
                                        skeleton
                                            .modern_data
                                            .influences
                                            .iter()
                                            .map(|v| joint_inverse_matrix[*v as usize])
                                            .collect::<Vec<_>>(),
                                    ),
                                ),
                                joints: skeleton
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

                info!("动画设置完毕，应用启动。");

                commands.spawn((
                    Camera3d::default(),
                    Transform::from_xyz(1000.0, 1000.0, 1000.0)
                        .looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
                ));

                commands.spawn((
                    DirectionalLight {
                        shadows_enabled: true,

                        ..default()
                    },
                    Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.8, -0.5, 0.0)),
                ));
            },
        )
        .run();
}

fn print_joint_tree(joints: &[Joint]) {
    if joints.is_empty() {
        println!("关节列表为空。");
        return;
    }

    let mut parent_to_children: HashMap<i16, Vec<usize>> = HashMap::new();
    let mut roots = Vec::new();

    for (i, joint) in joints.iter().enumerate() {
        if joint.parent_id < 0 {
            roots.push(i);
        } else {
            parent_to_children
                .entry(joint.parent_id)
                .or_default()
                .push(i);
        }
    }

    for children in parent_to_children.values_mut() {
        children.sort();
    }
    roots.sort();

    println!("--- 关节层级树 ---");
    for &root_index in &roots {
        print_node(root_index, &parent_to_children, joints, "", true);
    }
    println!("--------------------");
}

fn print_node(
    index: usize,
    parent_to_children: &HashMap<i16, Vec<usize>>,
    joints: &[Joint],
    prefix: &str,
    is_last: bool,
) {
    let joint = &joints[index];

    let connector = if is_last { "└── " } else { "├── " };
    println!("{}{}{}", prefix, connector, joint.name);

    let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

    if let Some(children) = parent_to_children.get(&(index as i16)) {
        let last_child_index = children.len() - 1;
        for (i, &child_index) in children.iter().enumerate() {
            print_node(
                child_index,
                parent_to_children,
                joints,
                &new_prefix,
                i == last_child_index,
            );
        }
    }
}
