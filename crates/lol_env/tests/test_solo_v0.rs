use bevy::prelude::*;
use lol_base::map::MapPaths;
use lol_core::entities::minion::Minion;
use lol_core::life::Health;
use lol_core::skill::{Skill, Skills};
use lol_core::team::Team;
use lol_env::solo_v0::{SoloV0Action, SoloV0DiscreteAction, SoloV0Env};
use lol_env::traits::{EnvConfig, RenderMode, RlEnvironment};

#[test]
fn test_solo_v0_real_map_and_level_one_laning() {
    let mut env = SoloV0Env::with_config(EnvConfig {
        max_steps: 50,
        render_mode: RenderMode::Headless,
    });

    // 1. 验证真实地图配置
    {
        let map_paths = env.world().resource::<MapPaths>();
        assert_eq!(map_paths.name, "solo", "SoloV0 应使用 solo 地图");
    }

    // 2. 初始重置与观测验证
    let obs = env.reset();
    assert_eq!(obs.len(), 2);
    let fiora_obs = &obs[0];
    let riven_obs = &obs[1];

    // 验证初始坐标在上路兵线交汇点附近
    assert!(
        (fiora_obs.self_pos.x - 2350.0).abs() < 50.0
            && (fiora_obs.self_pos.z - 12750.0).abs() < 50.0,
        "剑姬初始坐标应在上路 Order 侧兵线交汇区，实际: {:?}",
        fiora_obs.self_pos
    );
    assert!(
        (riven_obs.self_pos.x - 2450.0).abs() < 50.0
            && (riven_obs.self_pos.z - 12850.0).abs() < 50.0,
        "瑞雯初始坐标应在上路 Chaos 侧兵线交汇区，实际: {:?}",
        riven_obs.self_pos
    );

    // 3. 验证 1 级技能设置：只学 1 级 Q，W/E/R 未学习（level=0）
    {
        let world = env.world();
        for &champ in &[env.fiora(), env.riven()] {
            let skills = world
                .get::<Skills>(champ)
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
    }

    // 4. 验证观测中的技能就绪状态：Q ready 为 true，W/E/R 为 false
    assert!(fiora_obs.q_ready, "剑姬 1 级 Q 技能应处于就绪状态");
    assert!(!fiora_obs.w_ready, "剑姬 0 级 W 技能应不可施放");
    assert!(!fiora_obs.e_ready, "剑姬 0 级 E 技能应不可施放");
    assert!(!fiora_obs.r_ready, "剑姬 0 级 R 技能应不可施放");

    assert!(riven_obs.q_ready, "瑞雯 1 级 Q 技能应处于就绪状态");
    assert!(!riven_obs.w_ready, "瑞雯 0 级 W 技能应不可施放");
    assert!(!riven_obs.e_ready, "瑞雯 0 级 E 技能应不可施放");
    assert!(!riven_obs.r_ready, "瑞雯 0 级 R 技能应不可施放");

    // 5. 验证 30s 预热后世界中小兵查询正常且已生成
    {
        let mut q_minions = env.world_mut().query::<(&Transform, &Minion)>();
        let minion_count = q_minions.iter(env.world()).count();
        assert!(
            minion_count > 0,
            "30s 预热后线上应已有生成的小兵，实际数量: {}",
            minion_count
        );
    }

    // 6. 验证环境步进与多回合 reset 稳定性
    let step_res = env.step(&[
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastQ),
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Move),
    ]);
    assert_eq!(step_res.len(), 2);

    // 再次重置并验证小兵重新生成
    let reset_obs = env.reset();
    assert_eq!(reset_obs.len(), 2);
    assert!(reset_obs[0].q_ready);
    assert!(!reset_obs[0].w_ready);

    {
        let mut q_minions = env
            .world_mut()
            .query::<(Entity, &Transform, &Team, &Health)>();
        let minion_count = q_minions.iter(env.world()).count();
        assert!(
            minion_count > 0,
            "重置后 30s 预热线上应重新生成小兵，实际数量: {}",
            minion_count
        );
    }
}

