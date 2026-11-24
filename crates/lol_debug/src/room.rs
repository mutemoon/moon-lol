//! 供本地 `lol` CLI 驱动游戏的两个钩子：暂停计划 + 全量 ECS 快照；截图见 `lol_debug::screenshot`。
//!
//! 房间由 CLI 托管，两者不共享进程：配置档经 `--profile` 交给游戏，其余命令走既有 WS 控制面。

use bevy::ecs::reflect::ReflectComponent;
use bevy::prelude::*;
use lol_client::launch::RoomProfile;
use lol_rpc::{CommandWsRequest, RpcAppExt};
use serde::Deserialize;

/// 当前房间的暂停计划（由 `--profile` 载入）。
#[derive(Resource, Default)]
struct PauseSchedule(Vec<f64>);

/// 房间配置档插件。
pub struct PluginRoomProfile {
    profile: RoomProfile,
}

impl PluginRoomProfile {
    pub fn load(path: &std::path::Path) -> Self {
        Self {
            profile: RoomProfile::load(path),
        }
    }
}

impl Plugin for PluginRoomProfile {
    fn build(&self, app: &mut App) {
        app.insert_resource(PauseSchedule(self.profile.pause_at.clone()));
        if !self.profile.pause_at.is_empty() {
            app.add_systems(Update, auto_pause);
        }

        app.register_rpc::<SnapshotParams>("snapshot");
        app.add_observer(on_snapshot);
    }
}

/// 命中最早的未执行暂停时刻后把虚拟时间归零。
///
/// 时刻保留在计划里：`GameTime` 停住后不会再次越过同一条，后续时刻在 `unpause` 后继续生效。
fn auto_pause(
    schedule: Res<PauseSchedule>,
    game_time: Option<Res<lol_core::game::GameTime>>,
    mut time: ResMut<Time<Virtual>>,
) {
    if time.relative_speed() == 0.0 {
        return;
    }
    let Some(game_time) = game_time else {
        return;
    };
    let now = game_time.elapsed_secs() as f64;
    let Some(&next) = schedule
        .0
        .iter()
        .filter(|&&t| t > now)
        .min_by(|a, b| a.total_cmp(b))
    else {
        return;
    };
    time.set_relative_speed(0.0);
    info!("暂停计划命中 {next:.2}s（当前 {now:.2}s），已暂停游戏时间");
}

#[derive(Deserialize, Default)]
pub struct SnapshotParams {
    /// 只保留名字含该子串的组件（大小写不敏感）。
    #[serde(default)]
    pub component: Option<String>,
    /// 只保留组件值含该子串的实体（大小写不敏感）。
    #[serde(default)]
    pub value_contains: Option<String>,
    /// 每个组件值最多保留字符数。
    #[serde(default)]
    pub max_value_chars: Option<usize>,
}

/// 导出所有实体的组件值（反射 Debug 文本）。
///
/// 默认只给非 `bevy_` 前缀的业务组件——bevy 基础设施组件对调 UI 无意义，且量级巨大。
fn on_snapshot(event: On<CommandWsRequest<SnapshotParams>>, world: &World) {
    let params = &event.params;
    let component_filter = params.component.as_deref().map(str::to_ascii_lowercase);
    let value_filter = params
        .value_contains
        .as_deref()
        .map(str::to_ascii_lowercase);
    let max_chars = params.max_value_chars.unwrap_or(240);

    let Some(registry) = world.get_resource::<AppTypeRegistry>() else {
        lol_rpc::respond(
            &event,
            Err("世界中没有 AppTypeRegistry（反射未注册）".into()),
        );
        return;
    };
    let registry = registry.read();
    let components = world.components();

    let mut entities = Vec::new();
    for entity in world.iter_entities().map(|e| e.id()) {
        let mut comps = Vec::new();
        for &component_id in world.entity(entity).archetype().components() {
            let Some(info) = components.get_info(component_id) else {
                continue;
            };
            let name = info.name().to_string();
            let keep = match &component_filter {
                Some(filter) => name.to_ascii_lowercase().contains(filter),
                None => !name.starts_with("bevy_"),
            };
            if !keep {
                continue;
            }

            let text = info
                .type_id()
                .and_then(|type_id| registry.get(type_id))
                .and_then(|registration| registration.data::<ReflectComponent>())
                .and_then(|reflect_component| reflect_component.reflect(world.entity(entity)))
                .map(|value| format!("{value:?}"))
                .unwrap_or_else(|| "<未注册反射>".to_string());
            let text = truncate(text, max_chars);

            if let Some(filter) = &value_filter
                && !text.to_ascii_lowercase().contains(filter)
            {
                continue;
            }
            comps.push(serde_json::json!({ "component": name, "value": text }));
        }
        if comps.is_empty() {
            continue;
        }
        comps.sort_by(|a, b| a["component"].as_str().cmp(&b["component"].as_str()));
        entities.push(serde_json::json!({
            "entity": entity.to_bits(),
            "components": comps,
        }));
    }
    entities.sort_by_key(|e| e["entity"].as_u64().unwrap_or(0));

    lol_rpc::respond(
        &event,
        Ok(serde_json::json!({
            "entity_count": entities.len(),
            "entities": entities,
        })),
    );
}

fn truncate(text: String, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text;
    }
    text.chars().take(max_chars).collect::<String>() + "…"
}
