use bevy::prelude::*;
use lol_base::map::MapPaths;
use lol_core::base::stats::ChampionStats;
use lol_core::entities::minion::Minion;
use lol_core::skill::{Skill, Skills};
use lol_env::fiora_v3::{FioraV3Action, FioraV3DiscreteAction, FioraV3Env, FioraV3Obs};
use lol_env::traits::{EnvConfig, RenderMode, RlEnvironment};

#[test]
fn test_fiora_v3_real_map_and_single_agent() {
    let mut env = FioraV3Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    // 1. 验证真实 solo 地图配置
    {
        let map_paths = env.world().resource::<MapPaths>();
        assert_eq!(map_paths.name, "solo", "FioraV3 应使用 solo 地图");
    }

    // 2. 验证单智能体配置
    assert_eq!(FioraV3Env::num_agents(), 1, "FioraV3 应为单智能体环境");
    assert_eq!(FioraV3Env::agent_names(), &["Fiora"]);

    // 3. 初始重置与观测验证
    let obs = env.reset();
    assert_eq!(obs.len(), 1, "单智能体环境 reset 应返回 1 个观测");
    let fiora_obs = &obs[0];

    // 验证初始坐标在上路 Order 侧兵线交汇区附近
    assert!(
        (fiora_obs.self_pos.x - 2350.0).abs() < 50.0
            && (fiora_obs.self_pos.z - 12750.0).abs() < 50.0,
        "剑姬初始坐标应在上路 Order 侧兵线交汇区，实际: {:?}",
        fiora_obs.self_pos
    );

    // 4. 验证 1 级技能设置：只学 1 级 Q，W/E/R 未学习（level=0）
    {
        let world = env.world();
        let skills = world
            .get::<Skills>(env.fiora())
            .expect("英雄实体应挂载 Skills 组件");
        let s_entities = skills.to_vec();
        assert!(s_entities.len() >= 4);

        let q_skill = world.get::<Skill>(s_entities[0]).unwrap();
        let w_skill = world.get::<Skill>(s_entities[1]).unwrap();
        let e_skill = world.get::<Skill>(s_entities[2]).unwrap();
        let r_skill = world.get::<Skill>(s_entities[3]).unwrap();

        assert_eq!(q_skill.level, 1, "1级对线 Q 技能等级应为 1");
        assert_eq!(w_skill.level, 0, "1级对线 W 技能等级应为 0（未学习）");
        assert_eq!(e_skill.level, 0, "1级对线 E 技能等级应为 0（未学习）");
        assert_eq!(r_skill.level, 0, "1级对线 R 技能等级应为 0（未学习）");
    }

    // 5. 验证 30s 预热后世界中小兵生成
    {
        let mut q_minions = env.world_mut().query::<(&Transform, &Minion)>();
        let minion_count = q_minions.iter(env.world()).count();
        assert!(
            minion_count > 0,
            "30s 预热后线上应已有生成的小兵，实际数量: {}",
            minion_count
        );
    }
}

