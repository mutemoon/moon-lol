use bevy::prelude::*;
use lol_champions::fiora::Fiora;
use lol_champions::fiora::passive::Vital;
use lol_champions::riven::Riven;
use lol_core::action::{self, CommandAction};
use lol_core::attack::{Attack, AttackState};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::gold::Gold;
use lol_core::base::level::Level;
use lol_core::base::stats::ChampionStats;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::champion::{AgentId, Champion};
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::run::{Run, RunTarget};
use lol_core::skill::{CoolDown, Skill, SkillPoints, Skills};
use lol_core::team::Team;
use lol_server::events::CommandWsRequest;
use lol_server::protocol::WsResponse;
use serde_json::{Value, from_value, json, to_value};

use crate::driver::{AgentDriver, DEFAULT_TICK_BUDGET, ScriptAgent, ScriptDriver, ScriptRuntimes};
use crate::models::{Observe, ObserveHero, ObserveMinion, ObserveMyself, ObserveSkill};
use crate::rl::{
    DEFAULT_MAX_STEPS, MoonLoLEnv, RewardShaper, RlEnvs, b64_decode, b64_encode, pack_observe,
    unpack_action,
};

type PlayerQ<'w, 's> = Query<
    'w,
    's,
    (
        (
            Entity,
            &'static Transform,
            Option<&'static AttackState>,
            Option<&'static Run>,
            &'static Team,
            &'static Champion,
        ),
        (
            &'static Health,
            Option<&'static Level>,
            Option<&'static AbilityResource>,
            Option<&'static Damage>,
        ),
        (
            Option<&'static Armor>,
            Option<&'static Attack>,
            Option<&'static SkillPoints>,
            Option<&'static Gold>,
            Option<&'static ChampionStats>,
            Option<&'static Skills>,
        ),
    ),
>;

pub fn get_observe(
    player_entity: Entity,
    player_q: &PlayerQ,
    skills_q: &Query<(&Skill, Option<&CoolDown>)>,
    minions_q: &Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    champion_q: &Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    transforms_q: &Query<&Transform>,
    time: f32,
) -> Option<Observe> {
    let Ok((
        (_player_entity, transform, attack_state, run, player_team, _controller),
        (health, opt_level, opt_ability, opt_damage),
        (opt_armor, opt_attack, opt_skill_points, opt_gold, opt_stats, opt_skills),
    )) = player_q.get(player_entity)
    else {
        return None;
    };

    let player_pos = transform.translation;
    let player_team = player_team.clone();

    let run_target = get_world_run_target(transforms_q, run);

    let (health_val, max_health) = (health.value, health.max);
    let level = opt_level.map(|l| l.value as u32).unwrap_or(1);
    let ability_resource = opt_ability.map(|ar| (ar.value, ar.max));
    let attack_damage = opt_damage.map(|d| d.0).unwrap_or(0.0);
    let armor = opt_armor.map(|a| a.0).unwrap_or(0.0);

    let (attack_range, attack_speed) = opt_attack
        .map(|att| {
            let total_as = att.base_attack_speed
                * (1.0 + att.bonus_attack_speed + att.buff_bonus_attack_speed);
            let capped_as = total_as.min(att.attack_speed_cap);
            (att.range, capped_as)
        })
        .unwrap_or((0.0, 0.0));

    let skill_points = opt_skill_points.map(|sp| sp.0).unwrap_or(0);
    let gold_value = opt_gold.map(|g| g.current).unwrap_or(0.0);

    let (kills, deaths, assists, minion_kills) = opt_stats
        .map(|s| (s.kills, s.deaths, s.assists, s.minion_kills))
        .unwrap_or((0, 0, 0, 0));

    let mut skills = Vec::new();
    if let Some(skills_comp) = opt_skills {
        for (idx, skill_entity) in skills_comp.iter().enumerate() {
            if let Ok((skill, cd_opt)) = skills_q.get(skill_entity) {
                let cooldown_remaining =
                    cd_opt.and_then(|cd| cd.timer.as_ref().map(|t| t.remaining_secs()));
                skills.push(ObserveSkill {
                    index: idx,
                    level: skill.level,
                    cooldown_remaining,
                });
            }
        }
    }

    let myself = ObserveMyself {
        position: player_pos.xz(),
        attack_state: attack_state.cloned(),
        run_target,
        health: health_val,
        max_health,
        level,
        ability_resource,
        attack_damage,
        attack_range,
        attack_speed,
        armor,
        skill_points,
        skills,
        gold: gold_value,
        kills,
        deaths,
        assists,
        minion_kills,
    };

    let minions = get_world_minions(minions_q, player_pos, &player_team);
    let (friendly_heroes, enemy_heroes) =
        get_world_heroes(champion_q, player_entity, player_pos, &player_team);

    Some(Observe {
        time,
        myself,
        minions,
        friendly_heroes,
        enemy_heroes,
    })
}

