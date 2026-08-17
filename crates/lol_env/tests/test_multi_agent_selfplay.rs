use lol_env::fiora_riven_selfplay::{FioraRivenSelfPlayEnv, SelfPlayAction, SelfPlayDiscreteAction};
use lol_env::parallel::ParallelFioraRivenSelfPlayEnvs;
use lol_env::traits::{EnvConfig, RenderMode, RlEnvironment};

#[test]
fn test_multi_agent_selfplay_signatures_and_throughput() {
    assert_eq!(FioraRivenSelfPlayEnv::num_agents(), 2);
    assert_eq!(FioraRivenSelfPlayEnv::agent_names(), &["Fiora", "Riven"]);

    let mut env = FioraRivenSelfPlayEnv::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let initial_obs = env.reset();
    assert_eq!(initial_obs.len(), 2, "自博弈环境应同时产出双方初始观测");
    assert_eq!(initial_obs[0].role_id, 0.0, "首个智能体应为剑姬 (0.0)");
    assert_eq!(initial_obs[1].role_id, 1.0, "次个智能体应为瑞雯 (1.0)");

    let act_fiora = SelfPlayAction::new(0.5, 0.0, SelfPlayDiscreteAction::Move);
    let act_riven = SelfPlayAction::new(-0.5, 0.0, SelfPlayDiscreteAction::Move);

    let step_res = env.step(&[act_fiora, act_riven]);
    assert_eq!(step_res.len(), 2, "应同时返回双方各自的 StepResult");
    assert_eq!(step_res[0].obs.role_id, 0.0);
    assert_eq!(step_res[1].obs.role_id, 1.0);
}

#[test]
fn test_multi_agent_parallel_envs_batch_throughput() {
    let num_envs = 4;
    let par_envs = ParallelFioraRivenSelfPlayEnvs::with_config(
        num_envs,
        EnvConfig {
            max_steps: 20,
            render_mode: RenderMode::Headless,
        },
    );

    let all_obs = par_envs.reset_all();
    assert_eq!(all_obs.len(), num_envs);
    for env_obs in &all_obs {
        assert_eq!(env_obs.len(), 2, "每个环境实例内部应产生 2 个智能体观测");
    }

    let flat_obs = par_envs.reset_all_flat();
    assert_eq!(
        flat_obs.len(),
        num_envs * 2,
        "展平观测应包含 8 个智能体样本"
    );

    let flat_actions = vec![
        SelfPlayAction::new(0.0, 0.0, SelfPlayDiscreteAction::NoOp);
        num_envs * 2
    ];
    let flat_res = par_envs.step_all_flat(&flat_actions);
    assert_eq!(
        flat_res.len(),
        num_envs * 2,
        "单次批量 step_all_flat 应产出 8 个 StepResult"
    );
}
