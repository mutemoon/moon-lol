use bevy::prelude::*;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_champions::fiora::Fiora;
use lol_champions::riven::Riven;
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, Skill, SkillCooldownMode};

use lol_server::events::CommandWsRequest;
use lol_server::protocol::{GodModeParams, SwitchChampionParams, ToggleCooldownParams, WsResponse};

pub struct PluginDebug;

impl Plugin for PluginDebug {
    fn build(&self, app: &mut App) {
        app.insert_resource(GlobalDebugState::default());
        app.add_observer(on_command_ws_request);
    }
}

/// Global debug state tracked across commands.
#[derive(Resource, Default)]
pub struct GlobalDebugState {
    pub cooldown_disabled: bool,
    pub god_mode: bool,
    pub paused: bool,
}

fn on_command_ws_request(
    event: On<CommandWsRequest>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut debug_state: ResMut<GlobalDebugState>,
    mut time: ResMut<Time<Virtual>>,
    champions: Query<(Entity, &Champion, Option<&Name>)>,
    mut skills: Query<(Entity, &mut Skill, Option<&mut CoolDown>)>,
    mut transforms: Query<&mut Transform>,
) {
    let cmd = event.cmd.as_str();
    let id = event.id;
    let params = &event.params;

    let result: Result<serde_json::Value, String> = match cmd {
        "switch_champion" => (|| -> Result<serde_json::Value, String> {
            let p: SwitchChampionParams = serde_json::from_value(params.clone())
                .map_err(|e| format!("invalid params: {e}"))?;

            let name = p.name;
            let champion_entity = champions.iter().map(|(e, _, _)| e).next().ok_or("no champion found")?;

            let name_lower = name.to_lowercase();
            let config_path = format!("characters/{name_lower}/config.ron");
            let skin_path = format!("characters/{name_lower}/skins/skin0.ron");

            let config_record = asset_server.load(&config_path);
            let config_skin = asset_server.load(&skin_path);

            let mut e = commands.entity(champion_entity);
            e.remove::<Riven>();
            e.remove::<Fiora>();

            match name.as_str() {
                "Riven" => { e.insert(Riven); },
                "Fiora" => { e.insert(Fiora); },
                _ => return Err("unknown champion".to_string()),
            };

            e.insert(ConfigCharacterRecord { character_record: config_record });
            e.insert(ConfigSkin { skin: config_skin });

            Ok(serde_json::json!({"name": name}))
        })(),
        "god_mode" => (|| -> Result<serde_json::Value, String> {
            let p: GodModeParams = serde_json::from_value(params.clone())
                .map_err(|e| format!("invalid params: {e}"))?;

            for (entity, _, _) in champions.iter() {
                let mut e = commands.entity(entity);
                if p.enabled {
                    e.insert(BuffDamageReduction { percentage: 1.0, damage_type: None });
                } else {
                    e.remove::<BuffDamageReduction>();
                }
            }

            debug_state.god_mode = p.enabled;
            Ok(serde_json::json!({"enabled": p.enabled}))
        })(),
        "toggle_cooldown" => (|| -> Result<serde_json::Value, String> {
            let p: ToggleCooldownParams = serde_json::from_value(params.clone())
                .map_err(|e| format!("invalid params: {e}"))?;

            for (_, mut skill, cd_opt) in skills.iter_mut() {
                skill.cooldown_mode = if p.enabled { SkillCooldownMode::Manual } else { SkillCooldownMode::AfterCast };
                if p.enabled {
                    if let Some(mut cd) = cd_opt {
                        cd.timer = None;
                    }
                }
            }

            debug_state.cooldown_disabled = p.enabled;
            Ok(serde_json::json!({"enabled": p.enabled}))
        })(),
        "reset_position" => (|| -> Result<serde_json::Value, String> {
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
        })(),
        "toggle_pause" => (|| -> Result<serde_json::Value, String> {
            let paused = !debug_state.paused;
            time.set_relative_speed(if paused { 0.0 } else { 1.0 });
            debug_state.paused = paused;
            Ok(serde_json::json!({"paused": paused}))
        })(),
        "get_state" => (|| -> Result<serde_json::Value, String> {
            let champion_name = champions.iter().next()
                .and_then(|(_, _, name)| name.map(|n| n.to_string()))
                .unwrap_or_default();

            Ok(serde_json::json!({
                "champion": champion_name,
                "god_mode": debug_state.god_mode,
                "cooldown_disabled": debug_state.cooldown_disabled,
                "paused": debug_state.paused,
            }))
        })(),
        _ => return,
    };

    if let Ok(mut lock) = event.response.lock() {
        *lock = Some(match result {
            Ok(data) => WsResponse::ok_with_data(id, data),
            Err(e) => WsResponse::err(id, e),
        });
    }
}
