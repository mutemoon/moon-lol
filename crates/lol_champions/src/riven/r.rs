use bevy::prelude::*;
use lol_base::spell::Spell;
use lol_core::attack::Attack;
use lol_core::base::buff::Buffs;
use lol_core::damage::Damage;
use lol_core::life::Health;
use lol_core::missile::CommandMissileCreate;
use lol_core::skill::get_skill_value;
use lol_core::team::Team;

use crate::riven::Riven;
use crate::riven::buffs::BuffRivenR;

pub struct PluginRivenR;

impl Plugin for PluginRivenR {
    fn build(&self, _app: &mut App) {}
}

pub fn cast_riven_wind_slash(
    commands: &mut Commands,
    entity: Entity,
    missile_handles: &[Handle<Spell>; 3],
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_targets: &Query<(Entity, &Team, &Transform, &Health)>,
    spell_obj: &Spell,
    skill_level: usize,
    total_ad: f32,
) {
    let Ok(transform) = q_transform.get(entity) else {
        return;
    };
    let Ok(_team) = q_team.get(entity) else {
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

    // 平均生命值比例用于计算导弹伤害（取目标平均）
    let avg_hp_ratio = {
        let mut total_ratio = 0.0f32;
        let mut count = 0u32;
        for (_, target_team, _, health) in q_targets.iter() {
            if target_team == _team {
                continue;
            }
            total_ratio += (health.value / health.max).clamp(0.0, 1.0);
            count += 1;
        }
        if count > 0 {
            total_ratio / count as f32
        } else {
            1.0
        }
    };
    let missing_hp_ratio = 1.0 - avg_hp_ratio;
    let damage = min_damage + (max_damage - min_damage) * missing_hp_ratio;

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
            damage,
            speed: None,
            particle_hash: None,
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
    let mut expired = Vec::new();

    for (buff_entity, mut buff) in q_buff_r.iter_mut() {
        buff.timer.tick(time.delta());
        if buff.timer.is_finished() {
            expired.push(buff_entity);
        }
    }

    if expired.is_empty() {
        return;
    }

    // 遍历所有 Riven 实体，检查它们的 buffs 是否有过期的 R 被动
    for (buffs, mut damage, mut attack) in q_champion.iter_mut() {
        for buff in buffs.iter() {
            if expired.contains(&buff) {
                // R 到期，恢复基础属性
                // 伤害和攻击距离已在初次 R 时直接修改，这里回退
                damage.0 = damage.0 * (1.0 - 0.25 / 1.25); // base_ad = total / 1.25
                attack.range -= 75.0;
                commands.entity(buff).despawn();
                break;
            }
        }
    }
}
