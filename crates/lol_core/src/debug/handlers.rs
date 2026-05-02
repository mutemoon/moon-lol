use bevy::prelude::*;

use super::protocol::{
    CmdKind, GodModeParams, SwitchChampionParams, ToggleCooldownParams, WsResponse,
};
use crate::buffs::damage_reduction::BuffDamageReduction;
use crate::entities::champion::Champion;
use crate::skill::{CoolDown, Skill, SkillCooldownMode};

/// Dispatch a WS command to the appropriate handler.
pub fn dispatch(world: &mut World, id: u64, cmd: CmdKind, params: serde_json::Value) -> WsResponse {
    let result = match cmd {
        CmdKind::SwitchChampion => handle_switch_champion(world, params),
        CmdKind::GodMode => handle_god_mode(world, params),
        CmdKind::ToggleCooldown => handle_toggle_cooldown(world, params),
        CmdKind::ResetPosition => handle_reset_position(world),
        CmdKind::TogglePause => handle_toggle_pause(world),
        CmdKind::GetState => handle_get_state(world),
    };

    match result {
        Ok(data) => WsResponse::ok_with_data(id, data),
        Err(e) => WsResponse::err(id, e),
    }
}

fn handle_switch_champion(
    world: &mut World,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let p: SwitchChampionParams =
        serde_json::from_value(params).map_err(|e| format!("invalid params: {e}"))?;

    let name = p.name.clone();

    // Collect champion entity and its skill children.
    let (champion_entity, skill_entities) = {
        let champion_entity = world
            .query::<(Entity, &Champion)>()
            .iter(world)
            .map(|(e, _)| e)
            .next();

        let mut skill_entities = Vec::new();
        if let Some(champion) = champion_entity {
            for (e, skill_of) in world
                .query::<(Entity, &crate::skill::SkillOf)>()
                .iter(world)
            {
                if skill_of.0 == champion {
                    skill_entities.push(e);
                }
            }
        }
        (champion_entity, skill_entities)
    };

    for se in skill_entities {
        world.commands().entity(se).despawn();
    }
    if let Some(entity) = champion_entity {
        world.commands().entity(entity).despawn();
    }

    // Push to the switch queue for lol_champions to process.
    let mut queue = world.resource_mut::<ChampionSwitchQueue>();
    queue.0.push(name.clone());

    Ok(serde_json::json!({"name": name}))
}

fn handle_god_mode(
    world: &mut World,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let p: GodModeParams =
        serde_json::from_value(params).map_err(|e| format!("invalid params: {e}"))?;

    let entities: Vec<Entity> = world
        .query::<(Entity, &Champion)>()
        .iter(world)
        .map(|(e, _)| e)
        .collect();

    for entity in entities {
        let mut e = world.entity_mut(entity);
        if p.enabled {
            e.insert(BuffDamageReduction {
                percentage: 1.0,
                damage_type: None,
            });
        } else {
            e.remove::<BuffDamageReduction>();
        }
    }

    {
        let mut state = world.resource_mut::<super::GlobalDebugState>();
        state.god_mode = p.enabled;
    }

    Ok(serde_json::json!({"enabled": p.enabled}))
}

fn handle_toggle_cooldown(
    world: &mut World,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let p: ToggleCooldownParams =
        serde_json::from_value(params).map_err(|e| format!("invalid params: {e}"))?;

    let skill_entities: Vec<Entity> = world
        .query::<(Entity, &Skill)>()
        .iter(world)
        .map(|(e, _)| e)
        .collect();

    for entity in skill_entities {
        let mut e = world.entity_mut(entity);
        if let Some(mut skill) = e.get_mut::<Skill>() {
            skill.cooldown_mode = if p.enabled {
                SkillCooldownMode::Manual
            } else {
                SkillCooldownMode::AfterCast
            };
        }
        if p.enabled {
            if let Some(mut cd) = e.get_mut::<CoolDown>() {
                cd.timer = None;
            }
        }
    }

    {
        let mut state = world.resource_mut::<super::GlobalDebugState>();
        state.cooldown_disabled = p.enabled;
    }

    Ok(serde_json::json!({"enabled": p.enabled}))
}

fn handle_reset_position(world: &mut World) -> Result<serde_json::Value, String> {
    let entity = world
        .query::<(Entity, &Champion)>()
        .iter(world)
        .map(|(e, _)| e)
        .next();

    match entity {
        Some(entity) => {
            let mut e = world.entity_mut(entity);
            if let Some(mut t) = e.get_mut::<Transform>() {
                t.translation = Vec3::ZERO;
                Ok(serde_json::json!({}))
            } else {
                Err("no transform".into())
            }
        }
        None => Err("no champion found".into()),
    }
}

fn handle_toggle_pause(world: &mut World) -> Result<serde_json::Value, String> {
    let paused = {
        let state = world.resource::<super::GlobalDebugState>();
        !state.paused
    };

    let mut time = world.resource_mut::<Time<Virtual>>();
    time.set_relative_speed(if paused { 0.0 } else { 1.0 });

    {
        let mut state = world.resource_mut::<super::GlobalDebugState>();
        state.paused = paused;
    }

    Ok(serde_json::json!({"paused": paused}))
}

fn handle_get_state(world: &mut World) -> Result<serde_json::Value, String> {
    let saved = {
        let state = world.resource::<super::GlobalDebugState>();
        (state.god_mode, state.cooldown_disabled, state.paused)
    };

    let champion_name = world
        .query::<(&Name, &Champion)>()
        .iter(world)
        .next()
        .map(|(n, _)| n.to_string())
        .unwrap_or_default();

    Ok(serde_json::json!({
        "champion": champion_name,
        "god_mode": saved.0,
        "cooldown_disabled": saved.1,
        "paused": saved.2,
    }))
}

/// Simple queue resource — lol_champions reads from this each frame.
#[derive(Resource, Default)]
pub struct ChampionSwitchQueue(pub Vec<String>);
