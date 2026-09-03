#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use candle_core::{DType, Device, Result, Tensor};
    use candle_nn::{VarBuilder, VarMap};
    use lol_rl_protocol::{ActionSpace, PolicyBackbone};

    use crate::algo::buffer::RolloutBuffer;
    use crate::algo::ppo::agent::PPOAgent;
    use crate::algo::ppo::config::PPOConfig;
    use crate::policy::{ActorCritic, HeroEmbedConfig};

    #[test]
    fn save_load_roundtrip() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 64;
        let action_dim = 9;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
        let (probs_before, _) = agent.actor_critic.forward(&state)?;
        let probs_before_vec: Vec<f32> = probs_before.squeeze(0)?.to_vec1()?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        assert!(save_path.exists());
        assert!(save_path.metadata().unwrap().len() > 0);

        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
            &save_path,
        )?;
        let (probs_after, _) = loaded.actor_critic.forward(&state)?;
        let probs_after_vec: Vec<f32> = probs_after.squeeze(0)?.to_vec1()?;

        for (i, (b, a)) in probs_before_vec
            .iter()
            .zip(probs_after_vec.iter())
            .enumerate()
        {
            assert!(
                (b - a).abs() < 1e-4,
                "action {} prob mismatch: before={}, after={}",
                i,
                b,
                a
            );
        }

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_empty_file_fails() {
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_empty");
        std::fs::create_dir_all(&tmp_dir).ok();
        let empty_path = tmp_dir.join("empty.safetensors");
        std::fs::write(&empty_path, []).ok();

        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &empty_path,
        );
        assert!(result.is_err());

        let _ = std::fs::remove_file(&empty_path);
    }

    #[test]
    fn load_custom_hidden_dim_auto_detect() -> Result<()> {
        let state_dim = 17;
        let hidden_dim = 256;
        let action_dim = 5;
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            ActionSpace::Discrete(action_dim),
            config.clone(),
            device.clone(),
        )?;

        let tmp_dir = std::env::temp_dir().join("moon_lol_test_256");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("test_ckpt_256.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;

        // Load with dummy hidden_dim=64, it should auto-detect 256 from safetensors file
        let loaded = PPOAgent::load(
            state_dim,
            64,
            ActionSpace::Discrete(action_dim),
            config,
            device.clone(),
            &save_path,
        )?;
        let state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let (probs, val) = loaded.actor_critic.forward(&state)?;
        assert_eq!(probs.dim(1)?, action_dim);
        assert_eq!(val.dim(1)?, 1);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn load_nonexistent_file_fails() {
        let result = PPOAgent::load(
            17,
            64,
            ActionSpace::Discrete(9),
            PPOConfig::default(),
            Device::Cpu,
            &PathBuf::from("/nonexistent/path/checkpoint.safetensors"),
        );
        assert!(result.is_err());
    }

    #[test]
    fn hybrid_ppo_smoke() -> Result<()> {
        let state_dim = 9;
        let hidden_dim = 32;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..8 {
            let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let (encoded, log_prob, value) = agent.actor_critic.sample_action(&state, None)?;
            assert_eq!(encoded.len(), 3, "hybrid 编码应为 [move_x, move_z, attack]");
            buffer.push_unmasked(obs_vec, encoded, log_prob, 0.1, value, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite(), "policy_loss 应为有限值");
        assert!(stats.value_loss.is_finite(), "value_loss 应为有限值");

        // 保存/加载混合 checkpoint
        let tmp_dir = std::env::temp_dir().join("moon_lol_test_hybrid");
        std::fs::create_dir_all(&tmp_dir).ok();
        let save_path = tmp_dir.join("hybrid_ckpt.safetensors");
        let _ = std::fs::remove_file(&save_path);
        agent.save(&save_path)?;
        let loaded = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            &save_path,
        )?;
        let obs_vec: Vec<f32> = (0..state_dim).map(|i| i as f32 * 0.1).collect();
        let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &loaded.device())?;
        let (encoded_after, _, _) = loaded.actor_critic.sample_action(&state, None)?;
        assert_eq!(encoded_after.len(), 3);

        let _ = std::fs::remove_file(&save_path);
        Ok(())
    }

    #[test]
    fn hybrid_ppo_fiora_v2_smoke() -> Result<()> {
        let state_dim = 58;
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 7,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer = RolloutBuffer::new();
        for step in 0..16 {
            let mut obs_vec: Vec<f32> = (0..state_dim).map(|i| (i as f32 * 0.05).sin()).collect();
            obs_vec[16] = if step % 2 == 0 { 1.5 } else { 3.0 };
            let state = Tensor::from_vec(obs_vec.clone(), (1, state_dim), &device)?;
            let mask = Some(vec![true, true, step % 2 == 0, true, true, true, true]);
            let (encoded, log_prob, value) =
                agent.actor_critic.sample_action(&state, mask.as_deref())?;
            assert_eq!(
                encoded.len(),
                3,
                "FioraV2 hybrid 动作编码应为 [offset_x, offset_z, discrete_idx]"
            );
            let disc_idx = encoded[2] as usize;
            assert!(disc_idx < 7, "离散动作索引应在 [0, 6] 范围内");
            buffer.push(obs_vec, encoded, log_prob, 0.25, value, false, mask);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(
            stats.policy_loss.is_finite(),
            "V2 policy_loss 应为有效有限值"
        );
        assert!(stats.value_loss.is_finite(), "V2 value_loss 应为有效有限值");
        assert!(stats.entropy.is_finite(), "V2 entropy 应为有效有限值");

        // 验证批量采样 sample_batch
        let states = Tensor::zeros((4, state_dim), DType::F32, &device)?;
        let batch_samples = agent.actor_critic.sample_batch(&states, None)?;
        assert_eq!(batch_samples.len(), 4);

        // 验证策略可视化显示
        let dummy_state = Tensor::zeros((1, state_dim), DType::F32, &device)?;
        let labels = ["NoOp", "Move", "Attack", "Q", "E", "R", "Flash"];
        let display = agent
            .actor_critic
            .policy_display_real(&dummy_state, None, &labels)?;
        match display {
            lol_rl_protocol::PolicyDisplay::HybridMulti {
                continuous_means,
                discrete_probs,
            } => {
                assert_eq!(continuous_means.len(), 2);
                assert_eq!(discrete_probs.len(), 7);
                let sum_prob: f32 = discrete_probs.iter().map(|p| p.prob).sum();
                assert!((sum_prob - 1.0).abs() < 1e-3, "离散概率之和应为 1.0");
            }
            other => panic!("预期返回 PolicyDisplay::HybridMulti，实际为 {:?}", other),
        }

        // 验证多 Buffer 掩码 Mini-Batch 更新
        let stats_multi = agent.update_multi_buffer(&[buffer], &[0.0], 8)?;
        assert!(
            stats_multi.policy_loss.is_finite(),
            "update_multi_buffer policy_loss 应为有效有限值"
        );
        assert!(
            stats_multi.value_loss.is_finite(),
            "update_multi_buffer value_loss 应为有效有限值"
        );

        Ok(())
    }

    #[test]
    fn selfplay_single_policy_smoke() -> Result<()> {
        let state_dim = 60; // 包含 role_id 与 40 维修饰符槽位
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
        )?;

        let mut buffer_f = RolloutBuffer::new();
        let mut buffer_r = RolloutBuffer::new();

        // 模拟自博弈推演：双方 Agent 各自维护独立的轨迹 Buffer
        for step in 0..10 {
            // 1. Fiora 视角
            let mut obs_f = vec![0.0f32; state_dim];
            obs_f[0] = 0.0; // role_id = 0.0 (Fiora)
            obs_f[17] = 1.2; // distance / 100
            let state_f = Tensor::from_vec(obs_f.clone(), (1, state_dim), &device)?;
            let mask_f = Some(vec![true, true, true, true, true, true, true, true]);
            let (act_f, log_prob_f, val_f) = agent
                .actor_critic
                .sample_action(&state_f, mask_f.as_deref())?;
            assert_eq!(act_f.len(), 3);
            let reward_f = if step % 2 == 0 { 0.5 } else { -0.5 };
            buffer_f.push(obs_f, act_f, log_prob_f, reward_f, val_f, false, mask_f);

            // 2. Riven 视角
            let mut obs_r = vec![0.0f32; state_dim];
            obs_r[0] = 1.0; // role_id = 1.0 (Riven)
            obs_r[17] = 1.2; // distance / 100
            let state_r = Tensor::from_vec(obs_r.clone(), (1, state_dim), &device)?;
            let mask_r = Some(vec![true, true, true, true, true, true, true, true]);
            let (act_r, log_prob_r, val_r) = agent
                .actor_critic
                .sample_action(&state_r, mask_r.as_deref())?;
            assert_eq!(act_r.len(), 3);
            let reward_r = -reward_f; // 严格零和
            buffer_r.push(obs_r, act_r, log_prob_r, reward_r, val_r, false, mask_r);
        }

        assert_eq!(buffer_f.len(), 10);
        assert_eq!(buffer_r.len(), 10);

        // 执行单模型多角色样本独立 GAE + 联合 Mini-Batch PPO 更新
        let stats = agent.update_multi_buffer(&[buffer_f, buffer_r], &[0.0, 0.0], 8)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.entropy.is_finite());

        Ok(())
    }

    #[test]
    fn test_orthogonal_weight_properties() {
        use crate::policy::orthogonal_weight;
        let out_dim = 16;
        let in_dim = 32;
        let gain = 1.414f32;
        let w = orthogonal_weight(out_dim, in_dim, gain);
        assert_eq!(w.len(), out_dim * in_dim);

        // 验证行向量正交性：W * W^T ≈ gain^2 * I
        for r1 in 0..out_dim {
            for r2 in 0..out_dim {
                let dot: f32 = (0..in_dim)
                    .map(|c| w[r1 * in_dim + c] * w[r2 * in_dim + c])
                    .sum();
                if r1 == r2 {
                    let expected = gain * gain;
                    assert!(
                        (dot - expected).abs() < 1e-3,
                        "对角元素 dot ({dot}) 应接近 gain^2 ({expected})"
                    );
                } else {
                    assert!(dot.abs() < 1e-3, "非对角元素 dot ({dot}) 应接近 0 (正交)");
                }
            }
        }
    }

    #[test]
    fn test_industrial_ppo_clip_vloss_and_grad_norm() -> Result<()> {
        let state_dim = 8;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(4);
        let mut config = PPOConfig::default();
        config.clip_vloss = true;
        config.max_grad_norm = 0.5;
        let device = Device::Cpu;

        let mut agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device)?;
        agent.set_lr(1e-4)?;

        let mut buffer = RolloutBuffer::new();
        for _ in 0..10 {
            let obs = vec![0.5f32; state_dim];
            buffer.push_unmasked(obs, vec![0.0], -0.5, 1.0, 0.2, false);
        }

        let stats = agent.update(&buffer, 0.0)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.total_loss.is_finite());
        assert!(stats.kl.is_finite());
        assert!(stats.clip_frac >= 0.0 && stats.clip_frac <= 1.0);

        Ok(())
    }

    #[test]
    fn test_truncation_vs_termination_gae() -> Result<()> {
        let state_dim = 4;
        let hidden_dim = 16;
        let action_space = ActionSpace::Discrete(2);
        let mut config = PPOConfig::default();
        config.gamma = 0.99;
        config.gae_lambda = 0.95;
        let device = Device::Cpu;

        let agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device)?;

        // 场景 1: 真正终止 (terminated = true, truncated = false)
        let mut buffer_term = RolloutBuffer::new();
        buffer_term.push_full(
            vec![0.0; state_dim],
            vec![0.0],
            -0.1,
            1.0,
            0.5,
            true,  // terminated
            false, // truncated
            None,
            None,
        );
        let (_, adv_term) = agent.compute_gae(&buffer_term, 2.0);
        // delta = reward(1.0) + gamma * next_val(2.0) * 0.0 - val(0.5) = 0.5
        assert!(
            (adv_term[0] - 0.5).abs() < 1e-5,
            "真正终止不应 bootstrap 任何未来价值"
        );

        // 场景 2: 超时截断 (terminated = false, truncated = true, 指定真实残局价值 3.0)
        let mut buffer_trunc = RolloutBuffer::new();
        buffer_trunc.push_full(
            vec![0.0; state_dim],
            vec![0.0],
            -0.1,
            1.0,
            0.5,
            false,     // terminated
            true,      // truncated
            Some(3.0), // 真实残局价值
            None,
        );
        // 传入 last_val = 0.0 (开局重置价值)，但应优先使用 3.0 真实残局价值
        let (_, adv_trunc) = agent.compute_gae(&buffer_trunc, 0.0);
        // delta = reward(1.0) + gamma(0.99) * next_val(3.0) * 1.0 - val(0.5) = 1.0 + 2.97 - 0.5 = 3.47
        assert!(
            (adv_trunc[0] - 3.47).abs() < 1e-4,
            "超时截断必须优先使用真实残局价值进行无偏 bootstrap"
        );

        Ok(())
    }

    #[test]
    fn test_hero_id_embedding_selfplay_and_checkpoint() -> Result<()> {
        let state_dim = 36;
        let hidden_dim = 64;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 8,
        };
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let hero_cfg = HeroEmbedConfig {
            num_heroes: 2,
            embed_dim: 16,
        };

        let mut agent = PPOAgent::with_hero_embed(
            state_dim,
            hidden_dim,
            action_space.clone(),
            config.clone(),
            device.clone(),
            hero_cfg,
        )?;
        assert!(agent.actor_critic.has_hero_embed());

        // 验证两种角色的前向推理与采样
        let mut buffer_f = RolloutBuffer::new();
        let mut buffer_r = RolloutBuffer::new();

        for _ in 0..10 {
            // Fiora 视角 (role_id = 0.0)
            let mut obs_f = vec![0.0f32; state_dim];
            obs_f[0] = 0.0;
            obs_f[17] = 1.0;
            let state_f = Tensor::from_vec(obs_f.clone(), (1, state_dim), &device)?;
            let (act_f, log_prob_f, val_f) = agent.actor_critic.sample_action(&state_f, None)?;
            assert_eq!(act_f.len(), 3);
            buffer_f.push_unmasked(obs_f, act_f, log_prob_f, 1.0, val_f, false);

            // Riven 视角 (role_id = 1.0)
            let mut obs_r = vec![0.0f32; state_dim];
            obs_r[0] = 1.0;
            obs_r[17] = 1.0;
            let state_r = Tensor::from_vec(obs_r.clone(), (1, state_dim), &device)?;
            let (act_r, log_prob_r, val_r) = agent.actor_critic.sample_action(&state_r, None)?;
            assert_eq!(act_r.len(), 3);
            buffer_r.push_unmasked(obs_r, act_r, log_prob_r, -1.0, val_r, false);
        }

        // PPO 联合更新
        let stats = agent.update_multi_buffer(&[buffer_f, buffer_r], &[0.0, 0.0], 8)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());

        // 保存并恢复 checkpoint
        let tmp_dir = std::env::temp_dir();
        let ckpt_path = tmp_dir.join("test_hero_embed_model.safetensors");
        agent.save(&ckpt_path)?;

        let loaded_agent = PPOAgent::load(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device.clone(),
            &ckpt_path,
        )?;
        assert!(loaded_agent.actor_critic.has_hero_embed());

        // 验证加载后的模型与原模型输出一致
        let test_obs = vec![1.0f32; state_dim];
        let test_t = Tensor::from_vec(test_obs, (1, state_dim), &device)?;
        let orig_v = agent.actor_critic.get_values(&test_t)?;
        let loaded_v = loaded_agent.actor_critic.get_values(&test_t)?;
        assert!((orig_v[0] - loaded_v[0]).abs() < 1e-5);

        let _ = std::fs::remove_file(&ckpt_path);
        Ok(())
    }

    #[test]
    fn test_mamba_policy_forward_and_ssm() -> Result<()> {
        let state_dim = 16;
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(5);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let agent = PPOAgent::new(state_dim, hidden_dim, action_space, config, device.clone())?;

        // 1. 2D Tensor (batch, state_dim)
        let obs_2d = Tensor::randn(0.0f32, 1.0f32, (4, state_dim), &device)?;
        let (logits_2d, values_2d) = agent.actor_critic.forward(&obs_2d)?;
        assert_eq!(logits_2d.dims(), &[4, 5]);
        assert_eq!(values_2d.dims(), &[4, 1]);

        // 2. 3D Tensor 序列 (batch, seq_len, state_dim)
        let obs_3d = Tensor::randn(0.0f32, 1.0f32, (2, 8, state_dim), &device)?;
        let (logits_3d, values_3d) = agent.actor_critic.forward(&obs_3d)?;
        assert_eq!(logits_3d.dims(), &[2, 8, 5]);
        assert_eq!(values_3d.dims(), &[2, 8, 1]);

        // 3. 验证 Mamba 参数梯度反向传播
        let (log_probs, values, entropy) = agent.actor_critic.evaluate_actions(
            &obs_2d,
            &Tensor::zeros((4, 1), DType::F32, &device)?,
            None,
        )?;
        let loss = ((&log_probs + &values)? + &entropy)?.sum_all()?;
        let _grads = loss.backward()?;
        assert!(loss.to_scalar::<f32>()?.is_finite());

        Ok(())
    }

    #[test]
    fn test_belief_state_mamba() -> Result<()> {
        let state_dim = 20;
        let hidden_dim = 32;
        let belief_dim = 8;
        let action_space = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 4,
        };
        let device = Device::Cpu;

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);

        let mamba_config = crate::policy::MambaConfig::new(hidden_dim);
        let ac = ActorCritic::with_hero_embed_and_mamba(
            state_dim,
            hidden_dim,
            action_space,
            HeroEmbedConfig::default(),
            mamba_config,
            Some(belief_dim),
            vb,
        )?;

        assert!(ac.belief_head().is_some());
        assert_eq!(ac.belief_head().unwrap().belief_dim, belief_dim);

        let dummy_state = Tensor::zeros((2, state_dim), DType::F32, &device)?;
        let belief_res = ac.forward_belief(&dummy_state)?;
        assert!(belief_res.is_some());
        let (mu, std) = belief_res.unwrap();
        assert_eq!(mu.dims(), &[2, belief_dim]);
        assert_eq!(std.dims(), &[2, belief_dim]);

        // 验证 std 全部为正数
        let std_vec = std.flatten_all()?.to_vec1::<f32>()?;
        for s in std_vec {
            assert!(s > 0.0, "Belief std 必须大于 0");
        }

        // 设备迁移验证
        let ac_cpu = ac.to_device(&Device::Cpu)?;
        assert!(ac_cpu.belief_head().is_some());

        Ok(())
    }

    #[test]
    fn test_mamba_stateful_step() -> Result<()> {
        let hidden_dim = 16;
        let device = Device::Cpu;
        let cfg = crate::policy::MambaConfig::new(hidden_dim);

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let mamba = crate::policy::MambaBlock::new(&cfg, vb)?;

        let mut state = crate::policy::MambaState::new(1, &cfg, &device)?;

        // 模拟多步单帧推演
        for step in 0..5 {
            let x = Tensor::new(&[[step as f32 * 0.1; 16]], &device)?;
            let y = mamba.step(&x, &mut state)?;
            assert_eq!(y.dims(), &[1, 16]);
            assert_eq!(state.pos, step + 1);
        }

        // 状态重置验证
        state.reset(1, &cfg, &device)?;
        assert_eq!(state.pos, 0);

        Ok(())
    }

    #[test]
    fn test_mamba_forward_seq_vs_step_equivalence() -> Result<()> {
        let hidden_dim = 16;
        let device = Device::Cpu;
        let cfg = crate::policy::MambaConfig::new(hidden_dim);

        let varmap = VarMap::new();
        let vb = VarBuilder::from_varmap(&varmap, DType::F32, &device);
        let mamba = crate::policy::MambaBlock::new(&cfg, vb)?;

        let seq_len = 8;
        let mut xs_vec = Vec::with_capacity(seq_len * hidden_dim);
        for t in 0..seq_len {
            for d in 0..hidden_dim {
                xs_vec.push(((t + 1) as f32 * 0.1 + (d as f32) * 0.05).sin());
            }
        }

        // 路径 1: forward_seq (并行因果卷积 + Selective Scan)
        let xs_3d = Tensor::from_vec(xs_vec.clone(), (1, seq_len, hidden_dim), &device)?;
        let ys_seq = mamba.forward_seq(&xs_3d)?;
        let ys_seq_vec: Vec<Vec<f32>> = ys_seq.squeeze(0)?.to_vec2()?;

        // 路径 2: 循环 step (单步递推状态)
        let mut state = crate::policy::MambaState::new(1, &cfg, &device)?;
        let mut ys_step_vec = Vec::with_capacity(seq_len);
        for t in 0..seq_len {
            let x_t = xs_3d.narrow(1, t, 1)?.squeeze(1)?;
            let y_t = mamba.step(&x_t, &mut state)?;
            ys_step_vec.push(y_t.squeeze(0)?.to_vec1::<f32>()?);
        }

        for t in 0..seq_len {
            for d in 0..hidden_dim {
                let seq_val = ys_seq_vec[t][d];
                let step_val = ys_step_vec[t][d];
                let diff = (seq_val - step_val).abs();
                assert!(
                    diff < 1e-4,
                    "t={t}, d={d}: seq={seq_val} vs step={step_val}, diff={diff}"
                );
            }
        }

        Ok(())
    }

    #[test]
    fn test_dual_backbone_mlp_and_mamba_roundtrip() -> Result<()> {
        let state_dim = <lol_env::FioraV2Env as lol_env::RlEnvironment>::state_dim();
        let hidden_dim = 64;
        let action_space = ActionSpace::Discrete(4);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        // 1. 测试 MLP 主干
        let mlp_agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            PolicyBackbone::Mlp,
        )?;
        assert_eq!(
            mlp_agent.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mlp
        );

        let tmp_dir = std::env::temp_dir();
        let mlp_ckpt = tmp_dir.join("test_mlp_ckpt.safetensors");
        mlp_agent.save(&mlp_ckpt)?;

        let loaded_mlp = PPOAgent::load_for_env::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            &mlp_ckpt,
        )?;
        assert_eq!(
            loaded_mlp.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mlp
        );
        let _ = std::fs::remove_file(&mlp_ckpt);

        // 2. 测试 Mamba 主干
        let mamba_agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config.clone(),
            device.clone(),
            PolicyBackbone::Mamba,
        )?;
        assert_eq!(
            mamba_agent.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mamba
        );

        let mamba_ckpt = tmp_dir.join("test_mamba_ckpt.safetensors");
        mamba_agent.save(&mamba_ckpt)?;

        let loaded_mamba = PPOAgent::load_for_env::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            &mamba_ckpt,
        )?;
        assert_eq!(
            loaded_mamba.actor_critic.backbone().backbone_type(),
            PolicyBackbone::Mamba
        );
        let _ = std::fs::remove_file(&mamba_ckpt);

        Ok(())
    }

    #[test]
    fn test_mamba_chunk_sequence_ppo_update() -> Result<()> {
        let state_dim = <lol_env::FioraV2Env as lol_env::RlEnvironment>::state_dim();
        let hidden_dim = 32;
        let action_space = ActionSpace::Discrete(4);
        let config = PPOConfig::default();
        let device = Device::Cpu;

        let mut agent = PPOAgent::create_for_env_with_backbone::<lol_env::FioraV2Env>(
            state_dim,
            hidden_dim,
            action_space,
            config,
            device,
            PolicyBackbone::Mamba,
        )?;

        // 构造两个轨迹 Buffer（分别包含 25 个连续时间步）
        let mut buf1 = RolloutBuffer::new();
        let mut buf2 = RolloutBuffer::new();
        let enc_dim = <lol_env::FioraV2Env as lol_env::RlEnvironment>::action_dim();
        for t in 0..25 {
            let state = vec![t as f32 * 0.1; state_dim];
            let mut act = vec![0.0; enc_dim];
            if let Some(last) = act.last_mut() {
                *last = (t % 7) as f32;
            }
            buf1.push_unmasked(state.clone(), act.clone(), -1.0, 1.0, 0.5, t == 24);
            buf2.push_unmasked(state, act, -1.0, -1.0, -0.5, t == 24);
        }

        let stats = agent.update_multi_buffer(&[buf1, buf2], &[0.5, -0.5], 16)?;
        assert!(stats.policy_loss.is_finite());
        assert!(stats.value_loss.is_finite());
        assert!(stats.total_loss.is_finite());

        Ok(())
    }
}
