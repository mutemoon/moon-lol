use bevy::prelude::*;
use lol_core::attack::EventAttackEnd;
use lol_core::base::buff::{Buff, Buffs};
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};

use crate::riven::Riven;

/// 锐雯被动额外伤害倍率
const RIVEN_PASSIVE_BONUS_RATIO: f32 = 0.2;

#[derive(Component, Clone, Debug, Default)]
#[require(Buff = Buff { name: "RivenPassive" })]
pub struct BuffRivenPassive;

/// 当锐雯造成伤害时，如果有被动层数，触发额外伤害并消耗一层
pub fn on_damage_create_trigger_bonus(
    trigger: On<EventAttackEnd>,
    mut commands: Commands,
    q_riven: Query<&Damage, With<Riven>>,
    q_buffs: Query<&Buffs>,
    q_buff_riven_passive: Query<&BuffRivenPassive>,
) {
    let source = trigger.entity;

    // 只处理锐雯造成的伤害
    let Ok(damage) = q_riven.get(source) else {
        return;
    };

    let Ok(buffs) = q_buffs.get(source) else {
        return;
    };

    // 查找被动buff
    for buff in buffs.iter() {
        if q_buff_riven_passive.get(buff).is_err() {
            continue;
        }

        let bonus_damage = damage.0 * RIVEN_PASSIVE_BONUS_RATIO;

        // 触发额外伤害
        commands.trigger(CommandDamageCreate {
            entity: trigger.target,
            source,
            damage_type: DamageType::Physical,
            amount: bonus_damage,
        });

        commands.entity(buff).despawn();
        info!("{:?} 锐雯被动触发，额外伤害: {:.1}", source, bonus_damage);

        return;
    }
}
