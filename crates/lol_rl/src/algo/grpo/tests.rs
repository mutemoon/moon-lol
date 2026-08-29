#[cfg(test)]
mod tests {
    use candle_core::Device;
    use lol_rl_protocol::{ActionSpace, PolicyBackbone};

    use crate::algo::buffer::RolloutBuffer;
    use crate::algo::grpo::agent::GRPOAgent;
    use crate::algo::grpo::config::GRPOConfig;
    use crate::policy::HeroEmbedConfig;

    #[test]
    fn test_grpo_advantage_computation() {
        let agent = GRPOAgent::new(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                gamma: 0.9,
                ..Default::default()
            },
            Device::Cpu,
        )
        .unwrap();

        // 构造 2 个 buffer 作为一组
        let mut b1 = RolloutBuffer::new();
        b1.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 1.0, 0.0, false);
        b1.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 2.0, 0.0, true);

        let mut b2 = RolloutBuffer::new();
        b2.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 0.0, 0.0, false);
        b2.push_unmasked(vec![0.0; 4], vec![0.0], 0.0, 0.0, 0.0, true);

        let (advs, _mean, std) = agent.compute_group_advantages(&[b1, b2], 2);
        assert_eq!(advs.len(), 2);
        assert_eq!(advs[0].len(), 2);
        assert_eq!(advs[1].len(), 2);
        assert!(std > 0.0);
        // 回报高的 b1 其 advantage 应该显著大于回报为 0 的 b2
        assert!(advs[0][0] > advs[1][0]);
    }

    #[test]
    fn test_grpo_update_mlp() {
        let mut agent = GRPOAgent::with_hero_embed_and_backbone(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                grpo_epochs: 2,
                ..Default::default()
            },
            Device::Cpu,
            HeroEmbedConfig::default(),
            PolicyBackbone::Mlp,
        )
        .unwrap();

        let mut b1 = RolloutBuffer::new();
        b1.push_unmasked(vec![0.0, 1.0, 0.5, 0.2], vec![1.0], -1.0, 1.0, 0.0, false);
        b1.push_unmasked(vec![0.0, 0.8, 0.3, 0.1], vec![2.0], -1.2, 2.0, 0.0, true);

        let mut b2 = RolloutBuffer::new();
        b2.push_unmasked(vec![0.0, 0.5, 0.1, 0.0], vec![0.0], -0.9, -1.0, 0.0, false);
        b2.push_unmasked(vec![0.0, 0.2, 0.0, 0.0], vec![0.0], -0.8, -2.0, 0.0, true);

        let stats = agent.update_multi_buffer(&[b1, b2], 2).unwrap();
        assert!(stats.policy_loss.is_finite());
        assert!(stats.entropy >= 0.0);
    }

    #[test]
    fn test_grpo_update_mamba() {
        let mut agent = GRPOAgent::with_hero_embed_and_backbone(
            4,
            16,
            ActionSpace::Discrete(3),
            GRPOConfig {
                group_size: 2,
                grpo_epochs: 2,
                ..Default::default()
            },
            Device::Cpu,
            HeroEmbedConfig::default(),
            PolicyBackbone::Mamba,
        )
        .unwrap();

        let mut b1 = RolloutBuffer::new();
        for _ in 0..8 {
            b1.push_unmasked(vec![0.0, 1.0, 0.5, 0.2], vec![1.0], -1.0, 1.0, 0.0, false);
        }

        let mut b2 = RolloutBuffer::new();
        for _ in 0..8 {
            b2.push_unmasked(vec![0.0, 0.5, 0.1, 0.0], vec![0.0], -0.9, -1.0, 0.0, false);
        }

        let stats = agent.update_multi_buffer(&[b1, b2], 8).unwrap();
        assert!(stats.policy_loss.is_finite());
    }

    #[test]
    fn test_grpo_has_zero_critic_parameters() {
        use lol_env::{FioraV2Env, RlEnvironment};
        let agent = GRPOAgent::create_for_env_with_backbone::<FioraV2Env>(
            FioraV2Env::state_dim(),
            64,
            FioraV2Env::action_space(),
            GRPOConfig::default(),
            Device::Cpu,
            PolicyBackbone::Mlp,
        )
        .unwrap();

        let summary = agent.parameter_summary();
        agent.print_parameter_summary();

        // 验证 100% 没有任何 Critic 模块或参数
        let categories = summary.category_totals();
        assert!(
            !categories
                .iter()
                .any(|(cat, _)| cat.contains("Critic") || cat.contains("价值"))
        );
        assert!(!summary.layers.iter().any(|l| l.name.contains("critic")));
    }
}
