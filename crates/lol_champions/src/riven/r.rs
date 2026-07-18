use bevy::prelude::*;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::attack::Attack;
use lol_core::base::buff::{BuffOf, Buffs};
use lol_core::damage::Damage;
use lol_core::missile::{CommandMissileCreate, MissileMissingHpScaling};
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value,
    get_skill_value,
};

use crate::riven::Riven;
use crate::riven::buffs::BuffRivenR;

pub fn on_riven_r(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_riven: Query<(), With<Riven>>,
    mut q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
    mut q_damage: Query<&mut Damage>,
    mut q_attack: Query<&mut Attack>,
    q_transform: Query<&Transform>,
    res_spells: Res<Assets<Spell>>,
    res_asset_server: Res<AssetServer>,
) {
    let entity = trigger.event_target();
    if q_riven.get(entity).is_err() {
        return;
    }

    let Ok((skill, mut cooldown, recast)) = q_skill.get_mut(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::R) {
        return;
    }

    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };

    let damage_value = q_damage.get(entity).map(|d| d.0).unwrap_or(64.0);

    let stage = recast.map(|w| w.stage).unwrap_or(1);

    match stage {
        2 => {
            // Wind Slash — 三枚导弹
            let missile_handles = [
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileRight.ron"),
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileCenter.ron"),
                res_asset_server.load("characters/riven/spells/RivenWindslashMissileLeft.ron"),
            ];
            cast_riven_wind_slash(
                &mut commands,
                entity,
                &missile_handles,
                &q_transform,
                spell_obj,
                skill.level,
                damage_value,
            );

            commands
                .entity(trigger.skill_entity)
                .remove::<SkillRecastWindow>();
        }
        _ => {
            // 初次 R — 从 RON 读取增伤、攻击距离加成、持续时间，开启连招窗口
            let bonus_ad_ratio =
                get_skill_data_value(spell_obj, "PercentBonusAD", skill.level).unwrap_or(0.25);
            let bonus_range =
                get_skill_data_value(spell_obj, "TooltipAttackRange", skill.level).unwrap_or(75.0);
            let duration = get_skill_data_value(spell_obj, "Duration", skill.level).unwrap_or(15.0);

            let bonus_ad = damage_value * bonus_ad_ratio;

            if let Ok(mut dmg) = q_damage.get_mut(entity) {
                dmg.0 += bonus_ad;
            }
            if let Ok(mut atk) = q_attack.get_mut(entity) {
                atk.range += bonus_range;
            }

            commands.entity(entity).with_related::<BuffOf>(BuffRivenR {
                timer: Timer::from_seconds(duration, TimerMode::Once),
                bonus_ad_ratio,
                bonus_range,
            });

            // 覆盖冷却为真实 R 冷却（cooldownTime 数组以 level 为索引以避免 nil 占位）
            let r_cd = spell_obj
                .spell_data
                .as_ref()
                .and_then(|d| d.cooldown_time.as_ref())
                .and_then(|v| v.get(skill.level).copied())
                .unwrap_or(120.0);
            cooldown.duration = r_cd;
            cooldown.timer = Some(Timer::from_seconds(r_cd, TimerMode::Once));

            commands
                .entity(trigger.skill_entity)
                .insert(SkillRecastWindow::new(2, 2, duration));

            commands.trigger(CommandAnimationPlay {
                entity,
                hash: "Spell4A".to_string(),
                repeat: false,
                duration: None,
            });
        }
    }
}

fn cast_riven_wind_slash(
    commands: &mut Commands,
    entity: Entity,
    missile_handles: &[Handle<Spell>; 3],
    q_transform: &Query<&Transform>,
    spell_obj: &Spell,
    skill_level: usize,
    total_ad: f32,
) {
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let forward = transform.forward();

    let min_damage = get_skill_value(spell_obj, "min_damage", skill_level, |stat| {
        if stat == 2 { total_ad } else { 0.0 }
    })
    .unwrap_or(50.0);

    let max_damage = get_skill_value(spell_obj, "max_damage", skill_level, |stat| {
        if stat == 2 { total_ad } else { 0.0 }
    })
    .unwrap_or(150.0);

    // 按目标已损失生命值缩放：命中时由导弹系统逐目标重算，
    // 血越少越痛（斩杀）。无 Health 数据时回退到 min_damage。
    let scaling = MissileMissingHpScaling {
        min_damage,
        max_damage,
    };

    let range = 1100.0;
    let spread_angle = 7.0_f32.to_radians();
    let origin = transform.translation;

    // 三枚导弹：左、中、右（相对于 forward 方向的偏移角度）
    let angles = [-spread_angle, 0.0, spread_angle];

    for (angle, handle) in angles.iter().zip(missile_handles.iter()) {
        let rot = Quat::from_axis_angle(Vec3::Y, *angle);
        let world_dir = rot * forward;
        let destination = origin + world_dir * range;

        commands.trigger(CommandMissileCreate {
            entity,
            target: None,
            destination: Some(destination),
            spell: handle.clone(),
            damage: min_damage,
            speed: None,
            particle_hash: None,
            sticky: false,
            pass_through: true,
            collision_target: default(),
            missing_hp_scaling: Some(scaling),
        });
    }
}

/// 更新 R buff 计时器，到期后移除并恢复属性
pub fn update_riven_buffs(
    mut commands: Commands,
    mut q_champion: Query<(&Buffs, &mut Damage, &mut Attack), With<Riven>>,
    mut q_buff_r: Query<(Entity, &mut BuffRivenR)>,
    time: Res<Time<Fixed>>,
) {
    // 收集到期的 buff 及其存储的加成值，避免二次借用
    let mut expired: Vec<(Entity, f32, f32)> = Vec::new();

    for (buff_entity, mut buff) in q_buff_r.iter_mut() {
        buff.timer.tick(time.delta());
        if buff.timer.is_finished() {
            expired.push((buff_entity, buff.bonus_ad_ratio, buff.bonus_range));
        }
    }

    if expired.is_empty() {
        return;
    }

    let expired_entities: Vec<Entity> = expired.iter().map(|(e, _, _)| *e).collect();

    // 遍历所有 Riven 实体，检查它们的 buffs 是否有过期的 R 被动
    for (buffs, mut damage, mut attack) in q_champion.iter_mut() {
        for buff_entity in buffs.iter() {
            if let Some((_, bonus_ad_ratio, bonus_range)) =
                expired.iter().find(|(e, _, _)| *e == buff_entity)
            {
                // R 到期，用 BuffRivenR 中存储的数值恢复属性
                damage.0 /= 1.0 + bonus_ad_ratio;
                attack.range -= bonus_range;
                break;
            }
        }
    }

    for buff_entity in expired_entities {
        commands.entity(buff_entity).despawn();
    }
}
