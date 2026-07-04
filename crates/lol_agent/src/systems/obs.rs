use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::attack::{Attack, AttackState};
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::gold::Gold;
use lol_core::base::level::Level;
use lol_core::base::stats::ChampionStats;
use lol_core::damage::{Armor, Damage};
use lol_core::entities::champion::Champion;
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::run::{Run, RunTarget};
use lol_core::skill::{CoolDown, Skill, SkillPoints, Skills};
use lol_core::team::Team;

use crate::models::{Observe, ObserveHero, ObserveMinion, ObserveMyself, ObserveSkill};

pub type PlayerQ<'w, 's> = Query<
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

pub fn observe(
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

pub fn get_world_run_target(transforms_q: &Query<&Transform>, run: Option<&Run>) -> Option<Vec2> {
    let r = run?;
    match r.target {
        RunTarget::Position(pos) => Some(pos),
        RunTarget::Target(t) => {
            let transform = transforms_q.get(t).ok()?;
            Some(transform.translation.xz())
        }
    }
}

pub fn get_world_minions(
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

pub fn get_world_heroes(
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

pub fn format_observation(obs: &Observe) -> String {
    let mut out = String::new();

    // 游戏时间
    out.push_str(&format!("[游戏时间] {:.2}s\n", obs.time));

    // 自身状态
    let myself = &obs.myself;
    let mut self_status = format!(
        "[自身状态] 等级: {} | 生命值: {:.1}/{:.1}",
        myself.level, myself.health, myself.max_health
    );

    if let Some((current_ar, max_ar)) = myself.ability_resource {
        self_status.push_str(&format!(" | 法力/能量: {:.1}/{:.1}", current_ar, max_ar));
    }

    self_status.push_str(&format!(
        " | 位置: [{:.1}, {:.1}]",
        myself.position.x, myself.position.y
    ));

    let target_str = if let Some(target) = myself.run_target {
        format!("[{:.1}, {:.1}]", target.x, target.y)
    } else {
        "无".to_string()
    };
    self_status.push_str(&format!(" | 移动目标点: {}", target_str));

    self_status.push_str(&format!(
        " | 金币: {:.0} | 补刀: {} | KDA: {}/{}/{}\n",
        myself.gold, myself.minion_kills, myself.kills, myself.deaths, myself.assists
    ));
    out.push_str(&self_status);

    let attack_state = match &myself.attack_state {
        Some(state) => match &state.status {
            lol_core::attack::AttackStatus::Windup { .. } => "前摇中",
            lol_core::attack::AttackStatus::Cooldown { .. } => "冷却中",
        },
        None => "就绪",
    };

    out.push_str(&format!(
        "[自身属性] 攻击力: {:.1} | 射程: {:.1} | 攻速: {:.2} | 护甲: {:.1} | 攻击状态: {}\n",
        myself.attack_damage, myself.attack_range, myself.attack_speed, myself.armor, attack_state
    ));

    // 技能
    let mut skills_str = "[技能] ".to_string();
    let mut skill_parts = Vec::new();
    for skill in &myself.skills {
        let name = match skill.index {
            0 => "Q",
            1 => "W",
            2 => "E",
            3 => "R",
            _ => "未知",
        };
        let status = if skill.level == 0 {
            "未学习".to_string()
        } else if let Some(secs) = skill.cooldown_remaining {
            if secs > 0.0 {
                format!("冷却中: {:.1}s", secs)
            } else {
                "就绪".to_string()
            }
        } else {
            "就绪".to_string()
        };
        skill_parts.push(format!("{}: 等级 {} ({})", name, skill.level, status));
    }
    skills_str.push_str(&skill_parts.join(" | "));
    skills_str.push_str(&format!(" (未分配技能点: {})\n", myself.skill_points));
    out.push_str(&skills_str);

    // 敌方英雄
    out.push_str("[敌方英雄] ");
    if obs.enemy_heroes.is_empty() {
        out.push_str("无\n");
    } else {
        out.push_str("\n");
        for enemy in &obs.enemy_heroes {
            out.push_str(&format!(
                "  - ID: {} | 生命值: {:.1}/{:.1} | 位置: [{:.1}, {:.1}] | 距离: {:.1}\n",
                enemy.entity.to_bits(),
                enemy.health,
                enemy.max_health,
                enemy.position.x,
                enemy.position.y,
                enemy.distance
            ));
        }
    }

    // 友方英雄
    out.push_str("[友方英雄] ");
    if obs.friendly_heroes.is_empty() {
        out.push_str("无\n");
    } else {
        out.push_str("\n");
        for friendly in &obs.friendly_heroes {
            out.push_str(&format!(
                "  - ID: {} | 生命值: {:.1}/{:.1} | 位置: [{:.1}, {:.1}] | 距离: {:.1}\n",
                friendly.entity.to_bits(),
                friendly.health,
                friendly.max_health,
                friendly.position.x,
                friendly.position.y,
                friendly.distance
            ));
        }
    }

    // 小兵
    out.push_str("[敌方小兵] ");
    if obs.minions.is_empty() {
        out.push_str("无\n");
    } else {
        out.push_str("\n");
        for minion in &obs.minions {
            let vital_str = if let Some(vital) = &minion.vital {
                let dir_zh = match vital.direction {
                    lol_core::base::direction::Direction::Up => "上",
                    lol_core::base::direction::Direction::Down => "下",
                    lol_core::base::direction::Direction::Left => "左",
                    lol_core::base::direction::Direction::Right => "右",
                };
                format!(" | 弱点: {}", dir_zh)
            } else {
                "".to_string()
            };

            out.push_str(&format!(
                "  - ID: {} | 生命值: {:.1} | 位置: [{:.1}, {:.1}] | 距离: {:.1}{}\n",
                minion.entity.to_bits(),
                minion.health,
                minion.position.x,
                minion.position.y,
                minion.distance,
                vital_str
            ));
        }
    }

    out
}
