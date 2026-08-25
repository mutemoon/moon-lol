use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use bevy::prelude::*;

use crate::emitters::state::ParticleEmitterState;
use crate::particle::ParticleState;
use crate::particle::dynamic::ParticleMaterialDynamic;

/// 跨 Main World 与 Render World 共享的粒子指标计数器（通过原子计数器避免锁与开销）
#[derive(Resource, Clone, Default)]
pub struct ParticleMetricsShared {
    /// 过去统计周期内 as_bind_group 被调用的总次数
    pub as_bind_group_calls: Arc<AtomicUsize>,
    /// 过去统计周期内调用 render_device.create_buffer_with_data 创建 GPU Buffer 的数量
    pub gpu_buffers_created: Arc<AtomicUsize>,
    /// 过去统计周期内创建 GPU Buffer 的总字节数
    pub gpu_buffer_bytes: Arc<AtomicUsize>,
    /// 过去统计周期内调用 render_device.create_bind_group 创建 BindGroup 的数量
    pub bind_groups_created: Arc<AtomicUsize>,
    /// 过去统计周期内通过 DMA write_buffer 原地更新相机参数的次数
    pub dma_camera_writes: Arc<AtomicUsize>,
    /// 过去统计周期内新生成的粒子数量
    pub particles_spawned: Arc<AtomicUsize>,
}

#[derive(Resource)]
pub struct ParticleMetricsTimer {
    pub timer: Timer,
    pub frame_count: u32,
}

impl Default for ParticleMetricsTimer {
    fn default() -> Self {
        Self {
            timer: Timer::new(Duration::from_secs(1), TimerMode::Repeating),
            frame_count: 0,
        }
    }
}

pub struct PluginParticleMetrics;

impl Plugin for PluginParticleMetrics {
    fn build(&self, app: &mut App) {
        let metrics = ParticleMetricsShared::default();
        app.insert_resource(metrics.clone());
        app.init_resource::<ParticleMetricsTimer>();

        if let Some(render_app) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render_app.insert_resource(metrics);
        }

        app.add_systems(PostUpdate, log_particle_metrics_system.run_if(run_once));
    }
}

fn log_particle_metrics_system(
    time: Res<Time>,
    mut timer_res: ResMut<ParticleMetricsTimer>,
    metrics: Res<ParticleMetricsShared>,
    q_particles: Query<&ParticleState>,
    q_emitters: Query<&ParticleEmitterState>,
    res_mesh: Option<Res<Assets<Mesh>>>,
    res_materials: Option<Res<Assets<ParticleMaterialDynamic>>>,
) {
    timer_res.frame_count += 1;
    timer_res.timer.tick(time.delta());

    if !timer_res.timer.just_finished() {
        return;
    }

    let elapsed_secs = timer_res.timer.duration().as_secs_f32();
    let frames = timer_res.frame_count.max(1) as f32;
    timer_res.frame_count = 0;

    // 原子读取并重置计数器
    let bind_group_calls = metrics.as_bind_group_calls.swap(0, Ordering::Relaxed);
    let buffers_created = metrics.gpu_buffers_created.swap(0, Ordering::Relaxed);
    let buffer_bytes = metrics.gpu_buffer_bytes.swap(0, Ordering::Relaxed);
    let bind_groups_created = metrics.bind_groups_created.swap(0, Ordering::Relaxed);
    let dma_camera_writes = metrics.dma_camera_writes.swap(0, Ordering::Relaxed);
    let particles_spawned = metrics.particles_spawned.swap(0, Ordering::Relaxed);

    let active_particles = q_particles.iter().count();
    let active_emitters = q_emitters.iter().count();
    let mesh_count = res_mesh.map_or(0, |m| m.len());
    let material_count = res_materials.map_or(0, |m| m.len());

    // 当没有存活粒子/发射器，且该周期内没有新生成或 GPU 重建活动时，跳过日志输出避免刷屏
    if active_particles == 0
        && active_emitters == 0
        && particles_spawned == 0
        && bind_group_calls == 0
        && material_count == 0
    {
        return;
    }

    let spawn_rate = (particles_spawned as f32) / elapsed_secs;
    let bind_calls_per_sec = (bind_group_calls as f32) / elapsed_secs;
    let bind_calls_per_frame = (bind_group_calls as f32) / frames;
    let buffers_per_sec = (buffers_created as f32) / elapsed_secs;
    let buffers_per_frame = (buffers_created as f32) / frames;
    let buffer_kb_per_sec = (buffer_bytes as f32) / 1024.0 / elapsed_secs;
    let bg_per_sec = (bind_groups_created as f32) / elapsed_secs;
    let bg_per_frame = (bind_groups_created as f32) / frames;
    let dma_per_frame = (dma_camera_writes as f32) / frames;

    info!(
        "\n================ [PARTICLE-METRICS] 粒子系统性能指标 (周期: {:.1}s, 帧数: {}) ================\n\
         [CPU / Main World]\n\
           - 存活粒子实体 (Active Particles):    {}\n\
           - 存活发射器实体 (Active Emitters):   {}\n\
           - 新生成粒子速率 (Spawn Rate):        {:.1} 个/秒 (周期内共生成 {} 个)\n\
           - Mesh 资产总数 (Assets<Mesh>):       {}\n\
           - 动态材质资产数 (Assets<Material>):  {}\n\
         [GPU / Render World]\n\
           - as_bind_group 重建调用:             {:.1} 次/秒 (均 {:.1} 次/帧)\n\
           - GPU Uniform Buffer 新建:           {:.1} 个/秒 (均 {:.1} 个/帧, 吞吐: {:.2} KB/秒)\n\
           - BindGroup 新建:                     {:.1} 个/秒 (均 {:.1} 个/帧)\n\
           - DMA 相机参数原地更新:               {:.1} 次/秒 (均 {:.1} 次/帧 原地 write_buffer)\n\
         ==================================================================================================",
        elapsed_secs,
        frames as u32,
        active_particles,
        active_emitters,
        spawn_rate,
        particles_spawned,
        mesh_count,
        material_count,
        bind_calls_per_sec,
        bind_calls_per_frame,
        buffers_per_sec,
        buffers_per_frame,
        buffer_kb_per_sec,
        bg_per_sec,
        bg_per_frame,
        dma_camera_writes as f32 / elapsed_secs,
        dma_per_frame,
    );
}