fn get_world_run_target(transforms_q: &Query<&Transform>, run: Option<&Run>) -> Option<Vec2> {
    let r = run?;
    match r.target {
        RunTarget::Position(pos) => Some(pos),
        RunTarget::Target(t) => {
            let transform = transforms_q.get(t).ok()?;
            Some(transform.translation.xz())
        }
    }
}

fn get_world_minions(
    minions_q: &Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    player_pos: Vec3,
    player_team: &Team,
) -> Vec<ObserveMinion> {
    let mut minions = Vec::new();
    for (minion_entity, minion_transform, health, vital, minion_team, _) in minions_q.iter() {
        if minion_team == player_team {
            continue;
        }
        let distance = player_pos.distance(minion_transform.translation);
        if distance > 2000.0 {
            continue;
        }
        minions.push((
            distance,
            ObserveMinion {
                entity: minion_entity,
                position: minion_transform.translation.xz(),
                health: health.value,
                distance,
                vital: vital.cloned(),
            },
        ));
    }
    minions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    minions.into_iter().map(|(_, m)| m).collect()
}

fn get_world_heroes(
    champion_q: &Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    player_entity: Entity,
    player_pos: Vec3,
    player_team: &Team,
) -> (Vec<ObserveHero>, Vec<ObserveHero>) {
    let mut friendly_heroes = Vec::new();
    let mut enemy_heroes = Vec::new();
    for (hero_entity, hero_transform, health, hero_team) in champion_q.iter() {
        if hero_entity == player_entity {
            continue;
        }
        let distance = player_pos.distance(hero_transform.translation);
        if distance > 2000.0 {
            continue;
        }
        let observe_hero = ObserveHero {
            entity: hero_entity,
            position: hero_transform.translation.xz(),
            health: health.value,
            max_health: health.max,
            distance,
        };
        if hero_team == player_team {
            friendly_heroes.push((distance, observe_hero));
        } else {
            enemy_heroes.push((distance, observe_hero));
        }
    }
    friendly_heroes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    enemy_heroes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    (
        friendly_heroes.into_iter().map(|(_, h)| h).collect(),
        enemy_heroes.into_iter().map(|(_, h)| h).collect(),
    )
}

#[derive(serde::Deserialize)]
struct ActionWithEntity {
    entity_id: Option<u64>,
    action: action::Action,
}

/// 从指令 params 解析目标英雄实体：优先 `entity_id`，否则取首个存活英雄。
fn resolve_target(player_q: &PlayerQ, params: &Value) -> Result<Entity, String> {
    if let Some(eid) = params.get("entity_id").and_then(|v| v.as_u64()) {
        let ent = Entity::from_bits(eid);
        if player_q.get(ent).is_err() {
            return Err(format!("未找到指定的英雄实体 ID: {}", eid));
        }
        Ok(ent)
    } else {
        player_q
            .iter()
            .next()
            .map(|((entity, ..), ..)| entity)
            .ok_or_else(|| "未找到存活的英雄实体".to_string())
    }
}

