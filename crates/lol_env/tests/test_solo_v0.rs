use bevy::prelude::*;
use lol_base::map::MapPaths;
use lol_core::entities::minion::Minion;
use lol_core::skill::{Skill, Skills};
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
        assert_eq!(
            map_paths.name, "solo",
            "SoloV0 应使用 solo 地图"
        );
    }

    // 2. 初始重置与观测验证
    let obs = env.reset();
    assert_eq!(obs.len(), 2);
    let fiora_obs = &obs[0];
    let riven_obs = &obs[1];

    // 验证初始坐标在上路兵线交汇点附近
    assert!(
        (fiora_obs.self_pos.x - 2200.0).abs() < 50.0
            && (fiora_obs.self_pos.z - 12650.0).abs() < 50.0,
        "剑姬初始坐标应在上路 Order 侧兵线交汇区，实际: {:?}",
        fiora_obs.self_pos
    );
    assert!(
        (riven_obs.self_pos.x - 2500.0).abs() < 50.0
            && (riven_obs.self_pos.z - 12910.0).abs() < 50.0,
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

    // 5. 验证 40s 预热后世界中已存在生成的小兵实体
    {
        let mut q_minions = env.world_mut().query::<(&Transform, &Minion)>();
        let minion_count = q_minions.iter(env.world()).count();
        assert!(
            minion_count > 0,
            "40s 预热后地图上应已生成小兵实体，实际数量: {minion_count}"
        );
    }

    // 6. 验证环境步进与多回合 reset 稳定性
    let step_res = env.step(&[
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::CastQ),
        SoloV0Action::new(0.0, 0.0, SoloV0DiscreteAction::Move),
    ]);
    assert_eq!(step_res.len(), 2);

    // 再次重置
    let reset_obs = env.reset();
    assert_eq!(reset_obs.len(), 2);
    assert!(reset_obs[0].q_ready);
    assert!(!reset_obs[0].w_ready);
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

    // 验证补刀收益（补一刀 +5.0）
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
        fiora_res.reward >= 5.0,
        "剑姬完成补刀时单步奖励应包含至少 +5.0 的补刀奖励，实际: {}",
        fiora_res.reward
    );
    assert_eq!(
        fiora_res.reward, -riven_res.reward,
        "双方奖励应严格满足零和对称性"
    );
}
