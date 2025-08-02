//! examples/league_animation.rs
//! 读取 League 骨骼 + 动画，并用 Bevy 驱动

// 引入 Bevy 的日志宏
use bevy::{
    log::{info, warn},
    pbr::MeshMaterial3d,
    prelude::*,
};
use binrw::BinRead;
use moon_lol::render::{AnimationData, AnimationFile, LeagueLoader, LeagueSkeleton};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufReader;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, animate_joints)
        .run();
}

/// 运行时组件：把动画数据挂在实体上，方便每一帧更新
#[derive(Component)]
struct AnimatedJoint {
    /// 对应动画轨道（joint hash）
    joint_hash: u32,
    /// 统一的动画数据引用（Arc 避免复制）
    anim: std::sync::Arc<AnimationData>,
    /// 当前播放时间
    time: f32,
}

/// Startup 系统：解析文件并生成骨骼实体
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. 解析骨骼 ----------------------------------------------------------
    info!("--- 开始加载骨骼 ---");
    let skl_path = "assets/turret_skin44.bloom_sr_act2.skl";
    let skeleton = {
        let file = File::open(skl_path).expect("cannot open .skl");
        let mut reader = BufReader::new(file);
        LeagueSkeleton::read(&mut reader).expect("parse .skl failed")
    };
    let joints = &skeleton.modern_data.joints;
    info!("成功从 '{}' 解析到 {} 个骨骼关节。", skl_path, joints.len());
    info!(
        "{:?}",
        joints.iter().map(|v| &v.name).collect::<Vec<&String>>()
    );

    joints
        .iter()
        .map(|v| LeagueLoader::compute_joint_hash(&v.name))
        .for_each(|v| info!("{:x}", v));

    // 2. 解析动画（使用统一的动画数据结构）-------------------------------------
    info!("--- 开始加载动画 ---");
    let anm_path = "assets/respawn1.bloom_structure.anm";
    let animation_data = {
        let file = File::open(anm_path).expect("cannot open .anm");
        let mut reader = BufReader::new(file);
        let anim_file = AnimationFile::read(&mut reader).expect("parse .anm failed");
        anim_file.into_unified()
    };
    info!(
        "成功从 '{}' 解析到动画，包含 {} 个关节轨道。",
        anm_path,
        animation_data.joint_hashes.len()
    );

    // [调试日志] 比较骨骼和动画的 Hashes
    let skl_hashes_with_names: HashMap<u32, String> = joints
        .iter()
        .map(|j| {
            let hash = moon_lol::render::LeagueLoader::compute_joint_hash(&j.name);
            (hash, j.name.clone())
        })
        .collect();

    let anm_hashes: HashSet<u32> = animation_data.joint_hashes.iter().cloned().collect();

    info!("--- 哈希匹配检查 ---");
    info!("骨骼中的关节 Hashes ({} 个):", skl_hashes_with_names.len());
    for (hash, name) in &skl_hashes_with_names {
        info!("  - Hash: {}, Name: {}", hash, name);
    }
    info!(
        "动画中的关节 Hashes ({} 个): {:?}",
        anm_hashes.len(),
        anm_hashes
    );

    let mut match_count = 0;
    info!("--- 匹配结果 ---");
    for (hash, name) in &skl_hashes_with_names {
        if anm_hashes.contains(hash) {
            info!(
                "[匹配成功] 关节 '{}' (hash: {}) 在动画中找到了轨道。",
                name, hash
            );
            match_count += 1;
        } else {
            warn!(
                "[匹配失败] 关节 '{}' (hash: {}) 在动画中没有找到轨道。",
                name, hash
            );
        }
    }
    if match_count == 0 {
        warn!("严重警告：没有任何一个骨骼关节在动画数据中找到匹配！动画将不会播放。");
    } else {
        info!("共有 {} 个关节匹配成功。", match_count);
    }
    info!("--------------------");

    // 3. 建立骨骼实体，同时记录 hash -> Entity 映射 -------------------------
    let sphere = meshes.add(Sphere::new(5.0));
    let mat = materials.add(Color::srgb(1.0, 0.2, 0.2));

    let mut index_to_entity = vec![Entity::PLACEHOLDER; joints.len()];
    let animation_data_arc = std::sync::Arc::new(animation_data);

    for (i, joint) in joints.iter().enumerate() {
        let hash = moon_lol::render::LeagueLoader::compute_joint_hash(&joint.name);
        let ent = commands
            .spawn((
                Mesh3d(sphere.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_matrix(joint.local_transform),
                Name::new(joint.name.clone()),
                // 动画组件
                AnimatedJoint {
                    joint_hash: hash,
                    anim: animation_data_arc.clone(),
                    time: 0.0,
                },
            ))
            .id();
        index_to_entity[i] = ent;
    }

    // 建立父子关系
    for (i, joint) in joints.iter().enumerate() {
        if joint.parent_id >= 0 {
            let parent = index_to_entity[joint.parent_id as usize];
            commands.entity(parent).add_child(index_to_entity[i]);
        }
    }
    info!("已生成实体并建立父子关系。");

    // 4. 相机/灯光 ---------------------------------------------------------
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-250.0, 1000.0, 1000.0).looking_at(Vec3::new(0.0, 50.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::YXZ, -0.8, -0.5, 0.0)),
    ));
    info!("场景设置完毕，应用启动。");
}