#[test]
fn test_fiora_v3_last_hit_reward_and_attack_no_cs_penalty() {
    let mut env = FioraV3Env::with_config(EnvConfig {
        max_steps: 10,
        render_mode: RenderMode::Headless,
    });
    env.reset();

    // 1. 测试普通攻击但未产生补刀时的惩罚
    // 假设选择攻击槽位 1（敌方小兵），但当前 step 未斩杀小兵
    let step_penalty = env.step(&[FioraV3Action::with_target(
        0.0,
        0.0,
        1,
        FioraV3DiscreteAction::Attack,
    )]);
    assert_eq!(step_penalty.len(), 1);
    let res_penalty = &step_penalty[0];

    assert_eq!(
        res_penalty.reward_variables.get("self_attack_no_cs"),
        Some(&1.0),
        "对小兵发起普通攻击但未击杀时，self_attack_no_cs 应为 1.0"
    );
    assert_eq!(
        res_penalty.reward_variables.get("self_cs"),
        Some(&0.0),
        "未击杀小兵时，self_cs 应为 0.0"
    );
    assert!(
        res_penalty.reward < 0.0,
        "普通攻击但未产生补刀时应受到惩罚，实际奖励: {}",
        res_penalty.reward
    );

    // 2. 测试产生击杀时的正向补刀奖励
    let fiora_entity = env.fiora();
    env.app_mut()
        .add_systems(Update, move |mut q: Query<&mut ChampionStats>| {
            if let Ok(mut stats) = q.get_mut(fiora_entity) {
                if stats.minion_kills == 0 {
                    stats.minion_kills += 1;
                }
            }
        });

    let step_reward = env.step(&[FioraV3Action::with_target(
        0.0,
        0.0,
        1,
        FioraV3DiscreteAction::Attack,
    )]);
    let res_reward = &step_reward[0];

    assert_eq!(
        res_reward.reward_variables.get("self_cs"),
        Some(&1.0),
        "补刀击杀时 self_cs 应为 1.0"
    );
    assert_eq!(
        res_reward.reward_variables.get("self_attack_no_cs"),
        Some(&0.0),
        "补刀成功时 self_attack_no_cs 应为 0.0（不计惩罚）"
    );
    assert!(
        res_reward.reward > 0.0,
        "补刀成功时应获得正向奖励，实际奖励: {}",
        res_reward.reward
    );
}

#[test]
fn test_fiora_v3_conditional_target_masks() {
    use lol_rl_protocol::ObsContext;

    let obs = FioraV3Obs {
        role_id: 0.0,
        self_pos: Vec3::ZERO,
        self_hp: 1000.0,
        self_max_hp: 1000.0,
        target_pos: Vec3::new(100.0, 0.0, 0.0),
        target_hp: 1000.0,
        target_max_hp: 1000.0,
        distance: 100.0,
        attack_state: 0,
        attack_is_windup: false,
        attack_is_cooldown: false,
        attack_timer_remaining: 0.0,
        q_ready: true,
        q_cd_remaining: 0.0,
        w_ready: true,
        w_cd_remaining: 0.0,
        e_ready: true,
        e_cd_remaining: 0.0,
        r_ready: true,
        r_cd_remaining: 0.0,
        flash_ready: true,
        flash_cd_remaining: 0.0,
        self_modifiers: Vec::new(),
        target_modifiers: Vec::new(),
        visible_units: vec![
            // Slot 0: 敌方英雄
            ObsContext::new()
                .with_var("unit_type", 1.0)
                .with_var("rel_pos[0]", 1.0)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 1.0)
                .with_var("is_enemy", 1.0),
            // Slot 1: 敌方近战小兵
            ObsContext::new()
                .with_var("unit_type", 2.0)
                .with_var("rel_pos[0]", 0.5)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 0.8)
                .with_var("is_enemy", 1.0),
            // Slot 2: 友方近战小兵
            ObsContext::new()
                .with_var("unit_type", 2.0)
                .with_var("rel_pos[0]", -0.5)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 0.9)
                .with_var("is_enemy", 0.0),
        ],
        visible_unit_entities: vec![None; 3],
    };

    assert!(obs.is_target_enemy(0), "Slot 0 英雄应判定为敌方");
    assert!(obs.is_target_enemy(1), "Slot 1 敌方小兵应判定为敌方");
    assert!(!obs.is_target_enemy(2), "Slot 2 友方小兵应判定为非敌方");

    let action_masks = FioraV3Env::action_masks(&obs).expect("应返回因式分解动作掩码");
    let cond_masks = action_masks
        .conditional_target_masks
        .expect("应包含自回归条件目标掩码矩阵");

    // 敌方小兵：开放攻击
    assert!(cond_masks[1][2], "敌方小兵 Attack 有效");
    // 友方小兵：屏蔽攻击
    assert!(!cond_masks[2][2], "友方小兵 Attack 必须被屏蔽");
}