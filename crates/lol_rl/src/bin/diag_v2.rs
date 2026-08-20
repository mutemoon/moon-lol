//! 快速诊断：v2 环境随机策略一集，打印 reward 分布，理解 reward 基线。
use lol_env::RlEnvironment;
use lol_env::fiora_v2::{FioraV2Action, FioraV2DiscreteAction, FioraV2Env};
use rand::Rng;

fn main() {
    let mut env = FioraV2Env::new();
    let mut obs = env.reset();
    let mut total = 0.0f32;
    let mut damage_sum = 0.0f32;
    let mut step_penalty_sum = 0.0f32;
    let mut rng = rand::rng();
    for t in 0..200 {
        let action = FioraV2Action::new(
            rng.random_range(-1.0f32..1.0),
            rng.random_range(-1.0f32..1.0),
            FioraV2DiscreteAction::from_u8(rng.random_range(0..7)),
        );
        let res = <FioraV2Env as RlEnvironment>::step(&mut env, &[action]);
        let r = &res[0];
        total += r.reward;
        for item in &r.reward_breakdown {
            match item.name.as_str() {
                s if s.contains("伤害收益") => damage_sum += item.value,
                s if s.contains("时间步惩罚") => step_penalty_sum += item.value,
                _ => {}
            }
        }
        if r.terminated || r.truncated {
            println!(
                "step={t} 终止 terminated={} truncated={} | 累计R={total:.4} damage={damage_sum:.4} penalty={step_penalty_sum:.4}",
                r.terminated, r.truncated
            );
            break;
        }
        if t % 40 == 39 {
            println!(
                "step={t} 累计R={total:.4} damage={damage_sum:.4} penalty={step_penalty_sum:.4}"
            );
        }
        obs = res[0].obs.clone();
    }
    println!("总 reward = {total:.4} (damage={damage_sum:.4}, penalty={step_penalty_sum:.4})");
    let _ = obs;
}
