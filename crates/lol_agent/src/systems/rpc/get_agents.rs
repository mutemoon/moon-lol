use bevy::prelude::*;
use lol_core::entities::champion::AgentId;
use lol_rpc::CommandWsRequest as TypedCommandWsRequest;
use serde_json::{Value, json};

use crate::params::GetAgentsParams;

pub fn on_get_agents(
    event: On<TypedCommandWsRequest<GetAgentsParams>>,
    agent_id_q: Query<(Entity, &AgentId)>,
) {
    let result = (|| -> Result<Value, String> {
        let mut list = Vec::new();
        for (entity, agent_id) in agent_id_q.iter() {
            list.push(json!({
                "entity_id": entity.to_bits(),
                "agent_id": agent_id.0
            }));
        }
        Ok(json!(list))
    })();
    lol_rpc::respond(&event, result);
}
