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
use lol_core::entities::champion::{Champion, AgentId};
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::run::{Run, RunTarget};
use lol_core::skill::{CoolDown, Skill, SkillPoints, Skills};
use lol_core::team::Team;
use lol_render::controller::Controller;
use lol_server::events::CommandWsRequest;
use lol_server::protocol::WsResponse;
use serde_json::{Value, from_value, json, to_value};

use crate::models::{Observe, ObserveHero, ObserveMinion, ObserveMyself, ObserveSkill};

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
    riven_q: Query<Entity, With<Riven>>,
    fiora_q: Query<Entity, With<Fiora>>,
    agent_id_q: Query<(Entity, &AgentId)>,
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
        _ => return, // 未知指令不处理
    };

    if let Ok(mut lock) = event.response.lock() {
        *lock = Some(match result {
            Ok(data) => WsResponse::ok_with_data(id, data),
            Err(e) => WsResponse::err(id, e),
        });
    }
}