pub fn on_command_ws_request(
    event: On<CommandWsRequest>,
    mut commands: Commands,
    time_res: Res<Time>,
    player_q: PlayerQ,
    skills_q: Query<(&Skill, Option<&CoolDown>)>,
    minions_q: Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    champion_q: Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    transforms_q: Query<&Transform>,
    _riven_q: Query<Entity, With<Riven>>,
    _fiora_q: Query<Entity, With<Fiora>>,
    agent_id_q: Query<(Entity, &AgentId)>,
    mut rl_envs: ResMut<RlEnvs>,
) {
    let cmd = event.cmd.as_str();
    let id = event.id;
    let params = &event.params;

    let result = match cmd {
        "get_agents" => (|| -> Result<Value, String> {
            let mut list = Vec::new();
            for (entity, agent_id) in agent_id_q.iter() {
                list.push(json!({
                    "entity_id": entity.to_bits(),
                    "agent_id": agent_id.0
                }));
            }
            Ok(json!(list))
        })(),
        "get_observe" => (|| -> Result<Value, String> {
            let target_entity_id = params.get("entity_id").and_then(|v| v.as_u64());

            let target_entity = if let Some(eid) = target_entity_id {
                let ent = Entity::from_bits(eid);
                if player_q.get(ent).is_err() {
                    return Err(format!("未找到指定的英雄实体 ID: {}", eid));
                }
                ent
            } else {
                player_q
                    .iter()
                    .next()
                    .map(|((entity, ..), ..)| entity)
                    .ok_or_else(|| "未找到存活的英雄实体".to_string())?
            };

            let obs = get_observe(
                target_entity,
                &player_q,
                &skills_q,
                &minions_q,
                &champion_q,
                &transforms_q,
                time_res.elapsed_secs(),
            )
            .ok_or_else(|| "无法获取当前游戏局势观测数据".to_string())?;
            to_value(obs).map_err(|e| format!("序列化观测数据失败: {}", e))
        })(),
        "action" => (|| -> Result<Value, String> {
            let (action, target_entity_id) =
                if let Ok(wrapper) = from_value::<ActionWithEntity>(params.clone()) {
                    (wrapper.action, wrapper.entity_id)
                } else {
                    let action = from_value::<action::Action>(params.clone())
                        .map_err(|e| format!("无效的游戏动作指令数据: {}", e))?;
                    (action, None)
                };

            let target_entity = if let Some(eid) = target_entity_id {
                let ent = Entity::from_bits(eid);
                if player_q.get(ent).is_err() {
                    return Err(format!("未找到指定的英雄实体 ID: {}", eid));
                }
                ent
            } else {
                player_q
                    .iter()
                    .next()
                    .map(|((entity, ..), ..)| entity)
                    .ok_or_else(|| "未找到存活的英雄实体".to_string())?
            };

            commands.trigger(CommandAction {
                entity: target_entity,
                action,
            });

            Ok(json!({ "status": "success" }))
        })(),
        "set_script" => (|| -> Result<Value, String> {
            // 运行时为指定实体附加/热重载 Script Agent 脚本。
            let eid = params
                .get("entity_id")
                .and_then(|v| v.as_u64())
                .ok_or_else(|| "缺少 entity_id".to_string())?;
            let source = params
                .get("source")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少 source".to_string())?
                .to_string();
            let ent = Entity::from_bits(eid);
            if player_q.get(ent).is_err() {
                return Err(format!("未找到指定的英雄实体 ID: {}", eid));
            }
            commands.entity(ent).insert(ScriptAgent { source });
            Ok(json!({ "status": "success" }))
        })(),
        "get_observe_packed" => (|| -> Result<Value, String> {
            // 高频推理：msgpack(+base64) 编码的观测流。
            let target_entity = resolve_target(&player_q, params)?;
            let obs = get_observe(
                target_entity,
                &player_q,
                &skills_q,
                &minions_q,
                &champion_q,
                &transforms_q,
                time_res.elapsed_secs(),
            )
            .ok_or_else(|| "无法获取当前游戏局势观测数据".to_string())?;
            let bytes = pack_observe(&obs)?;
            Ok(json!({ "msgpack_b64": b64_encode(&bytes) }))
        })(),
        "action_packed" => (|| -> Result<Value, String> {
            // 高频推理：从 msgpack(+base64) 解码动作并下发。
            let b64 = params
                .get("msgpack_b64")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "缺少 msgpack_b64".to_string())?;
            let action = unpack_action(&b64_decode(b64)?)?;
            let target_entity = resolve_target(&player_q, params)?;
            commands.trigger(CommandAction {
                entity: target_entity,
                action,
            });
            Ok(json!({ "status": "success" }))
        })(),
        "rl_reset" => (|| -> Result<Value, String> {
            // Gym reset：初始化 MoonLoLEnv，返回初始观测（msgpack+base64）。
            let target_entity = resolve_target(&player_q, params)?;
            let obs = get_observe(
                target_entity,
                &player_q,
                &skills_q,
                &minions_q,
                &champion_q,
                &transforms_q,
                time_res.elapsed_secs(),
            )
            .ok_or_else(|| "无法获取当前游戏局势观测数据".to_string())?;
            let shaper = params
                .get("config_json")
                .map(RewardShaper::from_config_json)
                .unwrap_or_default();
            let mut env = MoonLoLEnv::new(shaper, DEFAULT_MAX_STEPS);
            let bytes = pack_observe(&obs)?;
            env.reset(obs);
            rl_envs.0.insert(target_entity, env);
            Ok(json!({ "observation_b64": b64_encode(&bytes) }))
        })(),
        "rl_step" => (|| -> Result<Value, String> {
            // Gym step：以当前观测计算 reward 分项与终止/截断，返回 StepResult + 新观测。
            let target_entity = resolve_target(&player_q, params)?;
            let obs = get_observe(
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
            let bytes = pack_observe(&obs)?;
            let result = env.step(obs);
            let mut v = serde_json::to_value(&result).map_err(|e| e.to_string())?;
            if let Value::Object(ref mut map) = v {
                map.insert("observation_b64".into(), json!(b64_encode(&bytes)));
            }
            Ok(v)
        })(),
        _ => return, // 未知指令不处理
    };

    if let Ok(mut lock) = event.response.lock() {
        *lock = Some(match result {
            Ok(data) => WsResponse::ok_with_data(id, data),
            Err(e) => WsResponse::err(id, e),
        });
    }
}

