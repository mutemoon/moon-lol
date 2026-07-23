use bevy::prelude::*;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base::particle::ConfigVfx;
use lol_champions::fiora::Fiora;
use lol_champions::riven::Riven;
use lol_core::entities::champion::Champion;
use lol_core::team::Team;
use lol_render::camera::CameraState;
use lol_render::particle::VfxHandle;
use lol_rpc::{CommandWsRequest as TypedCommandWsRequest, RpcAppExt};
use lol_server::server::send_event;

mod params;
#[cfg(debug_assertions)]
pub use params::{
    GetStateParams, GetTimeParams, GodModeParams, ResetPositionParams, SetSpeedParams,
    SwitchChampionParams, ToggleCooldownParams, TogglePauseParams,
};

pub struct PluginDebug;

impl Plugin for PluginDebug {
    fn build(&self, app: &mut App) {
        let god_mode = app
            .world()
            .get_resource::<lol_core::skill::GodMode>()
            .map(|r| r.0)
            .unwrap_or(false);
        let no_cooldown = app
            .world()
            .get_resource::<lol_core::skill::NoCooldown>()
            .map(|r| r.0)
            .unwrap_or(false);
        app.insert_resource(GlobalDebugState {
            god_mode,
            cooldown_disabled: no_cooldown,
            ..default()
        });

        #[cfg(debug_assertions)]
        {
            // 注册 debug 面 RPC 命令
            app.register_rpc::<SwitchChampionParams>("switch_champion");
            app.register_rpc::<GodModeParams>("god_mode");
            app.register_rpc::<ToggleCooldownParams>("toggle_cooldown");
            app.register_rpc::<ResetPositionParams>("reset_position");
            app.register_rpc::<TogglePauseParams>("toggle_pause");
            app.register_rpc::<SetSpeedParams>("set_speed");
            app.register_rpc::<GetStateParams>("get_state");
            app.register_rpc::<GetTimeParams>("get_time");

            app.add_observer(on_switch_champion)
                .add_observer(on_god_mode)
                .add_observer(on_toggle_cooldown)
                .add_observer(on_reset_position)
                .add_observer(on_toggle_pause)
                .add_observer(on_set_speed)
                .add_observer(on_get_state)
                .add_observer(on_get_time);
        }

        app.add_systems(PreUpdate, on_d_key_select_entity);
    }
}

/// Global debug state tracked across commands.
#[derive(Resource, Default)]
pub struct GlobalDebugState {
    pub cooldown_disabled: bool,
    pub god_mode: bool,
    pub paused: bool,
}

/// System: Listen for D key press and select entity via raycast for log filtering.
fn on_d_key_select_entity(
    world: &World,
    camera: Single<(&Camera, &GlobalTransform), With<CameraState>>,
    mut ray_cast: MeshRayCast,
    res_input: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    q_target: Query<(Entity, &Transform), With<Team>>,
) {
    // 无按键输入则跳过
    if res_input.get_just_pressed().next().is_none() {
        return;
    }

    debug!("=== on_key_pressed 开始 ===");
    debug!(
        "刚刚按下的键: {:?}",
        res_input.get_just_pressed().collect::<Vec<_>>()
    );

    // Only trigger on D key just pressed
    if !res_input.just_pressed(KeyCode::KeyD) {
        return;
    }

    let Some(viewport_position) = window.cursor_position() else {
        debug!("[Debug] No cursor position");
        return;
    };

    let (camera, camera_transform) = *camera;

    let Ok(ray) = camera.viewport_to_world(camera_transform, viewport_position) else {
        debug!("[Debug] Failed to create ray from viewport position");
        return;
    };

    // Filter to only pickable entities
    let filter = |_entity| {
        // Accept all entities that might have a mesh
        true
    };
    let settings = MeshRayCastSettings::default().with_filter(&filter);

    let hits = ray_cast.cast_ray(ray, &settings);
    debug!("[Debug] D key raycast hits: {:?}", hits.len());

    let Some(hit) = hits.first() else {
        return;
    };

    let position = hit.1.point;
    let mut min_distance = f32::MAX;
    let mut selected_entity = None;

    for (entity, transform) in q_target.iter() {
        let distance = position.distance(transform.translation);
        if distance < min_distance {
            min_distance = distance;
            selected_entity = Some(entity);
        }
    }

    let Some(entity) = selected_entity else {
        return;
    };

    debug!(
        "[Debug] Selected entity: {:?}, distance: {:.2}",
        entity, min_distance
    );

    // Send entity_selected event via WebSocket
    let event =
        lol_server::protocol::WsEvent::entity_selected(entity.index_u32(), "debug_select", "");
    send_event(world, event);
}

// ── Typed Debug Observers ──

