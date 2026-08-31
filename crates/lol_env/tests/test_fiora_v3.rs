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

    // 4. 验证 0 级技能设置：纯普攻补刀环境，Q/W/E/R 均未学习（level=0）
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

        assert_eq!(q_skill.level, 0, "补刀训练环境 Q 技能等级应为 0（未学习）");
        assert_eq!(w_skill.level, 0, "补刀训练环境 W 技能等级应为 0（未学习）");
        assert_eq!(e_skill.level, 0, "补刀训练环境 E 技能等级应为 0（未学习）");
        assert_eq!(r_skill.level, 0, "补刀训练环境 R 技能等级应为 0（未学习）");
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
    env.update_curriculum(1.0, 1.0, 0.1, 0.0);
    env.reset();

    // 1. 测试在开启无效攻击惩罚配置下，普通攻击但未产生补刀时的惩罚
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
        "开启惩罚系数时普通攻击未产生补刀应受到惩罚，实际奖励: {}",
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
        self_ad: 68.0,
        attack_state: 0,
        attack_is_windup: false,
        attack_is_cooldown: false,
        attack_timer_remaining: 0.0,
        self_modifiers: Vec::new(),
        visible_units: vec![
            // Slot 0: 敌方远程小兵
            ObsContext::new()
                .with_var("unit_type", 3.0)
                .with_var("rel_pos[0]", 1.0)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 0.5)
                .with_var("hp_norm", 0.25)
                .with_var("is_enemy", 1.0),
            // Slot 1: 敌方近战小兵
            ObsContext::new()
                .with_var("unit_type", 2.0)
                .with_var("rel_pos[0]", 0.5)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 0.8)
                .with_var("hp_norm", 0.4)
                .with_var("is_enemy", 1.0),
            // Slot 2: 友方近战小兵
            ObsContext::new()
                .with_var("unit_type", 2.0)
                .with_var("rel_pos[0]", -0.5)
                .with_var("rel_pos[1]", 0.0)
                .with_var("hp_pct", 0.9)
                .with_var("hp_norm", 0.45)
                .with_var("is_enemy", 0.0),
        ],
        visible_unit_entities: vec![None; 3],
        visible_missiles: Vec::new(),
    };

    assert!(obs.is_target_enemy(0), "Slot 0 敌方远程小兵应判定为敌方");
    assert!(obs.is_target_enemy(1), "Slot 1 敌方近战小兵应判定为敌方");
    assert!(!obs.is_target_enemy(2), "Slot 2 友方小兵应判定为非敌方");

    let action_masks = FioraV3Env::action_masks(&obs).expect("应返回因式分解动作掩码");
    let cond_masks = action_masks
        .conditional_target_masks
        .expect("应包含自回归条件目标掩码矩阵");

    // 敌方小兵：开放攻击
    assert!(cond_masks[0][2], "敌方小兵 0 Attack 有效");
    assert!(cond_masks[1][2], "敌方小兵 1 Attack 有效");
    // 友方小兵：屏蔽攻击
    assert!(!cond_masks[2][2], "友方小兵 Attack 必须被屏蔽");
}

#[test]
fn test_fiora_v3_attack_minion_step() {
    let mut env = FioraV3Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });
    let obs = env.reset();
    assert_eq!(obs.len(), 1);

    let mut curr_obs = obs;
    let mut got_cs = false;
    for _ in 0..25 {
        // 动态在当前观测中选取血量最低的敌方残血小兵槽位
        let target_idx = curr_obs[0]
            .visible_units
            .iter()
            .enumerate()
            .filter(|(_, u)| {
                u.vars.get("is_enemy").copied().unwrap_or(0.0) > 0.5
                    && u.vars.get("unit_type").copied().unwrap_or(0.0) > 0.0
            })
            .min_by(|(_, a), (_, b)| {
                let hp_a = a.vars.get("hp_norm").copied().unwrap_or(1.0);
                let hp_b = b.vars.get("hp_norm").copied().unwrap_or(1.0);
                hp_a.partial_cmp(&hp_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(idx, _)| idx as u8)
            .unwrap_or(0);

        let action = FioraV3Action::with_target(0.0, 0.0, target_idx, FioraV3DiscreteAction::Attack);
        let res = env.step(&[action]);
        let r = &res[0];
        if r.reward_variables.get("self_cs") == Some(&1.0) {
            got_cs = true;
            assert!(r.reward > 0.0, "成功补刀时应获得正向奖励");
            break;
        }
        curr_obs = vec![r.obs.clone()];
    }
    assert!(got_cs, "持续攻击最低血量残血小兵应在 25 步内成功补刀斩杀");
}

#[test]
fn test_fiora_v3_randomized_minion_health() {
    use lol_core::life::Health;

    let mut env = FioraV3Env::with_config(EnvConfig {
        max_steps: 10,
        render_mode: RenderMode::Headless,
    });
    env.reset();

    let mut killable_count = 0;
    let mut high_hp_count = 0;
    let mut total_minions = 0;

    let mut q = env.world_mut().query_filtered::<&Health, With<Minion>>();
    for h in q.iter(env.world()) {
        total_minions += 1;
        if h.value <= 68.0 {
            killable_count += 1;
        } else if h.value >= 120.0 {
            high_hp_count += 1;
        }
    }

    assert!(total_minions >= 6, "重置后小兵总数应至少有 6 个");
    assert!(killable_count >= 1, "应存在可以一击必杀的残血小兵 (<=68 HP)，实际: {}", killable_count);
    assert!(high_hp_count >= 1, "不能全是一击必杀，应存在较高血量小兵 (>=120 HP)，实际: {}", high_hp_count);
}

#[test]
fn test_fiora_v3_obs_health_norm_and_ad_and_missiles() {
    let mut env = FioraV3Env::with_config(EnvConfig {
        max_steps: 10,
        render_mode: RenderMode::Headless,
    });
    let obs = env.reset();
    assert_eq!(obs.len(), 1);
    let f_obs = &obs[0];

    // 1. 验证英雄自身属性
    assert!(f_obs.self_ad > 0.0, "英雄攻击力应大于 0");
    assert!(f_obs.self_hp > 0.0, "英雄当前血量应大于 0");

    // 2. 验证可见单位包含 hp_norm 与 hp_pct
    assert!(!f_obs.visible_units.is_empty(), "可见单位不应为空");
    let u0 = &f_obs.visible_units[0];
    assert!(u0.vars.contains_key("hp_norm"), "单位应包含 hp_norm 特征");
    assert!(u0.vars.contains_key("hp_pct"), "单位应包含 hp_pct 特征");

    // 3. 验证飞弹槽位结构
    assert_eq!(
        f_obs.visible_missiles.len(),
        lol_env::fiora_v3::FIORA_V3_MAX_VISIBLE_MISSILES,
        "飞弹槽位数量应为 4"
    );
    let m0 = &f_obs.visible_missiles[0];
    assert!(m0.vars.contains_key("rel_pos[0]"), "飞弹应包含 rel_pos[0]");
    assert!(m0.vars.contains_key("rel_pos[1]"), "飞弹应包含 rel_pos[1]");
    assert!(m0.vars.contains_key("is_enemy"), "飞弹应包含 is_enemy");
    assert!(m0.vars.contains_key("is_active"), "飞弹应包含 is_active");

    // 4. 验证 to_vector / eval_to_vector 正常运行
    let vec = f_obs.to_vector();
    assert_eq!(vec.len(), FioraV3Obs::dim(), "特征向量维度应与 Schema 严格一致");
}
