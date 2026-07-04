use bevy::prelude::*;
use lol_core::action::CommandAction;
use lol_rpc::CommandWsRequest as TypedCommandWsRequest;
use serde_json::{Value, json};

use crate::params::ActionParams;
use crate::systems::obs::PlayerQ;

pub fn on_action(
    event: On<TypedCommandWsRequest<ActionParams>>,
    mut commands: Commands,
    player_q: PlayerQ,
) {
    let params = &event.params;
    let result = (|| -> Result<Value, String> {
        let target_entity = lol_rpc::resolve_target(
            params.entity_id,
            |e| player_q.get(e).is_ok(),
            || player_q.iter().next().map(|((entity, ..), ..)| entity),
        )?;

        commands.trigger(CommandAction {
            entity: target_entity,
            action: params.action.clone(),
        });

        Ok(json!({ "status": "success" }))
    })();
    lol_rpc::respond(&event, result);
}