/// 每帧更新：根据动画数据改写 Transform
fn animate_joints(
    mut query: Query<(&Name, &mut Transform, &mut AnimatedJoint)>, // 添加 Name 以便日志记录
    time: Res<Time>,
) {
    // [调试日志] 确认系统正在运行
    // info!("'animate_joints' system is running at time: {:.2}", time.elapsed_seconds());

    for (name, mut transform, mut animated) in &mut query {
        animated.time += time.delta_secs();

        let anim = &animated.anim;
        let total_duration = anim.duration;
        let t = animated.time % total_duration;

        // 找到该关节的所有帧数据
        let joint_frames: Vec<&_> = anim
            .frames
            .iter()
            .filter(|frame| frame.joint_hash == animated.joint_hash)
            .collect();

        if joint_frames.is_empty() {
            // 这个关节没有动画数据，跳过
            continue;
        }

        // 根据时间找到当前帧和下一帧
        let mut current_frame = joint_frames[0];
        let mut next_frame = joint_frames[0];

        for i in 0..joint_frames.len() {
            if joint_frames[i].time <= t {
                current_frame = joint_frames[i];
                next_frame = if i + 1 < joint_frames.len() {
                    joint_frames[i + 1]
                } else {
                    joint_frames[0] // 循环到第一帧
                };
            } else {
                break;
            }
        }

        // 计算插值因子
        let time_diff = if next_frame.time > current_frame.time {
            next_frame.time - current_frame.time
        } else {
            // 处理循环情况（从最后一帧到第一帧）
            (total_duration - current_frame.time) + next_frame.time
        };

        let factor = if time_diff > 0.0 {
            let elapsed = if t >= current_frame.time {
                t - current_frame.time
            } else {
                (total_duration - current_frame.time) + t
            };
            (elapsed / time_diff).clamp(0.0, 1.0)
        } else {
            0.0
        };

        // 从调色板中获取变换数据并进行插值
        let trans = if current_frame.translation_id < anim.vector_palette.len() as u16
            && next_frame.translation_id < anim.vector_palette.len() as u16
        {
            Vec3::lerp(
                anim.vector_palette[current_frame.translation_id as usize].0,
                anim.vector_palette[next_frame.translation_id as usize].0,
                factor,
            )
        } else {
            Vec3::ZERO
        };

        let rot = if current_frame.rotation_id < anim.quat_palette.len() as u16
            && next_frame.rotation_id < anim.quat_palette.len() as u16
        {
            Quat::slerp(
                anim.quat_palette[current_frame.rotation_id as usize].0,
                anim.quat_palette[next_frame.rotation_id as usize].0,
                factor,
            )
        } else {
            Quat::IDENTITY
        };

        let scale = if current_frame.scale_id < anim.vector_palette.len() as u16
            && next_frame.scale_id < anim.vector_palette.len() as u16
        {
            Vec3::lerp(
                anim.vector_palette[current_frame.scale_id as usize].0,
                anim.vector_palette[next_frame.scale_id as usize].0,
                factor,
            )
        } else {
            Vec3::ONE
        };

        // [调试日志] 打印特定关节的详细信息，以防日志刷屏
        if name.as_str() == "Root" && (animated.time as i32 % 1 == 0) {
            info!(
                "关节: {}, hash: {:x}, t: {:.2}, frames: {}, factor: {:.2}",
                name,
                animated.joint_hash,
                t,
                joint_frames.len(),
                factor
            );
            info!(
                "  -> pos: ({:.2}, {:.2}, {:.2}), rot: ({:.2}, {:.2}, {:.2}, {:.2})",
                trans.x, trans.y, trans.z, rot.x, rot.y, rot.z, rot.w
            );
            info!(
                "  -> current_frame.time: {:.2}, next_frame.time: {:.2}, duration: {:.2}",
                current_frame.time, next_frame.time, total_duration
            );
        }

        *transform = Transform::from_translation(trans)
            .with_rotation(rot)
            .with_scale(scale);
    }
}