/// 每 FixedUpdate 驱动所有 Script Agent：构建观测 → 运行脚本 → 下发动作。
///
/// - 首帧为实体创建 [`ScriptDriver`]；`ScriptAgent.source` 变更则热重载（脚本 `state` 保留）。
/// - 脚本执行受时间片熔断保护，死循环不会挂起引擎。
/// - 已不再携带 `ScriptAgent` 的实体（如已销毁）会被清理出运行时表。
pub fn drive_script_agents(
    mut commands: Commands,
    time: Res<Time>,
    mut runtimes: NonSendMut<ScriptRuntimes>,
    script_q: Query<(Entity, Ref<ScriptAgent>)>,
    player_q: PlayerQ,
    skills_q: Query<(&Skill, Option<&CoolDown>)>,
    minions_q: Query<
        (Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane),
        (With<Minion>, Without<Death>),
    >,
    champion_q: Query<(Entity, &Transform, &Health, &Team), (With<Champion>, Without<Death>)>,
    transforms_q: Query<&Transform>,
) {
    for (entity, script) in script_q.iter() {
        if !runtimes.0.contains_key(&entity) {
            match ScriptDriver::new(DEFAULT_TICK_BUDGET) {
                Ok(mut d) => {
                    d.reload(&script.source);
                    runtimes.0.insert(entity, d);
                }
                Err(e) => {
                    warn!("创建 Script 驱动失败 ({entity}): {e}");
                    continue;
                }
            }
        } else if script.is_changed() {
            if let Some(d) = runtimes.0.get_mut(&entity) {
                d.reload(&script.source);
            }
        }

        let Some(driver) = runtimes.0.get_mut(&entity) else {
            continue;
        };

        let Some(obs) = get_observe(
            entity,
            &player_q,
            &skills_q,
            &minions_q,
            &champion_q,
            &transforms_q,
            time.elapsed_secs(),
        ) else {
            continue;
        };

        driver.observe(&obs);
        for action in driver.actions() {
            commands.trigger(CommandAction { entity, action });
        }
        if let Some(err) = driver.last_error() {
            warn!("Script Agent {entity} 执行错误: {err}");
        }
    }

    // 清理已不存在 ScriptAgent 的实体对应的运行时。
    runtimes.0.retain(|e, _| script_q.get(*e).is_ok());
}
