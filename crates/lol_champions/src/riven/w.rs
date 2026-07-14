use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::buffs::common_buffs::BuffCastBlock;
use lol_core::life::Health;
use lol_core::missile::CommandAttachedFieldCreate;
use lol_core::team::Team;

const RIVEN_W_STUN_DURATION: f32 = 0.75;
const RIVEN_W_STUN_RADIUS: f32 = 100.0;
const RIVEN_W_FIELD_DURATION: f32 = 0.25;
/// W 施法期间阻塞持续时间：8帧 at 30fps ≈ 0.2667s
pub const RIVEN_W_CAST_BLOCK_DURATION: f32 = 8.0 / 30.0;

pub struct PluginRivenW;

impl Plugin for PluginRivenW {
    fn build(&self, _app: &mut App) {}
}

pub fn cast_riven_w(commands: &mut Commands, entity: Entity, damage_amount: f32) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
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

    // W 施法期间自阻塞（阻止移动和其他技能）。
    // MovementBlock/CastBlock 由 PluginCc 的 On<Add, BuffCastBlock> 观察者自动桥接到自身。
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCastBlock::new(RIVEN_W_CAST_BLOCK_DURATION));
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
                // 眩晕：独立 buff 实体，标记由 PluginCc 观察者桥接（Buff 自己管自己）
                commands
                    .entity(target)
                    .with_related::<BuffOf>(DebuffStun::new(RIVEN_W_STUN_DURATION));
            }
        }
    }
}