#[test]
fn test_solo_v0_last_hit_and_minion_shaping_reward() {
    use lol_core::base::stats::ChampionStats;

    let mut env = SoloV0Env::with_config(EnvConfig {
        max_steps: 10,
        render_mode: RenderMode::Headless,
    });
    env.reset();

    let fiora_entity = env.fiora();
    // 注册临时测试系统，在 step 循环内触发一次补刀
    env.app_mut()
        .add_systems(Update, move |mut q: Query<&mut ChampionStats>| {
            if let Ok(mut stats) = q.get_mut(fiora_entity) {
                if stats.minion_kills == 0 {
                    stats.minion_kills += 1;
                }
            }
        });

    let step_res = env.step(&[
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::NoOp),
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::NoOp),
    ]);

    let fiora_res = &step_res[0];
    let riven_res = &step_res[1];

    // 验证补刀收益（补一刀 +1.0 课程奖励）
    assert_eq!(
        fiora_res.reward_variables.get("self_cs"),
        Some(&1.0),
        "剑姬 reward_variables 应当记录 self_cs = 1.0"
    );
    assert_eq!(
        riven_res.reward_variables.get("target_cs"),
        Some(&1.0),
        "锐雯 reward_variables 应当记录 target_cs = 1.0"
    );
    assert!(
        (fiora_res.reward - 1.0).abs() < 1e-4,
        "剑姬完成补刀时单步奖励应为 +1.0，实际: {}",
        fiora_res.reward
    );
    assert!(
        (riven_res.reward - 0.0).abs() < 1e-4,
        "锐雯未补刀且无动作时单步奖励应为 0.0（不被动受对手补刀惩罚），实际: {}",
        riven_res.reward
    );
}

#[test]
fn test_solo_v0_curriculum_hp_scale_and_update() {
    use lol_core::entities::minion::Minion;
    use lol_core::life::Health;

    let mut env = SoloV0Env::with_config(EnvConfig {
        max_steps: 10,
        render_mode: RenderMode::Headless,
    });
    env.reset();

    // 更新课程参数：设置非对称对比残血小兵为 10% 血量
    env.update_curriculum(0.1, 2.0, 0.2, 0.5);

    let mut q_minions = env.world_mut().query_filtered::<&Health, With<Minion>>();
    let minion_hps: Vec<f32> = q_minions.iter(env.world()).map(|h| h.value).collect();
    assert!(!minion_hps.is_empty(), "必须存在存活小兵");

    let low_hp_count = minion_hps.iter().filter(|&&hp| hp <= 60.0).count();
    let full_hp_count = minion_hps.iter().filter(|&&hp| hp > 400.0).count();

    assert!(
        low_hp_count > 0,
        "对比式课程中必须存在 <= 60.0 HP 的残血小兵供模型锁定，实际数量: {}",
        low_hp_count
    );
    assert!(
        full_hp_count > 0,
        "对比式课程中必须存在 > 400.0 HP 的满血小兵作为对比项，实际数量: {}",
        full_hp_count
    );

    // 恢复为 100% 满血时，所有小兵均恢复满血
    env.update_curriculum(1.0, 2.0, 0.2, 0.5);
    let mut q_minions_full = env.world_mut().query_filtered::<&Health, With<Minion>>();
    for h in q_minions_full.iter(env.world()) {
        assert_eq!(h.value, h.max, "scale=1.0 时所有小兵应恢复 100% 满血");
    }
}

