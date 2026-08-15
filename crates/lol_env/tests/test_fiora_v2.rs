use lol_env::fiora_v2::{
    FioraV2Action, FioraV2DiscreteAction, FioraV2Env, FioraV2Obs,
};
use lol_env::{EnvConfig, RenderMode, RlEnvironment};
use lol_rl_protocol::ActionSpace;

#[test]
fn test_fiora_v2_env_basic_step_and_obs() {
    let mut env = FioraV2Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let obs = env.reset();
    assert_eq!(FioraV2Obs::dim(), 31);
    assert_eq!(obs.to_vector().len(), 31);
    assert!(obs.fiora_hp > 0.0);
    assert!(obs.riven_hp > 0.0);

    // 1. 测试 NoOp 动作（不报错，推进 10 帧）
    let noop_act = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp);
    let res_noop = env.step(noop_act);
    assert_eq!(res_noop.step, 1);
    assert_eq!(res_noop.obs.to_vector().len(), 31);

    // 2. 测试 Move 动作（复用 offset）
    let move_act = FioraV2Action::new(0.5, -0.5, FioraV2DiscreteAction::Move);
    let res_move = env.step(move_act);
    assert_eq!(res_move.step, 2);

    // 3. 测试 CastQ 动作（复用 offset）
    let q_act = FioraV2Action::new(0.8, 0.0, FioraV2DiscreteAction::CastQ);
    let res_q = env.step(q_act);
    assert_eq!(res_q.step, 3);

    // 4. 测试 CastE 动作
    let e_act = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::CastE);
    let res_e = env.step(e_act);
    assert_eq!(res_e.step, 4);

    // 5. 测试 CastR 动作
    let r_act = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::CastR);
    let res_r = env.step(r_act);
    assert_eq!(res_r.step, 5);

    // 6. 测试 Attack 动作
    let atk_act = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::Attack);
    let res_atk = env.step(atk_act);
    assert_eq!(res_atk.step, 6);
}

#[test]
fn test_fiora_v2_action_encoding_roundtrip() {
    let actions = [
        FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp),
        FioraV2Action::new(0.75, -0.25, FioraV2DiscreteAction::Move),
        FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::Attack),
        FioraV2Action::new(-0.6, 0.4, FioraV2DiscreteAction::CastQ),
        FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::CastE),
        FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::CastR),
    ];

    for act in actions {
        let enc = act.to_encoding();
        assert_eq!(enc.len(), 3);
        let recovered = FioraV2Action::from_encoding(&enc);
        assert_eq!(act.discrete, recovered.discrete);
        assert!((act.offset_x - recovered.offset_x).abs() < 1e-4);
        assert!((act.offset_z - recovered.offset_z).abs() < 1e-4);
    }
}

#[test]
fn test_fiora_v2_action_space_meta() {
    let action_space = FioraV2Env::action_space();
    match action_space {
        ActionSpace::Hybrid {
            continuous_dims,
            discrete_classes,
        } => {
            assert_eq!(continuous_dims, 2);
            assert_eq!(discrete_classes, 6);
            assert_eq!(action_space.actor_head_dim(), 8);
            assert_eq!(action_space.encoding_dim(), 3);
        }
        _ => panic!("FioraV2Env must have Hybrid action space"),
    }
}
