#![cfg(test)]

//! Fiora 被动（破绽 Vital）扇形视觉指示器测试。
//!
//! 被动会在敌方英雄身上标记一个要害方向；本组测试验证该要害的扇形视觉指示器
//! 随 Vital 的出现 / 方向变化 / 消失正确同步。
//!
//! 视觉系统直接以 `Vital` 组件为驱动源，因此多数用例直接在敌人身上挂一个已知
//! 方向的 Vital，绕过被动的随机方向，便于断言扇形朝向。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Quat, Transform};
use lol_core::base::direction::Direction;
use lol_core::damage::{CommandDamageCreate, DamageType};
use lol_core::life::Health;
use lol_core::movement::Movement;

use super::tests::build_headless;
use crate::fiora::passive::{
    FIORA_PASSIVE_ACTIVE_DURATION, FIORA_PASSIVE_DURATION, FioraVitalVisual, Vital,
};
use crate::test_utils::ChampionTestHarness;

const EPSILON: f32 = 1e-3;

/// Vital 朝向 → 扇形应指向的世界方向（与 `is_in_direction` 的语义一致：
/// 要害方向表示攻击者应从哪一侧接近，扇形指向该侧）。
fn direction_forward(direction: &Direction) -> Vec3 {
    match direction {
        Direction::Up => Vec3::new(0.0, 0.0, 1.0),
        Direction::Down => Vec3::new(0.0, 0.0, -1.0),
        Direction::Right => Vec3::new(1.0, 0.0, 0.0),
        Direction::Left => Vec3::new(-1.0, 0.0, 0.0),
    }
}

/// 直接在敌人身上挂一个指定方向的 Vital，绕过被动的随机方向选择。
fn spawn_vital(h: &mut ChampionTestHarness, enemy: Entity, direction: Direction) {
    h.app.world_mut().entity_mut(enemy).insert(Vital::new(
        direction,
        FIORA_PASSIVE_ACTIVE_DURATION,
        FIORA_PASSIVE_DURATION,
    ));
}

/// 收集场上所有 Vital 视觉指示器：(视觉实体, 目标实体, 朝向旋转)。
fn collect_visuals(h: &mut ChampionTestHarness) -> Vec<(Entity, Entity, Quat)> {
    let world = h.app.world_mut();
    let mut q = world.query::<(Entity, &FioraVitalVisual, &Transform)>();
    q.iter(world)
        .map(|(e, v, t)| (e, v.target, t.rotation))
        .collect()
}

/// Vital 出现时生成一个扇形视觉，且扇形朝向与 Vital 方向一致。
#[test]
fn fiora_passive_vital_visual_spawns_and_tracks_direction() {
    let mut h = build_headless("fiora_passive_visual_track");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    spawn_vital(&mut h, enemy, Direction::Right);
    h.advance(0.2);

    let visuals = collect_visuals(&mut h);
    assert_eq!(visuals.len(), 1, "应恰好生成一个 Vital 视觉指示器");
    let (_, target, rot) = visuals[0];
    assert_eq!(target, enemy, "视觉指示器应指向该敌人");
    assert!(
        (rot * Vec3::Z - direction_forward(&Direction::Right)).length() < EPSILON,
        "视觉扇形应朝向 Right（+X）方向"
    );

    // 改变 Vital 方向，视觉应跟随更新
    h.app
        .world_mut()
        .entity_mut(enemy)
        .get_mut::<Vital>()
        .unwrap()
        .direction = Direction::Up;
    h.advance(0.2);

    let visuals = collect_visuals(&mut h);
    assert_eq!(visuals.len(), 1, "Vital 仍在，视觉不应重复生成");
    let (_, _, rot) = visuals[0];
    assert!(
        (rot * Vec3::Z - direction_forward(&Direction::Up)).length() < EPSILON,
        "改变 Vital 方向后，视觉扇形应朝向 Up（+Z）方向"
    );

    h.finish();
}

/// 目标消失（死亡 / 离开）后，Vital 随之消失，视觉指示器应被回收。
#[test]
fn fiora_passive_vital_visual_despawns_when_target_gone() {
    let mut h = build_headless("fiora_passive_visual_despawn");
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    spawn_vital(&mut h, enemy, Direction::Up);
    h.advance(0.2);
    assert_eq!(
        collect_visuals(&mut h).len(),
        1,
        "Vital 出现后应生成视觉指示器"
    );

    // 敌人消失，Vital 随之消失，视觉应被回收
    h.app.world_mut().entity_mut(enemy).despawn();
    h.advance(0.2);
    assert_eq!(
        collect_visuals(&mut h).len(),
        0,
        "目标消失后视觉指示器应被回收"
    );

    h.finish();
}

