//! 目标选择机制独立诊断脚本 (Target Selection Diagnostic Toy Task)
//! 
//! 本脚本隔离了复杂的 MOBA 对线多任务环境，专门构建一个仅考察“从多个小兵中识别并攻击最低血量目标”的微型 MDP。
//! 验证 PPO + StructuredActionHead (UnitSelection Attention) 在当前架构下能否快速学会目标选择。

use candle_core::{Device, Tensor};
use lol_env::solo_v0::SoloV0Env;
use lol_env::traits::RlEnvironment;
use lol_rl_protocol::PolicyBackbone;
use lol_rl::ppo::{PPOAgent, PPOConfig, RolloutBuffer};
use rand::Rng;

fn main() -> anyhow::Result<()> {
    println!("🧪 [Target Selection Diagnostic] 启动目标选择独立验证程序...\n");

    let obs_schema = SoloV0Env::obs_schema().expect("obs schema");
    let action_schema = SoloV0Env::action_schema().expect("action schema");

    let hidden_dim = 128;
    let config = PPOConfig {
        lr: 1e-3,
        gamma: 0.95,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        c1: 0.5,
        c2: 0.02, // 熵正则化鼓励适度探索
        ppo_epochs: 4,
        clip_vloss: true,
        max_grad_norm: 0.5,
    };

    let device = Device::Cpu;
    let mut agent = PPOAgent::from_schemas(
        obs_schema,
        action_schema,
        hidden_dim,
        config,
        device.clone(),
        PolicyBackbone::Mlp,
    )?;

    println!("🧠 [模型初始化] 架构: MLP + UnitSelection (Pointer Attention)");
    agent.print_parameter_summary();
    println!();

    let total_iterations = 40;
    let rollout_steps = 128;
    let mut rng = rand::rng();

    println!("-----------------------------------------------------------------------------------------");
    println!(" Iter | Target Acc (选残血) | Action Acc (选普攻) | Perfect Last-Hit | Avg Return | PPO Loss ");
    println!("-----------------------------------------------------------------------------------------");

    for iter in 1..=total_iterations {
        let mut buffer = RolloutBuffer::new();
        let mut correct_target_count = 0usize;
        let mut correct_action_count = 0usize;
        let mut perfect_hit_count = 0usize;
        let mut total_reward = 0.0f32;

        let state_dim = SoloV0Env::state_dim();
        for _ in 0..rollout_steps {
            // 构造观测数据 (160维向量)
            let mut state_vec = vec![0.0f32; state_dim];

            // 1. 角色与基础特征
            state_vec[0] = 0.0; // Role: Fiora
            state_vec[1] = 0.0; // rel_x
            state_vec[2] = 0.5; // rel_z
            state_vec[3] = 0.5; // distance
            state_vec[4] = 1.0; // attack ready
            state_vec[8] = 1.0; // q ready
            state_vec[18] = 1.0; // self hp pct
            state_vec[19] = 1.0; // target hp pct

            // 2. 构造 20 个槽位的可见单位 (Slot 0 是敌方英雄，Slot 1..6 是存活小兵，Slot 7..19 是空白单位)
            // 基础槽位从 index 60 开始，每个槽位 5 维: [unit_type, rel_x, rel_z, hp_pct, is_enemy]
            let slot_offset_base = 60;
            
            // Slot 0: 敌方英雄
            state_vec[slot_offset_base] = 1.0; // unit_type = 1 (Champion)
            state_vec[slot_offset_base + 3] = 1.0; // hp_pct = 1.0
            state_vec[slot_offset_base + 4] = 1.0; // is_enemy = 1.0

            // 随机挑选一个残血小兵槽位 (1..=6)
            let target_low_hp_slot = rng.random_range(1..=6);

            for slot in 1..=6 {
                let off = slot_offset_base + slot * 5;
                state_vec[off] = 2.0; // unit_type = 2 (Melee Minion)
                state_vec[off + 1] = rng.random_range(-0.5..0.5); // rel_x
                state_vec[off + 2] = rng.random_range(0.1..0.8);  // rel_z
                state_vec[off + 4] = 1.0; // is_enemy = 1.0

                if slot == target_low_hp_slot {
                    // 该槽位为残血小兵 (5% ~ 15% HP)
                    state_vec[off + 3] = rng.random_range(0.05..0.15);
                } else {
                    // 其他槽位为高血量小兵 (80% ~ 100% HP)
                    state_vec[off + 3] = rng.random_range(0.80..1.00);
                }
            }

            // Slots 7..=19 保持默认空白槽位 (unit_type=0.0, hp_pct=0.0, is_enemy=0.0)

            let state_tensor = Tensor::from_vec(state_vec.clone(), (1, state_dim), &device)?;
            let (action_encoded, log_prob, val) = agent.actor_critic.sample_action(&state_tensor, None)?;

            let chosen_target = action_encoded[2] as usize;
            let chosen_action = action_encoded[3] as usize;

            let is_target_correct = chosen_target == target_low_hp_slot;
            let is_action_attack = chosen_action == 2; // Discrete 2 = Attack

            if is_target_correct {
                correct_target_count += 1;
            }
            if is_action_attack {
                correct_action_count += 1;
            }
            if is_target_correct && is_action_attack {
                perfect_hit_count += 1;
            }

            // 奖励规则：只要选对残血目标并执行 Attack，给 +1.0 满分；如果选对残血目标但没选 Attack，给 +0.2 引导
            let reward = if is_target_correct && is_action_attack {
                1.0f32
            } else if is_target_correct {
                0.2f32
            } else {
                0.0f32
            };

            total_reward += reward;

            buffer.push(
                state_vec,
                action_encoded,
                log_prob,
                reward,
                val,
                true, // 每步为一个决策回合 (Bandit / Step MDP)
                None,
            );
        }

        let stats = agent.update(&buffer, 0.0)?;

        let target_acc = (correct_target_count as f32 / rollout_steps as f32) * 100.0;
        let action_acc = (correct_action_count as f32 / rollout_steps as f32) * 100.0;
        let perfect_acc = (perfect_hit_count as f32 / rollout_steps as f32) * 100.0;
        let avg_return = total_reward / rollout_steps as f32;

        println!(
            " {:4} | {:17.1}% | {:17.1}% | {:14.1}% | {:10.3} | {:8.4} ",
            iter, target_acc, action_acc, perfect_acc, avg_return, stats.total_loss
        );
    }

    println!("-----------------------------------------------------------------------------------------\n");
    println!("✅ [Diagnostic Completed] 诊断测试运行结束。");

    Ok(())
}