fn on_switch_champion(
    event: On<TypedCommandWsRequest<SwitchChampionParams>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    champions: Query<(Entity, &Champion, Option<&Name>)>,
) {
    let name = event.params.name.clone();
    let result = (|| -> Result<serde_json::Value, String> {
        let champion_entity = champions
            .iter()
            .map(|(e, _, _)| e)
            .next()
            .ok_or("no champion found")?;

        let name_lower = name.to_lowercase();
        let config_path = format!("characters/{name_lower}/config.ron");
        let skin_path = format!("characters/{name_lower}/skins/skin0.ron");

        let config_record = asset_server.load(&config_path);
        let config_skin = asset_server.load(&skin_path);
        // 加载 vfx.ron 并持有 handle，防止资产被卸载
        let vfx_path = format!("characters/{name_lower}/vfx.ron");
        let vfx_handle = asset_server.load::<ConfigVfx>(&vfx_path);

        let mut e = commands.entity(champion_entity);
        e.insert(VfxHandle(vfx_handle));
        e.remove::<Riven>();
        e.remove::<Fiora>();

        match name.as_str() {
            "Riven" => {
                e.insert(Riven);
            }
            "Fiora" => {
                e.insert(Fiora);
            }
            _ => return Err("unknown champion".to_string()),
        };

        e.insert(ConfigCharacterRecord {
            character_record: config_record,
        });
        e.insert(ConfigSkin { skin: config_skin });

        Ok(serde_json::json!({"name": name}))
    })();
    lol_rpc::respond(&event, result);
}

fn on_god_mode(
    event: On<TypedCommandWsRequest<GodModeParams>>,
    mut debug_state: ResMut<GlobalDebugState>,
    god_mode: Option<ResMut<lol_core::skill::GodMode>>,
    no_cooldown: Option<ResMut<lol_core::skill::NoCooldown>>,
) {
    let enabled = event.params.enabled;
    if let Some(mut gm) = god_mode {
        gm.0 = enabled;
    }
    if enabled {
        if let Some(mut nc) = no_cooldown {
            nc.0 = true;
        }
        debug_state.cooldown_disabled = true;
    }
    debug_state.god_mode = enabled;
    lol_rpc::respond(&event, Ok(serde_json::json!({"enabled": enabled})));
}

fn on_toggle_cooldown(
    event: On<TypedCommandWsRequest<ToggleCooldownParams>>,
    mut debug_state: ResMut<GlobalDebugState>,
    no_cooldown: Option<ResMut<lol_core::skill::NoCooldown>>,
) {
    let enabled = event.params.enabled;
    if let Some(mut nc) = no_cooldown {
        nc.0 = enabled;
    }
    debug_state.cooldown_disabled = enabled;
    lol_rpc::respond(&event, Ok(serde_json::json!({"enabled": enabled})));
}

fn on_reset_position(
    event: On<TypedCommandWsRequest<ResetPositionParams>>,
    champions: Query<(Entity, &Champion, Option<&Name>)>,
    mut transforms: Query<&mut Transform>,
) {
    let result = (|| -> Result<serde_json::Value, String> {
        if let Some((entity, _, _)) = champions.iter().next() {
            if let Ok(mut t) = transforms.get_mut(entity) {
                t.translation = Vec3::ZERO;
                Ok(serde_json::json!({}))
            } else {
                Err("no transform".to_string())
            }
        } else {
            Err("no champion found".to_string())
        }
    })();
    lol_rpc::respond(&event, result);
}

fn on_toggle_pause(
    event: On<TypedCommandWsRequest<TogglePauseParams>>,
    mut debug_state: ResMut<GlobalDebugState>,
    mut time: ResMut<Time<Virtual>>,
) {
    let paused = !debug_state.paused;
    time.set_relative_speed(if paused { 0.0 } else { 1.0 });
    debug_state.paused = paused;
    lol_rpc::respond(&event, Ok(serde_json::json!({"paused": paused})));
}

fn on_set_speed(event: On<TypedCommandWsRequest<SetSpeedParams>>, mut time: ResMut<Time<Virtual>>) {
    let speed = event.params.speed;
    time.set_relative_speed(speed);
    lol_rpc::respond(&event, Ok(serde_json::json!({"speed": speed})));
}

fn on_get_state(
    event: On<TypedCommandWsRequest<GetStateParams>>,
    champions: Query<(Entity, &Champion, Option<&Name>)>,
    debug_state: Res<GlobalDebugState>,
) {
    let champion_name = champions
        .iter()
        .next()
        .and_then(|(_, _, name)| name.map(|n| n.to_string()))
        .unwrap_or_default();

    lol_rpc::respond(
        &event,
        Ok(serde_json::json!({
            "champion": champion_name,
            "god_mode": debug_state.god_mode,
            "cooldown_disabled": debug_state.cooldown_disabled,
            "paused": debug_state.paused,
        })),
    );
}

fn on_get_time(event: On<TypedCommandWsRequest<GetTimeParams>>, time: Res<Time>) {
    lol_rpc::respond(
        &event,
        Ok(serde_json::json!({ "time": time.elapsed_secs() })),
    );
}