/// 端到端：范围内敌方英雄应被被动标记 Vital，并随之生成视觉指示器。
///
/// 这同时验证了 `attach_fiora_passive_ability`：被动原本要求被动技能实体带
/// `AbilityFioraPassive` 标记，但该标记此前从未被挂上，导致 Vital 从不生成。
#[test]
fn fiora_passive_marks_enemy_in_range_with_vital() {
    let mut h = build_headless("fiora_passive_vital_spawn");
    // 范围内（VITAL_DISTANCE = 1000）的敌方英雄
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    h.advance(0.3);

    let has_vital = h.app.world().get::<Vital>(enemy).is_some();
    assert!(has_vital, "范围内敌方英雄应被被动标记 Vital");
    assert_eq!(
        collect_visuals(&mut h).len(),
        1,
        "Vital 生成后应出现视觉指示器"
    );

    h.finish();
}

// 复用 build_render 以便在开启 MOON_LOL_RUN_RENDER_TESTS 时录制扇形视觉视频；
// 未设置该环境变量时 harness 自动回退为无头模式，下面的断言仍然成立。
use super::tests::build_render;

#[test]
fn fiora_passive_vital_sector_render() {
    let mut h = build_render("fiora_passive_vital_sector");
    let _enemy = h.add_enemy(Vec3::new(250.0, 0.0, 0.0));
    // 让被动标记 Vital + 视觉生成
    h.advance(0.5);
    assert_eq!(
        collect_visuals(&mut h).len(),
        1,
        "场景中应有一个 Vital 视觉指示器"
    );
    // 推进一段时间以便录像
    h.advance(2.0);
    h.finish();
}

// ── 击破要害的治疗与移速 ──

/// 从匹配方向击破要害后，菲奥娜应获得移速加成（wiki：8%，1.5s）。
#[test]
fn fiora_passive_break_grants_move_speed() {
    let mut h = build_headless("fiora_passive_ms");
    let enemy = h.add_enemy(Vec3::new(450.0, 0.0, 0.0));
    // 菲奥娜在原点，敌人在 (450,0)：source.x < target.x -> 命中 Left 要害
    spawn_vital(&mut h, enemy, Direction::Left);
    h.advance(1.8); // 等要害进入活跃态

    let base_speed = h
        .app
        .world()
        .get::<Movement>(h.champion)
        .expect("菲奥娜应有 Movement")
        .speed;

    // 菲奥娜对敌人造成伤害 -> 触发 on_passive_damage_create -> 击破要害 -> 移速 buff
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::Physical,
        amount: 10.0,
        tag: None,
    });
    h.advance(0.2); // 让通用 update_move_speed_buff 应用 bonus

    let speed_after = h
        .app
        .world()
        .get::<Movement>(h.champion)
        .expect("菲奥娜应有 Movement")
        .speed;
    assert!(
        speed_after > base_speed + 20.0,
        "击破要害应获得移速加成：{} > {}",
        speed_after,
        base_speed
    );

    h.finish();
}

/// 从匹配方向击破要害后，菲奥娜应被治疗（生命值回升）。
#[test]
fn fiora_passive_break_heals_fiora() {
    let mut h = build_headless("fiora_passive_heal");
    let enemy = h.add_enemy(Vec3::new(450.0, 0.0, 0.0));
    spawn_vital(&mut h, enemy, Direction::Left);
    h.advance(1.8);

    let fiora_max = h
        .app
        .world()
        .get::<Health>(h.champion)
        .expect("菲奥娜应有 Health")
        .max;

    // 先让菲奥娜受伤（低于满血），便于观察治疗
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: h.champion,
        source: enemy,
        damage_type: DamageType::Physical,
        amount: 100.0,
        tag: None,
    });
    h.advance(0.1);
    let hp_damaged = h.health(h.champion);
    assert!(hp_damaged < fiora_max, "菲奥娜应已受伤");

    // 菲奥娜击破要害 -> 治疗菲奥娜
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::Physical,
        amount: 10.0,
        tag: None,
    });
    h.advance(0.1);
    let hp_after = h.health(h.champion);
    assert!(
        hp_after > hp_damaged,
        "击破要害应治疗菲奥娜：{} > {}",
        hp_after,
        hp_damaged
    );

    h.finish();
}
