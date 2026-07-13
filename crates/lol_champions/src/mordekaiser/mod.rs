pub mod buffs;
pub mod e;
pub mod passive;
pub mod q;
pub mod r;
pub mod w;

#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

#[derive(Default)]
pub struct PluginMordekaiser;

impl Plugin for PluginMordekaiser {
    fn build(&self, app: &mut App) {
        app.add_observer(on_mordekaiser_skill_cast);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Mordekaiser"))]
#[reflect(Component)]
pub struct Mordekaiser;

/// 莫德凯撒施法观察者：统一管线校验通过后派发，按技能槽位分发到各技能模块。
///
/// 当前为基础框架阶段，各技能以 stub 形式占位，仅播放动画并记录施法意图，
/// 具体伤害 / 位移 / 状态逻辑待后续按 TDD 流程实现。
fn on_mordekaiser_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_morde: Query<(), With<Mordekaiser>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_morde.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => q::cast_mordekaiser_q(&mut commands, entity, trigger.point),
        SkillSlot::W => w::cast_mordekaiser_w(&mut commands, entity),
        SkillSlot::E => e::cast_mordekaiser_e(&mut commands, entity, trigger.point),
        SkillSlot::R => r::cast_mordekaiser_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}
