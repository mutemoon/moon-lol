use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::entities::champion::Champion;
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::skill::{CoolDown, Skill};
use lol_core::team::Team;
use lol_rpc::CommandWsRequest as TypedCommandWsRequest;
use serde_json::{Value, to_value};

use crate::params::ObserveParams;
use crate::systems::obs::{PlayerQ, format_observation, observe};

pub fn on_observe(
    event: On<TypedCommandWsRequest<ObserveParams>>,
    player_q: PlayerQ,
    skills_q: Query<(&Skill, Option<&CoolDown>)>,
    minions_q: Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    champion_q: Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    transforms_q: Query<&Transform>,
    time_res: Res<Time>,
) {
    let params = &event.params;
    let result = (|| -> Result<Value, String> {
        let target_entity = lol_rpc::resolve_target(
            params.entity_id,
            |e| player_q.get(e).is_ok(),
            || player_q.iter().next().map(|((entity, ..), ..)| entity),
        )?;

        let obs = observe(
            target_entity,
            &player_q,
            &skills_q,
            &minions_q,
            &champion_q,
            &transforms_q,
            time_res.elapsed_secs(),
        )
        .ok_or_else(|| "无法获取当前游戏局势观测数据".to_string())?;

        if params.json {
            to_value(obs).map_err(|e| format!("序列化观测数据失败: {}", e))
        } else {
            Ok(Value::String(format_observation(&obs)))
        }
    })();
    lol_rpc::respond(&event, result);
}
