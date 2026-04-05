use bevy::ecs::entity::EntityHashMap;
use bevy::prelude::*;

#[derive(Component, Reflect, Default, Debug)]
#[reflect(Component)]
pub struct MobConfig {
    pub hp: f32,
    pub speed: f32,
    pub name: String,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // 关键！不注册类型，RON 就无法解析
        .register_type::<MobConfig>()
        .add_systems(Startup, setup)
        .add_systems(Update, apply_config_system)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Resource)]
struct ConfigHandle(Handle<DynamicScene>);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 1. 产生一个目标实体
    commands.spawn((Player, MobConfig::default()));

    // 2. 加载配置资产
    let handle = asset_server.load("configs/boss.scn.ron");
    commands.insert_resource(ConfigHandle(handle));
}

fn apply_config_system(
    mut commands: Commands,
    config: Res<ConfigHandle>,
    scenes: Res<Assets<DynamicScene>>,
    query: Query<Entity, With<Player>>,
) {
    // 只有当资产加载完成（存在于 Assets 中）时才运行
    if let Some(dynamic_scene) = scenes.get(&config.0) {
        if let Ok(player_entity) = query.get_single() {
            // 关键：将闭包提交给 World 执行
            commands.add(move |world: &mut World| {
                // 创建映射关系：把 RON 里的 0 号实体，“对齐”到世界里的 player_entity
                let mut entity_map = EntityHashMap::default();
                entity_map.insert(Entity::from_raw(0), player_entity);

                // 执行底层写入
                // 注意：这会自动根据 RON 里的数据，对 player_entity 执行 insert 或 apply
                let scene = world
                    .resource::<Assets<DynamicScene>>()
                    .get(&config.0)
                    .unwrap();
                scene.write_to_world(world, &mut entity_map).unwrap();

                info!("配置已应用！");
            });

            // 应用一次后移除资源，防止每帧都跑（按需处理）
            commands.remove_resource::<ConfigHandle>();
        }
    }
}
