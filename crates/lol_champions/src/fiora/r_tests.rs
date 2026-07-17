#![cfg(test)]

//! Fiora R（决斗 / Grand Challenge）单元测试。
//!
//! R 的核心机制：选定一个敌方英雄，在其身上揭露四个要害（持续约 8s）。
//! 从匹配方向攻击会击破要害并造成最大生命值真实伤害；四个要害全破、或目标
//! 在击破至少一个要害后死亡时，菲奥娜获得治疗光环。R 期间菲奥娜获得 30% 移速。

use bevy::math::{Vec2, Vec3};
use bevy::prelude::{Entity, Transform};
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::base::direction::Direction;
use lol_core::damage::{CommandDamageCreate, DamageType};

use super::tests::build_headless;
use crate::fiora::passive::Vital;
use crate::fiora::r::{BuffFioraR, BuffFioraRHeal};
use crate::test_utils::ChampionTestHarness;

const EPSILON: f32 = 1e-3;

/// R 应把四要害挂在「目标敌方英雄」身上，而不是菲奥娜自身。
#[test]
fn fiora_r_attaches_vitals_to_target() {
    let mut h = build_headless("fiora_r_target_attach");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0)); // 在 R 射程 500 内
    let caster = h.champion;
    let mana_before = h.mana();

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.1);

    // 目标身上应有 BuffFioraR（holder == 敌人）
    let enemy_has_r = r_buff_holder(&h, enemy).is_some();
    assert!(enemy_has_r, "R 应把四要害挂在目标敌方英雄身上");

    // 菲奥娜自身不应持有 BuffFioraR
    let self_has_r = r_buff_holder(&h, caster).is_some();
    assert!(!self_has_r, "R 不应把要害挂在菲奥娜自身身上");

    // 四个要害方向齐全
    let vitals = r_buff_vitals(&h, enemy).expect("目标应有 R 要害");
    assert_eq!(vitals.len(), 4, "R 应揭露四个要害");
    assert!(
        !h.can_cast(3),
        "R 施放后应进入冷却"
    );
    assert!(h.mana() < mana_before, "R 施放应消耗法力");

    h.finish();
}

/// 击破一个方向匹配的 R 要害应造成最大生命值真实伤害。
#[test]
fn fiora_r_vital_break_deals_true_damage() {
    let mut h = build_headless("fiora_r_true_damage");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    // 菲奥娜在原点 (0,0)，敌人在 (300,0)：source.x < target.x -> 命中 Left 要害。

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6); // R 进入活跃态
    // 移除被动可能挂上的 Vital，隔离 R 的真伤断言
    h.app.world_mut().entity_mut(enemy).remove::<Vital>();

    let hp_before = h.health(enemy);
    // 从菲奥娜位置发起一次物理伤害（触发 on_r_damage_create）
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::Physical,
        amount: 10.0,
        tag: None,
    });
    h.advance(0.1);

    let hp_loss = hp_before - h.health(enemy);
    // R 真伤 = 3% maxHP = 180（1 级），加上 10 物理共 190。断言 > 150 以隔离真伤。
    assert!(
        hp_loss > 150.0,
        "击破 R 要害应造成最大生命值真实伤害，实际损失 {:.1}",
        hp_loss
    );

    h.finish();
}

/// 击破全部四个要害应触发治疗光环。
#[test]
fn fiora_r_all_four_vitals_trigger_heal_aura() {
    let mut h = build_headless("fiora_r_heal_all_four");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6);
    h.app.world_mut().entity_mut(enemy).remove::<Vital>();

    // 从四个方向各击破一个要害：Left(西)、Right(东)、Up(北 +Z)、Down(南 -Z)
    damage_from(&mut h, enemy, Vec3::new(0.0, 0.0, 0.0)); // source.x<target.x -> Left
    assert_eq!(
        r_buff_vitals(&h, enemy).map(|v| v.len()).unwrap_or(0),
        3,
        "击破第一个要害后应剩 3 个"
    );
    damage_from(&mut h, enemy, Vec3::new(600.0, 0.0, 0.0)); // Right
    assert_eq!(
        r_buff_vitals(&h, enemy).map(|v| v.len()).unwrap_or(0),
        2,
        "击破第二个要害后应剩 2 个"
    );
    damage_from(&mut h, enemy, Vec3::new(300.0, 0.0, 300.0)); // Up (source.z>target.z)
    assert_eq!(
        r_buff_vitals(&h, enemy).map(|v| v.len()).unwrap_or(0),
        1,
        "击破第三个要害后应剩 1 个"
    );
    damage_from(&mut h, enemy, Vec3::new(300.0, 0.0, -300.0)); // Down
    // 四要害全破 -> 治疗光环
    assert!(has_heal_aura(&h), "击破全部四个要害应触发治疗光环");
    assert!(
        r_buff_holder(&h, enemy).is_none(),
        "四要害全破后 R buff 应被移除"
    );

    h.finish();
}

/// 目标在击破至少一个要害后死亡，也应触发治疗光环。
#[test]
fn fiora_r_target_death_triggers_heal_aura() {
    let mut h = build_headless("fiora_r_heal_on_death");
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.6);
    h.app.world_mut().entity_mut(enemy).remove::<Vital>();

    let hp_before = h.health(enemy);
    // 致死伤害：物理伤害超过目标生命值，并击破一个要害
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::Physical,
        amount: hp_before + 1000.0,
        tag: None,
    });
    h.advance(0.1);

    assert!(h.health(enemy) <= EPSILON, "目标应死亡");
    assert!(
        has_heal_aura(&h),
        "目标击破至少一个要害后死亡应触发治疗光环"
    );

    h.finish();
}

// ── helpers ──
//
// 只用 `&World` + `World::get`（不可变）查 buff，避免 `world_mut()` 借用冲突。

/// 目标是否持有 R 要害 buff；若持有返回其 holder（即目标自身）。
fn r_buff_holder(h: &ChampionTestHarness, target: Entity) -> Option<Entity> {
    let world = h.app.world();
    let buffs = world.get::<Buffs>(target)?;
    for buff_entity in buffs.iter() {
        if world.get::<BuffFioraR>(*buff_entity).is_some() {
            return world.get::<BuffOf>(*buff_entity).map(|bo| bo.0);
        }
    }
    None
}

/// 目标身上 R 要害的方向列表。
fn r_buff_vitals(h: &ChampionTestHarness, target: Entity) -> Option<Vec<Direction>> {
    let world = h.app.world();
    let buffs = world.get::<Buffs>(target)?;
    for buff_entity in buffs.iter() {
        if let Some(b) = world.get::<BuffFioraR>(*buff_entity) {
            return Some(b.vitals.clone());
        }
    }
    None
}

/// 菲奥娜是否持有治疗光环。
fn has_heal_aura(h: &ChampionTestHarness) -> bool {
    let world = h.app.world();
    let Some(buffs) = world.get::<Buffs>(h.champion) else {
        return false;
    };
    buffs
        .iter()
        .any(|be| world.get::<BuffFioraRHeal>(*be).is_some())
}

/// 把菲奥娜移到 `fiora_pos`，然后对 `enemy` 发起一次小物理伤害（触发 R 要害判定）。
fn damage_from(h: &mut ChampionTestHarness, enemy: Entity, fiora_pos: Vec3) {
    h.app
        .world_mut()
        .entity_mut(h.champion)
        .get_mut::<Transform>()
        .unwrap()
        .translation = fiora_pos;
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source: h.champion,
        damage_type: DamageType::Physical,
        amount: 10.0,
        tag: None,
    });
    h.advance(0.1);
}
