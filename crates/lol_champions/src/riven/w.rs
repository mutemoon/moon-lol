use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_core::life::Health;
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::movement::MovementBlock;
use lol_core::team::Team;

use crate::riven::buffs::BuffStun;

const RIVEN_W_STUN_DURATION: f32 = 0.75;
const RIVEN_W_STUN_RADIUS: f32 = 300.0;
const RIVEN_W_FIELD_DURATION: f32 = 0.25;

pub struct PluginRivenW;

impl Plugin for PluginRivenW {
    fn build(&self, _app: &mut App) {}
}

pub fn cast_riven_w(commands: &mut Commands, entity: Entity, damage_amount: f32) {
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Riven_W_Cast"),
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "Spell2".to_string(),
        repeat: false,
        duration: None,
    });

    // 使用通用附着伤害场替代 ActionDamage
    commands.trigger(CommandAttachedFieldCreate {
        entity,
        radius: RIVEN_W_STUN_RADIUS,
        damage: damage_amount,
        duration: RIVEN_W_FIELD_DURATION,
        grow_from: None,
        grow_duration: None,
    });
}

pub fn apply_w_stun_to_targets(
    commands: &mut Commands,
    caster_entity: Entity,
    q_transform: &Query<&Transform>,
    q_team: &Query<&Team>,
    q_targets: &Query<(Entity, &Team, &Transform, &Health)>,
) {
    if let (Ok(transform), Ok(team)) = (q_transform.get(caster_entity), q_team.get(caster_entity)) {
        for (target, target_team, target_transform, _) in q_targets.iter() {
            if target_team == team {
                continue;
            }
            if target_transform.translation.distance(transform.translation) <= RIVEN_W_STUN_RADIUS {
                commands.entity(target).insert(BuffStun {
                    timer: Timer::from_seconds(RIVEN_W_STUN_DURATION, TimerMode::Once),
                });
                commands.entity(target).insert(MovementBlock);
            }
        }
    }
}

/// 更新眩晕计时
pub fn update_riven_stun(
    mut commands: Commands,
    mut q_stun: Query<(Entity, &mut BuffStun)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut stun) in q_stun.iter_mut() {
        stun.timer.tick(time.delta());
        if stun.timer.is_finished() {
            commands.entity(entity).remove::<BuffStun>();
            commands.entity(entity).remove::<MovementBlock>();
        }
    }
}
