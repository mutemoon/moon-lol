use lol_core::skill::{Skill, Skills};
use lol_env::fiora_v2::{FioraV2Action, FioraV2DiscreteAction, FioraV2Env, FioraV2Obs};
use lol_env::{EnvConfig, RenderMode, RlEnvironment};
use lol_rl_protocol::ActionSpace;

#[test]
fn test_fiora_v2_env_basic_step_and_obs() {
    let mut env = FioraV2Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let obs = env.reset();
    assert_eq!(FioraV2Obs::dim(), 33);
    assert_eq!(obs.to_vector().len(), 33);
    assert!(obs.fiora_hp > 0.0);
    assert_eq!(obs.riven_hp, 10000.0, "Riven 初始血量应为 10000.0");
    assert_eq!(obs.riven_max_hp, 10000.0, "Riven 最大血量应为 10000.0");

    // 1. 测试 NoOp 动作（不报错，推进 10 帧）
    let noop_act = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp);
    let res_noop = env.step(noop_act);
    assert_eq!(res_noop.step, 1);
    assert_eq!(res_noop.obs.to_vector().len(), 33);

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

    // 7. 测试 CastFlash 动作（沿 offset 方向瞬移）
    let flash_act = FioraV2Action::new(1.0, 0.0, FioraV2DiscreteAction::CastFlash);
    let res_flash = env.step(flash_act);
    assert_eq!(res_flash.step, 7);
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
        FioraV2Action::new(0.5, 0.5, FioraV2DiscreteAction::CastFlash),
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
            assert_eq!(discrete_classes, 7);
            assert_eq!(action_space.actor_head_dim(), 9);
            assert_eq!(action_space.encoding_dim(), 3);
        }
        _ => panic!("FioraV2Env must have Hybrid action space"),
    }
}

/// 回归测试：重置（包括第二次重置）后技能等级仍为 Q=3/W=1/E=1/R=1，
/// 且 CastQ 可正常施放（复现「重置对局后技能点没点、放不了技能」的 bug）。
#[test]
fn test_fiora_v2_reset_keeps_skill_levels() {
    let mut env = FioraV2Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let _ = env.reset();
    let _ = env.reset(); // 第二次重置（复现 bug 的路径）

    {
        let world = env.app.world();
        let skills = world
            .get::<Skills>(env.fiora)
            .expect("重置后剑姬应已挂载 Skills");
        let skill_entities = skills.to_vec();
        assert!(
            skill_entities.len() >= 4,
            "重置后剑姬应至少有 4 个技能实体，实际 {}",
            skill_entities.len()
        );
        let expected = [3usize, 1, 1, 1];
        for (idx, level) in expected.into_iter().enumerate() {
            let skill = world
                .get::<Skill>(skill_entities[idx])
                .expect("技能实体应存在");
            assert_eq!(skill.level, level, "重置后技能 {idx} 等级应为 {level}");
        }
    }

    // CastQ 应可正常施放（技能等级 > 0）
    let res = env.step(FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::CastQ));
    assert_eq!(res.step, 1);
}

/// 闪现：瞬移约 300 单位、进入冷却、冷却中不再瞬移。
#[test]
fn test_fiora_v2_flash_teleports_300_and_cools_down() {
    let mut env = FioraV2Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let obs0 = env.reset();
    assert!(obs0.flash_ready, "初始闪现应就绪");
    assert_eq!(obs0.flash_cd_remaining, 0.0);

    // 向 +X 方向闪现
    let res = env.step(FioraV2Action::new(
        1.0,
        0.0,
        FioraV2DiscreteAction::CastFlash,
    ));
    let delta = res.obs.fiora_pos - obs0.fiora_pos;
    let dist = delta.length();
    assert!(
        (dist - 300.0).abs() < 1.0,
        "闪现距离应约为 300 单位，实际 {dist}"
    );
    assert!(delta.x > 0.0, "向 +X 闪现后 x 应增大");
    assert!(!res.obs.flash_ready, "闪现后应进入冷却");
    assert!(res.obs.flash_cd_remaining > 0.0, "闪现后应有剩余冷却");

    // 冷却中再次闪现 → 位置不变
    let pos_before = res.obs.fiora_pos;
    let res2 = env.step(FioraV2Action::new(
        -1.0,
        0.0,
        FioraV2DiscreteAction::CastFlash,
    ));
    let moved = (res2.obs.fiora_pos - pos_before).length();
    assert!(moved < 1.0, "冷却中不应瞬移，位移应为 0，实际 {moved}");
}

