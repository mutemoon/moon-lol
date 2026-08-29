#[cfg(test)]
mod tests {
    use candle_core::Device;
    use lol_env::solo_v0::SoloV0Env;
    use lol_rl_protocol::PolicyBackbone;

    use crate::algo::ppo::{PPOAgent, PPOConfig};
    use crate::engine::r#async::session::AsyncTrainingSession;
    use crate::engine::traits::TrainingEngine;

    #[test]
    fn test_async_session_step_once() -> candle_core::Result<()> {
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

        let mut session =
            AsyncTrainingSession::<SoloV0Env>::new(agent, 2, state_dim, 20, 32, 4, 100, device);

        let outcome = session.step_once(1, 3e-4, 32).expect("step_once 成功");
        assert!(outcome.num_samples > 0);

        session.update_curriculum(0.5, 1.0, 0.1, 0.3);
        let outcome2 = session.step_once(2, 3e-4, 32).expect("step_once 2 成功");
        assert!(outcome2.num_samples > 0);

        session.stop();
        Ok(())
    }
}
