use bevy::prelude::*;
use lol_rpc::CommandWsRequest as TypedCommandWsRequest;
use serde_json::{Value, json};

use crate::driver::ScriptAgent;
use crate::params::SetScriptParams;
use crate::systems::obs::PlayerQ;

pub fn on_set_script(
    event: On<TypedCommandWsRequest<SetScriptParams>>,
    mut commands: Commands,
    player_q: PlayerQ,
) {
    let params = &event.params;
    let result = (|| -> Result<Value, String> {
        let ent = Entity::from_bits(params.entity_id);
        if player_q.get(ent).is_err() {
            return Err(format!("未找到指定的英雄实体 ID: {}", params.entity_id));
        }
        commands.entity(ent).insert(ScriptAgent {
            source: params.source.clone(),
        });
        Ok(json!({ "status": "success" }))
    })();
    lol_rpc::respond(&event, result);
}
