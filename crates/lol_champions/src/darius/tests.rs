#![cfg(test)]

use bevy::ecs::entity::Entity;
use bevy::math::{Vec2, Vec3};
use lol_core::attack::CommandAttackStart;
use lol_core::base::ability_resource::AbilityResource;
use lol_core::base::buff::Buffs;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::on_hit::{BuffOnHitBonusDamage, BuffOnHitCounter, BuffOnHitSlow};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};

use crate::darius::Darius;
use crate::darius::buffs::{BuffDariusBleed, BuffDariusMight};
use crate::test_utils::*;

/// Give Darius enough mana to cast skills (fixes exported config having 0.07 mana)
pub fn give_mana(h: &mut ChampionTestHarness) {
    if let Some(mut ar) = h.app.world_mut().get_mut::<AbilityResource>(h.champion) {
        ar.value = 1000.0;
        ar.max = 1000.0;
    }
}

/// Darius 的当前攻击力。
pub fn darius_ad(h: &ChampionTestHarness) -> f32 {
    h.app
        .world()
        .get::<Damage>(h.champion)
        .map(|d| d.0)
        .unwrap_or(0.0)
}

/// 让 Darius 对敌人造成一次物理伤害（触发出血叠加）。
pub fn darius_hit(h: &mut ChampionTestHarness, enemy: Entity, amount: f32) {
    let source = h.champion;
    h.app.world_mut().trigger(CommandDamageCreate {
        entity: enemy,
        source,
        damage_type: DamageType::Physical,
        amount,
        tag: None,
    });
}

/// 读取敌人身上的出血层数（无出血返回 None）。
pub fn bleed_stacks(h: &ChampionTestHarness, enemy: Entity) -> Option<u8> {
    let buffs = h.app.world().get::<Buffs>(enemy)?;
    for b in buffs.iter() {
        if let Some(bleed) = h.app.world().get::<BuffDariusBleed>(*b) {
            return Some(bleed.stacks);
        }
    }
    None
}

/// 判断 Darius 是否处于诺克萨斯之力（血怒）状态。
pub fn has_might(h: &ChampionTestHarness) -> bool {
    let Some(buffs) = h.app.world().get::<Buffs>(h.champion) else {
        return false;
    };
    buffs
        .iter()
        .any(|b| h.app.world().get::<BuffDariusMight>(*b).is_some())
}

pub fn darius_config() -> ChampionHarnessConfig {
    ChampionHarnessConfig {
        champion_dir: "darius",
        config_path: "characters/Darius/config.ron",
        skin_path: "characters/Darius/skins/skin0.ron",
        add_champion_plugin: |app| {
            app.add_plugins(crate::darius::PluginDarius);
        },
    }
}

pub fn build_headless(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Darius>(name, HarnessMode::Headless, &darius_config())
}

pub fn build_render(name: &str) -> ChampionTestHarness {
    ChampionTestHarness::build::<Darius>(name, HarnessMode::Render, &darius_config())
}

/// Test that Q deals damage to enemies in range
#[test]
fn darius_q_deals_damage() {
    let mut h = build_headless("darius_q_damage");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.5);

    assert!(h.health(enemy) < hp_before, "Q 应造成伤害，敌人血量应下降");
    assert!(!h.can_cast(0), "Q 施放后应进入冷却");
    assert!(h.mana() < mana_before, "Q 施放应消耗法力");
    h.finish();
}

/// Test that W puts skill on cooldown and consumes mana
#[test]
fn darius_w_goes_on_cooldown() {
    let mut h = build_headless("darius_w_cooldown");
    give_mana(&mut h);
    let _enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));
    let mana_before = h.mana();

    // Cast W
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);

    // W should be on cooldown after casting
    assert!(
        !h.can_cast(1),
        "W 施放后应进入冷却"
    );
    assert!(
        h.mana() < mana_before,
        "W 施放应消耗法力"
    );
    h.finish();
}

/// Test that E cast goes on cooldown
#[test]
fn darius_e_cast_goes_on_cooldown() {
    let mut h = build_headless("darius_e_cooldown");
    give_mana(&mut h);
    let _enemy = h.add_enemy(Vec3::new(400.0, 0.0, 0.0));
    let mana_before = h.mana();

    // E has 535 range, enemy is at 400
    h.cast_skill(2, Vec2::new(400.0, 0.0)).advance(0.3);

    // E should be on cooldown after casting
    assert!(
        !h.can_cast(2),
        "E 施放后应进入冷却"
    );
    assert!(
        h.mana() < mana_before,
        "E 施放应消耗法力"
    );
    h.finish();
}