/// 闪现掩码：就绪时可施放，冷却中动作掩码屏蔽闪现。
#[test]
fn test_fiora_v2_skills_action_mask() {
    let mut env = FioraV2Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    let obs0 = env.reset();
    let mask0 = FioraV2Env::action_mask(&obs0).expect("应有动作掩码");
    assert_eq!(mask0.len(), 7);
    assert!(mask0[3], "初始 Q 技能就绪时掩码应为 true");
    assert!(mask0[4], "初始 E 技能就绪时掩码应为 true");
    assert!(mask0[5], "初始 R 技能就绪时掩码应为 true");
    assert!(mask0[6], "初始闪现就绪时掩码应为 true");

    // 施放 Q 突刺
    let res_q = env.step(FioraV2Action::new(0.5, 0.0, FioraV2DiscreteAction::CastQ));
    if !res_q.obs.q_ready {
        let mask_q = FioraV2Env::action_mask(&res_q.obs).unwrap();
        assert!(!mask_q[3], "Q 技能冷却中应被掩码");
        assert!(
            FioraV2Env::is_action_masked(&res_q.obs, 7),
            "Q 技能冷却中 preset 7 应被掩码"
        );
    }

    // 施放闪现进入冷却
    let res = env.step(FioraV2Action::new(
        1.0,
        0.0,
        FioraV2DiscreteAction::CastFlash,
    ));
    let mask1 = FioraV2Env::action_mask(&res.obs).expect("应有动作掩码");
    assert!(!mask1[6], "闪现冷却中应被掩码");
    assert!(
        FioraV2Env::is_action_masked(&res.obs, 10),
        "闪现冷却中 preset 10 应被掩码"
    );
}

#[test]
fn test_fiora_v2_reward_model_step_penalty_and_damage_norm() {
    use lol_env::fiora_v2::{FioraV2RewardContext, FioraV2RewardModel};
    use lol_env::reward::RewardModel;

    let model = FioraV2RewardModel;

    // 纯单步轻量时间惩罚：固定 -0.001
    let ctx_step = FioraV2RewardContext {
        prev_aligned: false,
        curr_aligned: false,
        is_vital_break: false,
        prev_riven_hp: 500.0,
        curr_riven_hp: 500.0,
        riven_max_hp: 500.0,
        elapsed_secs: 5.0,
    };
    let (reward_step, _, _) = model.evaluate(&ctx_step);
    assert!((reward_step - (-0.001)).abs() < 1e-5);

    // 击中伤害与击杀（归一化伤害: 100/500 = 0.2, 0.2 * 2.5 = 0.5; kill: 2.0; step_penalty: -0.001; total = 2.499）
    let ctx_kill = FioraV2RewardContext {
        prev_aligned: true,
        curr_aligned: true,
        is_vital_break: true,
        prev_riven_hp: 100.0,
        curr_riven_hp: 0.0,
        riven_max_hp: 500.0,
        elapsed_secs: 2.0,
    };
    let (reward_kill, breakdown, _) = model.evaluate(&ctx_kill);
    assert!((reward_kill - 2.499).abs() < 1e-4);
    assert_eq!(breakdown.len(), 3);
}

#[test]
fn test_fiora_v2_default_max_steps_and_truncation() {
    let env = FioraV2Env::new();
    assert_eq!(env.max_steps(), FioraV2Env::DEFAULT_MAX_STEPS);
    assert_eq!(
        FioraV2Env::default_max_steps(),
        FioraV2Env::DEFAULT_MAX_STEPS
    );

    // 测试指定 max_steps 的截断行为
    let mut short_env = FioraV2Env::new_with_max_steps(3);
    short_env.reset();
    let noop = FioraV2Action::new(0.0, 0.0, FioraV2DiscreteAction::NoOp);
    let r1 = short_env.step(noop);
    assert!(!r1.truncated);
    let r2 = short_env.step(noop);
    assert!(!r2.truncated);
    let r3 = short_env.step(noop);
    assert!(r3.truncated, "达到 max_steps 后应触发 truncated: true");
}
