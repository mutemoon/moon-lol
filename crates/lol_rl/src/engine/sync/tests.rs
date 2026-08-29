#[cfg(test)]
mod tests {
    use candle_core::{Device, Result};
    use lol_env::solo_v0::SoloV0Env;
    use lol_rl_protocol::PolicyBackbone;

    use crate::algo::grpo::{GRPOAgent, GRPOConfig};
    use crate::algo::ppo::{PPOAgent, PPOConfig};
    use crate::engine::sync::session::TrainingSession;

    #[test]
    fn test_solo_v0_mlp_training_2_envs() -> Result<()> {
        let device = Device::Cpu;
        let ppo_config = PPOConfig {
            lr: 3e-4,
            gamma: 0.99,
            gae_lambda: 0.95,
            clip_eps: 0.2,
            c1: 0.5,
            ppo_epochs: 2,
            clip_vloss: true,
            max_grad_norm: 0.5,
        };

        let state_dim = <SoloV0Env as lol_env::RlEnvironment>::state_dim();
        let hidden_dim = 64;
        let action_space = <SoloV0Env as lol_env::RlEnvironment>::action_space();

        let agent = PPOAgent::create_for_env_with_backbone::<SoloV0Env>(
            state_dim,
            hidden_dim,
            action_space,
            ppo_config,
            device.clone(),
            PolicyBackbone::Mlp,
        )?;

        // 2 个并行环境，horizon = 30 步快速测试
        let mut session = TrainingSession::<SoloV0Env>::new(agent, 2, state_dim, 30, device);

        let outcome = session.step_once(1, 3e-4, 32)?;
        assert!(outcome.num_samples > 0);
        // 验证 reward_breakdown 不为空并包含课程学习奖励项
        assert!(
            !outcome.reward_breakdown.is_empty(),
            "reward_breakdown 应该包含统计项"
        );
        assert!(
            outcome.reward_breakdown.contains_key("补刀奖励")
                || outcome.reward_breakdown.contains_key("补刀成功奖励")
                || outcome.reward_breakdown.contains_key("攻击小兵未补刀惩罚")
                || outcome.reward_breakdown.contains_key("攻击未补刀惩罚")
                || outcome.reward_breakdown.contains_key("消耗对手奖励")
                || outcome.reward_breakdown.contains_key("消耗对手")
        );

        // 验证课程学习参数下发与第二轮迭代
        session.update_curriculum(0.5, 1.0, 0.1, 0.3);
        let outcome2 = session.step_once(2, 3e-4, 32)?;
        assert!(outcome2.num_samples > 0);

        Ok(())
    }

    #[test]
    fn test_solo_v0_grpo_training_2_envs() -> Result<()> {
        let device = Device::Cpu;
        let grpo_config = GRPOConfig {
            lr: 3e-4,
            gamma: 0.99,
            clip_eps: 0.2,
            grpo_epochs: 2,
            group_size: 2,
            max_grad_norm: 0.5,
        };

        let state_dim = <SoloV0Env as lol_env::RlEnvironment>::state_dim();
        let hidden_dim = 64;
        let action_space = <SoloV0Env as lol_env::RlEnvironment>::action_space();

        let agent = GRPOAgent::create_for_env_with_backbone::<SoloV0Env>(
            state_dim,
            hidden_dim,
            action_space,
            grpo_config,
            device.clone(),
            PolicyBackbone::Mlp,
        )?;

        // 2 个并行环境，horizon = 30 步快速测试
        let mut session = TrainingSession::<SoloV0Env>::new(agent, 2, state_dim, 30, device);

        let outcome = session.step_once(1, 3e-4, 32)?;
        assert!(outcome.num_samples > 0);
        assert_eq!(outcome.stats.value_loss, 0.0, "GRPO value loss 应恒为 0");
        assert!(outcome.stats.policy_loss.is_finite());
        assert!(outcome.sps > 0.0);

        // 第二轮迭代
        let outcome2 = session.step_once(2, 3e-4, 32)?;
        assert!(outcome2.num_samples > 0);
        assert_eq!(outcome2.stats.value_loss, 0.0);

        Ok(())
    }
}
