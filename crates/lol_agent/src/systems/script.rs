use bevy::prelude::*;
use lol_champions::fiora::passive::Vital;
use lol_core::action::CommandAction;
use lol_core::entities::champion::Champion;
use lol_core::entities::minion::Minion;
use lol_core::lane::Lane;
use lol_core::life::{Death, Health};
use lol_core::skill::{CoolDown, Skill};
use lol_core::team::Team;

use super::obs::{PlayerQ, observe};
use crate::driver::{AgentDriver, DEFAULT_TICK_BUDGET, ScriptAgent, ScriptDriver, ScriptRuntimes};

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

        let Some(obs) = observe(
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
