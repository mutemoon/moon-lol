use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::action::CommandAction;
use lol_core::attack::{Attack, AttackState};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::level::Level;
use lol_core::base::gold::Gold;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::run::{Run, RunTarget};
use lol_core::skill::{CoolDown, Skill, SkillPoints, Skills};
use lol_core::team::Team;
use lol_render::controller::Controller;
use crate::models::{
    AiActor, ActorPhase, Observe, ObserveMyself, ObserveMinion, ObserveEnemyHero,
    ObserveSkill, AttackTarget,
};
use crate::tools::call_rig_llm;

#[derive(Resource)]
pub struct ActorResource {
    pub actor: AiActor,
    pub timer: Timer,
    pub phase: ActorPhase,
    pub cached_action: Option<crate::models::AgentDecisionResult>,
}

pub fn update_actor(
    mut commands: Commands,
    mut actor_resource: ResMut<ActorResource>,
    real_time: Res<Time<Real>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    q_player: Query<(
        Entity, &Transform,
        Option<&AttackState>, Option<&Run>, &Team,
        Option<&Health>, Option<&Level>, Option<&AbilityResource>,
        Option<&Attack>, Option<&Damage>, Option<&Armor>,
        Option<&Skills>, Option<&SkillPoints>,
        Option<&Gold>,
    ), With<Controller>>,
    q_enemy_hero: Query<(Entity, &Transform, &Health, &Team), (With<AttackTarget>, Without<Death>, Without<Controller>)>,
    q_minions: Query<(Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane), (With<Minion>, Without<Death>)>,
    q_all_transforms: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    actor_resource.timer.tick(real_time.delta());

    match actor_resource.phase {
        ActorPhase::Thinking => {
            // 1. 确保决策期间游戏时间暂停
            if !virtual_time.is_paused() {
                debug!("Actor 开始使用 Rig AI 框架进行决策模型分析... 游戏时间已暂停。");
                virtual_time.pause();
            }

            // 2. 如果还没有发起后台异步 LLM 决策，则发起一个后台线程请求
            if actor_resource.actor.rx.is_none() && actor_resource.cached_action.is_none() {
                let Some(observe) = build_observe(
                    &virtual_time, &q_player, &q_enemy_hero, &q_minions, &q_all_transforms, &q_skill,
                ) else {
                    return;
                };

                let (tx, rx) = std::sync::mpsc::channel();
                actor_resource.actor.rx = Some(std::sync::Mutex::new(rx));

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    let action = rt.block_on(call_rig_llm(observe));
                    let _ = tx.send(action);
                });
            }

            // 3. 检测后台决策是否已完成返回
            check_thinking_finished(&mut actor_resource);

            // 4. 决策一旦返回立刻下发动作，不再等待任何固定思考时间
            if actor_resource.cached_action.is_some() {
                let player_entity = q_player.single().map(|v| v.0).ok();
                dispatch_thinking_action(
                    &mut commands, &mut actor_resource, &mut virtual_time, player_entity,
                );
            }
        }
        ActorPhase::Executing => {
            if virtual_time.is_paused() {
                virtual_time.unpause();
            }
            if actor_resource.timer.just_finished() {
                debug!("动作执行完毕，Actor 重新开始决策思考。");
                actor_resource.phase = ActorPhase::Thinking;
                actor_resource.timer.set_duration(std::time::Duration::from_millis(0));
                actor_resource.timer.reset();
            }
        }
    }
}

fn build_observe(
    virtual_time: &Time<Virtual>,
    q_player: &Query<(
        Entity, &Transform,
        Option<&AttackState>, Option<&Run>, &Team,
        Option<&Health>, Option<&Level>, Option<&AbilityResource>,
        Option<&Attack>, Option<&Damage>, Option<&Armor>,
        Option<&Skills>, Option<&SkillPoints>,
        Option<&Gold>,
    ), With<Controller>>,
    q_enemy_hero: &Query<(Entity, &Transform, &Health, &Team), (With<AttackTarget>, Without<Death>, Without<Controller>)>,
    q_minions: &Query<(Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane), (With<Minion>, Without<Death>)>,
    q_all_transforms: &Query<&Transform>,
    q_skill: &Query<(&Skill, &CoolDown)>,
) -> Option<Observe> {
    let Ok((
        _, player_transform, attack_state, player_run, player_team,
        health, level, ability_resource,
        attack, damage, armor,
        skills, skill_points,
        gold,
    )) = q_player.single() else {
        return None;
    };

    let player_pos = player_transform.translation;
    let minions = collect_nearby_minions(player_pos, player_team, q_minions);
    let enemy_hero = collect_nearest_enemy_hero(player_pos, player_team, q_enemy_hero);
    let run_target = get_run_target(player_run, q_all_transforms);
    let skill_list = collect_skills(skills, q_skill);

    Some(Observe {
        time: virtual_time.elapsed_secs(),
        myself: ObserveMyself {
            position: player_pos.xz(),
            attack_state: attack_state.cloned(),
            run_target,
            health: health.map(|h| h.value).unwrap_or(0.0),
            max_health: health.map(|h| h.max).unwrap_or(0.0),
            level: level.map(|l| l.value).unwrap_or(1),
            ability_resource: ability_resource.map(|ar| (ar.value, ar.max)),
            attack_damage: damage.map(|d| d.0).unwrap_or(0.0),
            attack_range: attack.map(|a| a.range).unwrap_or(0.0),
            attack_speed: attack.map(|a| a.current_attack_speed()).unwrap_or(0.0),
            armor: armor.map(|a| a.0).unwrap_or(0.0),
            skill_points: skill_points.map(|sp| sp.0).unwrap_or(0),
            skills: skill_list,
            gold: gold.map(|g| g.current).unwrap_or(0.0),
        },
        minions,
        enemy_hero,
    })
}

