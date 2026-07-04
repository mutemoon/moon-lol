use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::entities::champion::Champion;
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::skill::{CoolDown, Skill};
use lol_core::team::Team;
use lol_rpc::CommandWsRequest as TypedCommandWsRequest;
use serde_json::Value;

use crate::params::RlStepParams;
use crate::rl::RlEnvs;
use crate::systems::obs::{PlayerQ, observe};

pub fn on_rl_step(
    event: On<TypedCommandWsRequest<RlStepParams>>,
    player_q: PlayerQ,
    skills_q: Query<(&Skill, Option<&CoolDown>)>,
    minions_q: Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    champion_q: Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    transforms_q: Query<&Transform>,
    time_res: Res<Time>,
    mut rl_envs: ResMut<RlEnvs>,
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

        let env = rl_envs
            .0
            .get_mut(&target_entity)
            .ok_or_else(|| "请先调用 rl_reset 初始化环境".to_string())?;

        let step_result = env.step(obs.clone());
        let mut v = serde_json::to_value(&step_result).map_err(|e| e.to_string())?;
        if let Value::Object(ref mut map) = v {
            map.insert(
                "observation".into(),
                serde_json::to_value(&obs).map_err(|e| e.to_string())?,
            );
        }

        Ok(v)
    })();
    lol_rpc::respond(&event, result);
}