/// Test that R deals damage to enemy
#[test]
fn darius_r_deals_damage() {
    let mut h = build_headless("darius_r_damage");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(300.0, 0.0, 0.0));
    let hp_before = h.health(enemy);
    let mana_before = h.mana();

    h.cast_skill(3, Vec2::new(300.0, 0.0)).advance(0.5);

    assert!(h.health(enemy) < hp_before, "R 应造成伤害");
    assert!(!h.can_cast(3), "R 施放后应进入冷却");
    assert!(h.mana() < mana_before, "R 施放应消耗法力");
    h.finish();
}

/// Test that Q applies hemorrhage stacks to enemy
#[test]
fn darius_q_applies_hemorrhage() {
    let mut h = build_headless("darius_q_hemorrhage");
    give_mana(&mut h);
    // Q has 270 range, place enemy at 200
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    // Cast Q to apply hemorrhage
    h.cast_skill(0, Vec2::new(200.0, 0.0)).advance(0.5);

    // Check that enemy has hemorrhage buff
    let buffs = h.app.world().get::<Buffs>(enemy);
    assert!(buffs.is_some(), "敌人应该有 Buffs 组件");

    let buffs = buffs.unwrap();
    let mut found_bleed = false;
    for buff_entity in buffs.iter() {
        if h.app
            .world()
            .get::<crate::darius::buffs::BuffDariusBleed>(*buff_entity)
            .is_some()
        {
            found_bleed = true;
            break;
        }
    }
    assert!(found_bleed, "Q 应给敌人叠加出血效果");
    h.finish();
}

/// Test that W applies on-hit buffs to the caster (减速由普攻命中时统一触发)
#[test]
fn darius_w_applies_on_hit_buffs() {
    let mut h = build_headless("darius_w_on_hit");
    give_mana(&mut h);
    let enemy = h.add_enemy(Vec3::new(200.0, 0.0, 0.0));

    // Cast W
    h.cast_skill(1, Vec2::new(200.0, 0.0)).advance(0.1);

    // 检查 Darius 身上有 BuffOnHitCounter
    let buffs = h.app.world().get::<Buffs>(h.champion);
    assert!(buffs.is_some(), "Darius 应该有 Buffs 组件");

    let buffs = buffs.unwrap();
    let mut found_counter = false;
    let mut found_damage = false;
    let mut found_slow = false;
    for buff_entity in buffs.iter() {
        if h.app
            .world()
            .get::<BuffOnHitCounter>(*buff_entity)
            .is_some()
        {
            found_counter = true;
        }
        if h.app
            .world()
            .get::<BuffOnHitBonusDamage>(*buff_entity)
            .is_some()
        {
            found_damage = true;
        }
        if h.app.world().get::<BuffOnHitSlow>(*buff_entity).is_some() {
            found_slow = true;
        }
    }
    assert!(found_counter, "W 应添加 BuffOnHitCounter");
    assert!(found_damage, "W 应添加 BuffOnHitBonusDamage");
    assert!(found_slow, "W 应添加 BuffOnHitSlow");

    // 验证：W 强化普攻命中后减速生效
    // 开始普攻：强化普攻会在命中时消耗 on-hit 并对敌人施加减速（持续 1s）
    h.app.world_mut().trigger(CommandAttackStart {
        entity: h.champion,
        target: enemy,
    });
    // 推进到命中；减速持续 1s，需在其过期前检查
    h.advance(0.6);

    // 检查敌人是否有减速
    let buffs = h.app.world().get::<Buffs>(enemy);
    assert!(buffs.is_some(), "敌人应该有 Buffs 组件");

    let buffs = buffs.unwrap();
    let mut found_slow_on_enemy = false;
    for buff_entity in buffs.iter() {
        if h.app.world().get::<DebuffSlow>(*buff_entity).is_some() {
            found_slow_on_enemy = true;
            break;
        }
    }
    assert!(found_slow_on_enemy, "W 强化普攻应施加减速效果");
    h.finish();
}
