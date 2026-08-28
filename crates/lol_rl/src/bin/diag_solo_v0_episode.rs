//! 详细诊断 SoloV0 对局中 Fiora、Riven 以及小兵的真实交互轨迹

use bevy::prelude::*;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_env::solo_v0::{SoloV0Action, SoloV0DiscreteAction, SoloV0Env};
use lol_env::traits::RlEnvironment;

fn main() {
    println!("🔍 [SoloV0 Episode Diagnosis] 启动环境真实轨迹追踪...\n");

    let mut env = SoloV0Env::new();
    let obs = env.reset();

    println!(
        "环境重置完成. 智能体数: {}, 初始观测维数: {}",
        obs.len(),
        SoloV0Env::state_dim()
    );

    let mut fiora_cs = 0.0f32;
    let mut riven_cs = 0.0f32;

    for t in 0..160 {
        // 观察当前世界状态
        let mut minion_info = Vec::new();
        {
            let world = env.world_mut();
            let mut q = world.query_filtered::<(Entity, &Transform, &Health, &lol_core::team::Team), With<Minion>>();
            for (e, tf, hp, team) in q.iter(world) {
                if hp.value > 0.0 {
                    minion_info.push((e, tf.translation, hp.value, hp.max, *team));
                }
            }
        }

        let fiora_pos = env
            .world()
            .get::<Transform>(env.fiora())
            .map(|t| t.translation)
            .unwrap_or_default();
        let riven_pos = env
            .world()
            .get::<Transform>(env.riven())
            .map(|t| t.translation)
            .unwrap_or_default();

        // 统计距离 Fiora 最近的小兵
        let mut minion_dists: Vec<(f32, f32, f32, lol_core::team::Team)> = minion_info
            .iter()
            .map(|(_, pos, hp, max, team)| (fiora_pos.distance(*pos), *hp, *max, *team))
            .collect();
        minion_dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        if t % 20 == 0 || t < 10 {
            println!(
                "Step {:3} | Fiora: ({:.0}, {:.0}) | Riven: ({:.0}, {:.0}) | 存活小兵: {} | 最近小兵: 距离={:.0}, 血量={:.0}/{:.0}, 队伍={:?}",
                t,
                fiora_pos.x,
                fiora_pos.z,
                riven_pos.x,
                riven_pos.z,
                minion_info.len(),
                minion_dists.first().map(|m| m.0).unwrap_or(9999.0),
                minion_dists.first().map(|m| m.1).unwrap_or(0.0),
                minion_dists.first().map(|m| m.2).unwrap_or(0.0),
                minion_dists
                    .first()
                    .map(|m| m.3)
                    .unwrap_or(lol_core::team::Team::Order),
            );
        }

        // Fiora 尝试选择最近的敌方小兵进行普攻
        let action_fiora = SoloV0Action {
            offset_x: 0.0,
            offset_z: 0.0,
            target_idx: 1, // 最近小兵
            discrete: SoloV0DiscreteAction::Attack,
        };

        let action_riven = SoloV0Action {
            offset_x: 0.0,
            offset_z: 0.0,
            target_idx: 0,
            discrete: SoloV0DiscreteAction::NoOp,
        };

        let res = env.step(&[action_fiora, action_riven]);

        if let Some(&cs) = res[0].reward_variables.get("self_cs") {
            if cs > 0.0 {
                fiora_cs += cs;
                println!(
                    "🎯 [Step {:3}] Fiora 成功击杀小兵！当前 CS: {:.0}",
                    t, fiora_cs
                );
            }
        }
        if let Some(&cs) = res[1].reward_variables.get("self_cs") {
            if cs > 0.0 {
                riven_cs += cs;
                println!(
                    "🎯 [Step {:3}] Riven 成功击杀小兵！当前 CS: {:.0}",
                    t, riven_cs
                );
            }
        }

        if res[0].terminated || res[0].truncated {
            println!("Episode 结束于 step {}", t);
            break;
        }
    }

    println!(
        "\n对局总结: Fiora CS = {}, Riven CS = {}",
        fiora_cs, riven_cs
    );
}