fn collect_skills(
    skills: Option<&Skills>,
    q_skill: &Query<(&Skill, &CoolDown)>,
) -> Vec<ObserveSkill> {
    let Some(skills) = skills else {
        return Vec::new();
    };

    skills.iter().enumerate().filter_map(|(i, skill_entity)| {
        let (skill, cooldown) = q_skill.get(skill_entity).ok()?;
        let cooldown_remaining = cooldown.timer.as_ref().and_then(|timer| {
            if timer.is_finished() {
                None
            } else {
                Some(timer.remaining_secs())
            }
        });
        Some(ObserveSkill {
            index: i,
            level: skill.level,
            cooldown_remaining,
        })
    }).collect()
}

fn collect_nearby_minions(
    player_pos: Vec3,
    player_team: &Team,
    q_minions: &Query<(Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane), (With<Minion>, Without<Death>)>,
) -> Vec<ObserveMinion> {
    let mut minions = Vec::new();
    for (minion_entity, minion_transform, health, vital, minion_team, _) in q_minions.iter() {
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

fn collect_nearest_enemy_hero(
    player_pos: Vec3,
    player_team: &Team,
    q_enemy_hero: &Query<(Entity, &Transform, &Health, &Team), (With<AttackTarget>, Without<Death>, Without<Controller>)>,
) -> Option<ObserveEnemyHero> {
    let mut enemy_hero = None;
    let mut min_hero_distance = f32::MAX;
    for (hero_entity, hero_transform, health, hero_team) in q_enemy_hero.iter() {
        if hero_team == player_team {
            continue;
        }
        let distance = player_pos.distance(hero_transform.translation);
        if distance >= min_hero_distance {
            continue;
        }
        min_hero_distance = distance;
        enemy_hero = Some(ObserveEnemyHero {
            entity: hero_entity,
            position: hero_transform.translation.xz(),
            health: health.value,
            max_health: health.max,
        });
    }
    enemy_hero
}

fn get_run_target(
    player_run: Option<&Run>,
    q_all_transforms: &Query<&Transform>,
) -> Option<Vec2> {
    let run = player_run?;
    match run.target {
        RunTarget::Position(pos) => Some(pos),
        RunTarget::Target(t) => {
            let transform = q_all_transforms.get(t).ok()?;
            Some(transform.translation.xz())
        }
    }
}

fn check_thinking_finished(actor_resource: &mut ActorResource) {
    let mut got_action = None;
    let mut finished = false;

    if let Some(ref rx_mutex) = actor_resource.actor.rx {
        if let Ok(rx) = rx_mutex.lock() {
            if let Ok(action) = rx.try_recv() {
                got_action = Some(action);
                finished = true;
            }
        }
    }

    if finished {
        actor_resource.cached_action = got_action;
        actor_resource.actor.rx = None;
        debug!("Rig LLM 决策已在后台计算完成，等待 5s 思考期满后派发。");
    }
}

fn dispatch_thinking_action(
    commands: &mut Commands,
    actor_resource: &mut ActorResource,
    virtual_time: &mut Time<Virtual>,
    player_entity: Option<Entity>,
) {
    let Some(opt_action) = actor_resource.cached_action.take() else {
        return;
    };

    debug!("Rig LLM 决策已返回，恢复游戏流速并执行动作。");
    virtual_time.unpause();

    let Some(player_entity) = player_entity else {
        warn!("未找到可控的角色实体，直接重新进入思考状态");
        actor_resource.phase = ActorPhase::Thinking;
        actor_resource.timer.set_duration(std::time::Duration::from_millis(0));
        actor_resource.timer.reset();
        return;
    };

    // 触发 Bevy EventAgentDecision 事件，向外部派发 AI 决策过程消息
    let observe_json = serde_json::to_value(&opt_action.observe).unwrap_or(serde_json::Value::Null);
    let action_str = match &opt_action.action {
        Some(act) => format!("{:?}", act),
        None => "Stop/None".to_string(),
    };

    commands.trigger(lol_core::action::EventAgentDecision {
        observe: observe_json,
        thinking: opt_action.thinking.clone(),
        action: action_str,
    });

    if let Some(action) = opt_action.action {
        info!("{:?} 派发决策动作 {:?}", player_entity, action);
        commands.trigger(CommandAction {
            entity: player_entity,
            action,
        });
    } else {
        info!("AI 决策为保持当前状态（不执行任何新动作）。");
    }

    actor_resource.phase = ActorPhase::Executing;
    actor_resource.timer.set_duration(std::time::Duration::from_millis(50));
    actor_resource.timer.reset();
}

pub fn get_observe(world: &mut World) -> Option<Observe> {
    let Ok((player_entity, transform, attack_state, run, player_team, _controller)) = world
        .query::<(
            Entity,
            &Transform,
            Option<&AttackState>,
            Option<&Run>,
            &Team,
            &Controller,
        )>()
        .single(world)
    else {
        return None;
    };

    let player_pos = transform.translation;
    let player_team = player_team.clone();

    let run_target = get_world_run_target(world, run);

    // 1. 生命值
    let (health_val, max_health) = world.get::<Health>(player_entity)
        .map(|h| (h.value, h.max))
        .unwrap_or((0.0, 0.0));

    // 2. 等级
    let level = world.get::<Level>(player_entity)
        .map(|l| l.value as u32)
        .unwrap_or(1);

    // 3. 蓝量/法力值
    let ability_resource = world.get::<AbilityResource>(player_entity)
        .map(|ar| (ar.value, ar.max));

    // 4. 攻击力
    let attack_damage = world.get::<Damage>(player_entity)
        .map(|d| d.0)
        .unwrap_or(0.0);

    // 5. 护甲
    let armor = world.get::<Armor>(player_entity)
        .map(|a| a.0)
        .unwrap_or(0.0);

    // 6. 攻击射程与速度
    let (attack_range, attack_speed) = world.get::<Attack>(player_entity)
        .map(|att| {
            let total_as = att.base_attack_speed * (1.0 + att.bonus_attack_speed + att.buff_bonus_attack_speed);
            let capped_as = total_as.min(att.attack_speed_cap);
            (att.range, capped_as)
        })
        .unwrap_or((0.0, 0.0));

    // 7. 技能点
    let skill_points = world.get::<SkillPoints>(player_entity)
        .map(|sp| sp.0)
        .unwrap_or(0);

    // 8. 金币
    let gold_value = world.get::<Gold>(player_entity)
        .map(|g| g.current)
        .unwrap_or(0.0);

    // 9. 技能列表与冷却
    let mut skills = Vec::new();
    if let Some(skills_comp) = world.get::<Skills>(player_entity) {
        for (idx, skill_entity) in skills_comp.iter().enumerate() {
            if let Some(skill) = world.get::<Skill>(skill_entity) {
                let cooldown_remaining = world.get::<CoolDown>(skill_entity)
                    .and_then(|cd| cd.timer.as_ref().map(|t| t.remaining_secs()));
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
    };

    let minions = get_world_minions(world, player_pos, &player_team);
    let enemy_hero = get_world_enemy_hero(world, player_pos, &player_team);
    let time = world.resource::<Time>().elapsed_secs();

    Some(Observe {
        time,
        myself,
        minions,
        enemy_hero,
    })
}

fn get_world_run_target(world: &World, run: Option<&Run>) -> Option<Vec2> {
    let r = run?;
    match r.target {
        RunTarget::Position(pos) => Some(pos),
        RunTarget::Target(t) => {
            let transform = world.get::<Transform>(t)?;
            Some(transform.translation.xz())
        }
    }
}

fn get_world_minions(world: &mut World, player_pos: Vec3, player_team: &Team) -> Vec<ObserveMinion> {
    let mut minions = Vec::new();
    let mut q_minions = world.query_filtered::<(Entity, &Transform, &Health, Option<&Vital>, &Team, &Lane), (With<Minion>, Without<Death>)>();
    for (minion_entity, minion_transform, health, vital, minion_team, _) in q_minions.iter(world) {
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

fn get_world_enemy_hero(world: &mut World, player_pos: Vec3, player_team: &Team) -> Option<ObserveEnemyHero> {
    let mut enemy_hero = None;
    let mut min_hero_distance = f32::MAX;
    let mut q_enemy_hero = world.query_filtered::<(Entity, &Transform, &Health, &Team), (With<AttackTarget>, Without<Death>, Without<Controller>)>();
    for (hero_entity, hero_transform, health, hero_team) in q_enemy_hero.iter(world) {
        if hero_team == player_team {
            continue;
        }
        let distance = player_pos.distance(hero_transform.translation);
        if distance >= min_hero_distance {
            continue;
        }
        min_hero_distance = distance;
        enemy_hero = Some(ObserveEnemyHero {
            entity: hero_entity,
            position: hero_transform.translation.xz(),
            health: health.value,
            max_health: health.max,
        });
    }
    enemy_hero
}