#[test]
fn test_solo_v0_action_mask_windup_and_cooldown() {
    use lol_env::solo_v0::SoloV0Obs;

    let mut obs = SoloV0Obs {
        role_id: 0.0,
        self_pos: Vec3::ZERO,
        self_hp: 1000.0,
        self_max_hp: 1000.0,
        target_pos: Vec3::new(100.0, 0.0, 0.0), // 距离 100 <= ATTACK_MASK_DISTANCE (220.0)
        target_hp: 1000.0,
        target_max_hp: 1000.0,
        distance: 100.0,
        attack_state: 0,
        attack_is_windup: false,
        attack_is_cooldown: false,
        attack_timer_remaining: 0.0,
        q_ready: true,
        q_cd_remaining: 0.0,
        w_ready: false,
        w_cd_remaining: 10.0,
        e_ready: false,
        e_cd_remaining: 8.0,
        r_ready: false,
        r_cd_remaining: 60.0,
        flash_ready: true,
        flash_cd_remaining: 0.0,
        self_modifiers: Vec::new(),
        target_modifiers: Vec::new(),
        visible_units: Vec::new(),
        visible_unit_entities: Vec::new(),
    };

    // 1. 正常就绪状态
    let mask = SoloV0Env::action_mask(&obs).expect("应返回掩码");
    assert!(mask[0], "NoOp 应有效");
    assert!(mask[1], "Move 应有效");
    assert!(mask[2], "距离在范围内且无冷却，Attack 应有效");
    assert!(mask[3], "Q ready，CastQ 应有效");
    assert!(!mask[4], "W 未学习/冷却中，CastW 应被屏蔽");
    assert!(mask[7], "Flash ready，闪现应有效");

    // 2. 前摇阶段 (attack_is_windup = true):
    // 前摇可以被取消，所以不能用前摇 mask 掉技能或移动
    obs.attack_is_windup = true;
    obs.attack_state = 1;
    let mask_windup = SoloV0Env::action_mask(&obs).expect("应返回掩码");
    assert!(mask_windup[1], "前摇中 Move 应有效（用于取消前摇）");
    assert!(
        mask_windup[3],
        "前摇中就绪技能 CastQ 应有效（用于取消前摇）"
    );
    assert!(mask_windup[7], "前摇中 Flash 应有效");

    // 3. 后摇阶段 (attack_is_cooldown = true):
    // 攻击生效后的后摇阶段必须 mask 掉普通攻击，但不用 mask 别的动作
    obs.attack_is_windup = false;
    obs.attack_is_cooldown = true;
    obs.attack_state = 2;
    let mask_cooldown = SoloV0Env::action_mask(&obs).expect("应返回掩码");
    assert!(!mask_cooldown[2], "后摇/普攻冷却阶段必须 mask 掉普通攻击");
    assert!(
        mask_cooldown[1],
        "后摇阶段 Move 应有效（用于走位/取消后摇）"
    );
    assert!(
        mask_cooldown[3],
        "后摇阶段 CastQ 应有效（用于施法/取消后摇）"
    );
    assert!(mask_cooldown[7], "后摇阶段 Flash 应有效");
}

#[test]
fn test_solo_v0_conditional_target_masks_friendly_unit() {
    use lol_env::solo_v0::SoloV0Obs;
    use lol_rl_protocol::ObsContext;

    let obs = SoloV0Obs {
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
            // Slot 3: 空槽位
            ObsContext::new(),
        ],
        visible_unit_entities: vec![None; 4],
    };

    assert!(obs.is_target_enemy(0), "Slot 0 英雄应判定为敌方");
    assert!(obs.is_target_enemy(1), "Slot 1 敌方小兵应判定为敌方");
    assert!(!obs.is_target_enemy(2), "Slot 2 友方小兵应判定为非敌方");
    assert!(!obs.is_target_enemy(3), "Slot 3 空槽位应判定为非敌方");

    let action_masks = SoloV0Env::action_masks(&obs).expect("应返回因式分解动作掩码");
    let cond_masks = action_masks
        .conditional_target_masks
        .expect("应包含自回归条件目标掩码矩阵");

    assert_eq!(cond_masks.len(), 8, "应包含 8 种离散动作对应的条件目标掩码");

    // 1. NoOp (0) 与 Move (1)：所有有效槽位（敌方英雄、敌方小兵、友方小兵）均开放，空槽位 3 禁用
    for act_idx in [0, 1] {
        let mask = &cond_masks[act_idx];
        assert!(mask[0], "动作 {act_idx} 下 Slot 0 有效");
        assert!(mask[1], "动作 {act_idx} 下 Slot 1 有效");
        assert!(
            mask[2],
            "动作 {act_idx} 下 Slot 2 (友军) 允许作为通用/移动参考目标"
        );
        assert!(!mask[3], "动作 {act_idx} 下 Slot 3 空槽位必须禁用");
    }

    // 2. Attack (2) 及 技能 (3..=7)：仅允许敌方目标 (Slot 0, Slot 1)，友军 (Slot 2) 与空槽位 (Slot 3) 必须禁用
    for act_idx in 2..=7 {
        let mask = &cond_masks[act_idx];
        assert!(mask[0], "动作 {act_idx} 下敌方英雄 Slot 0 有效");
        assert!(mask[1], "动作 {act_idx} 下敌方小兵 Slot 1 有效");
        assert!(!mask[2], "动作 {act_idx} 下友方小兵 Slot 2 必须被屏蔽");
        assert!(!mask[3], "动作 {act_idx} 下空槽位 Slot 3 必须被屏蔽");
    }
}
