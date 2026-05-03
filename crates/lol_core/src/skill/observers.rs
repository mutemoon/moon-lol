use bevy::ecs::event::EntityEvent;
use bevy::log::{debug, info};
use bevy::prelude::{
    Assets, Commands, Entity, Fixed, On, Query, Res, ResMut, Time, Timer, TimerMode, warn,
};
use lol_base::spell::Spell;

use super::enums::SkillCooldownMode;
use super::events::{
    CommandSkillLevelUp, CommandSkillStart, EventSkillCast, SkillCastFailureReason, SkillCastLog,
    SkillCastRecord, SkillCastResult,
};
use super::{CoolDown, SkillPoints, SkillRecastWindow, Skills};
use crate::base::ability_resource::AbilityResource;
use crate::base::level::{EventLevelUp, Level};
use crate::skill::Skill;

fn push_skill_log(
    log: &mut ResMut<SkillCastLog>,
    caster: Entity,
    skill_entity: Option<Entity>,
    index: usize,
    slot: Option<super::SkillSlot>,
    point: bevy::prelude::Vec2,
    result: SkillCastResult,
) {
    info!(
        "{:?}",
        SkillCastRecord {
            caster,
            skill_entity,
            index,
            slot,
            point,
            result,
        }
    );
    log.0.push(SkillCastRecord {
        caster,
        skill_entity,
        index,
        slot,
        point,
        result,
    });
}

pub fn on_skill_cast(
    trigger: On<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    res_assets_spell_object: Res<Assets<Spell>>,
    mut q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    mut log: ResMut<SkillCastLog>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        push_skill_log(
            &mut log,
            entity,
            None,
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSkills),
        );
        return;
    };
    let Some(&skill_entity) = skills.get(trigger.index) else {
        push_skill_log(
            &mut log,
            entity,
            None,
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::InvalidSkillIndex),
        );
        return;
    };
    let Ok((skill, mut cooldown_state, recast_window)) = q_skill.get_mut(skill_entity) else {
        warn!(
            "skill_entity {:?} not found or missing Skill component",
            skill_entity
        );
        commands.entity(skill_entity).log_components();
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            None,
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSkillEntity),
        );
        return;
    };

    // Skip cooldown check if there's an active recast window (e.g., Riven Q stages)
    let can_cast_despite_cooldown = recast_window
        .map(|w| !w.timer.is_finished())
        .unwrap_or(false);

    if !can_cast_despite_cooldown {
        if let Some(ref timer) = cooldown_state.timer {
            if !timer.is_finished() {
                debug!(
                    "{} 技能 {} 冷却中，剩余 {:.2}s",
                    entity,
                    trigger.index,
                    timer.remaining_secs()
                );
                push_skill_log(
                    &mut log,
                    entity,
                    Some(skill_entity),
                    trigger.index,
                    Some(skill.slot),
                    trigger.point,
                    SkillCastResult::Failed(SkillCastFailureReason::CoolingDown),
                );
                return;
            }
        }
    }

    let Some(spell_object) = res_assets_spell_object.get(&skill.spell) else {
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingSpellObject),
        );
        return;
    };

    if skill.level == 0 {
        debug!("{} 技能 {} 未学习，无法释放", entity, trigger.index);
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::NotLearned),
        );
        return;
    }

    let Ok(mut ability_resource) = q_ability_resource.get_mut(entity) else {
        push_skill_log(
            &mut log,
            entity,
            Some(skill_entity),
            trigger.index,
            Some(skill.slot),
            trigger.point,
            SkillCastResult::Failed(SkillCastFailureReason::MissingAbilityResource),
        );
        return;
    };

    if let Some(ref mana) = spell_object.spell_data.as_ref().unwrap().mana {
        let &current_mana = mana.get(skill.level as usize).unwrap();

        if ability_resource.value < current_mana {
            debug!(
                "{} 技能 {} 蓝量不足，需要 {:.0}，当前 {:.0}",
                entity, trigger.index, current_mana, ability_resource.value
            );
            push_skill_log(
                &mut log,
                entity,
                Some(skill_entity),
                trigger.index,
                Some(skill.slot),
                trigger.point,
                SkillCastResult::Failed(SkillCastFailureReason::InsufficientAbilityResource),
            );
            return;
        }

        ability_resource.value -= current_mana;
        debug!(
            "{} 技能 {} 消耗 {:.0} 蓝量，剩余 {:.0}",
            entity, trigger.index, current_mana, ability_resource.value
        );
    }

    push_skill_log(
        &mut log,
        entity,
        Some(skill_entity),
        trigger.index,
        Some(skill.slot),
        trigger.point,
        SkillCastResult::Started,
    );

    let cast_event = EventSkillCast {
        entity,
        skill_entity,
        index: trigger.index,
        point: trigger.point,
    };

    debug!("{} 技能 {} 进入代码驱动观察者流程", entity, trigger.index);
    commands.trigger(cast_event);

    if skill.cooldown_mode == SkillCooldownMode::AfterCast {
        cooldown_state.timer = Some(Timer::from_seconds(
            cooldown_state.duration,
            TimerMode::Once,
        ));
        debug!(
            "{} 技能 {} 开始冷却 {}s",
            entity, trigger.index, cooldown_state.duration
        );
    }
}

pub fn on_skill_level_up(
    trigger: On<CommandSkillLevelUp>,
    skills: Query<&Skills>,
    mut q_skill: Query<&mut Skill>,
    mut q_skill_points: Query<(&Level, &mut SkillPoints)>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.get(trigger.index) else {
        return;
    };
    let Ok(mut skill) = q_skill.get_mut(skill_entity) else {
        return;
    };
    let Ok((level, mut skill_points)) = q_skill_points.get_mut(entity) else {
        return;
    };

    debug!("{} 尝试升级技能: 索引 {}", entity, trigger.index);

    if skill_points.0 == 0 {
        debug!("{} 升级失败: 技能点不足", entity);
        return;
    }

    // 1 级只能加点 q w e，6 级才能加点 r，6 级前一个技能最多加 3 点
    if level.value < 6 {
        if trigger.index == 3 {
            debug!(
                "{} 升级失败: 等级 {} 小于 6 级不能升级大招",
                entity, level.value
            );
            return;
        }
        if skill.level >= 3 {
            debug!(
                "{} 升级失败: 等级 {} 小于 6 级，技能 {} 已达上限 (3)",
                entity, level.value, trigger.index
            );
            return;
        }
    }

    skill.level += 1;
    skill_points.0 -= 1;
    debug!(
        "{} 技能升级成功: 索引 {}, 新等级 {}, 剩余技能点 {}",
        entity, trigger.index, skill.level, skill_points.0
    );
}

pub fn on_level_up(event: On<EventLevelUp>, mut q_skill_points: Query<&mut SkillPoints>) {
    let entity = event.event_target();
    if let Ok(mut skill_points) = q_skill_points.get_mut(entity) {
        skill_points.0 += event.delta;
        debug!(
            "{} 升级: 获得 {} 技能点，当前技能点 {}",
            entity, event.delta, skill_points.0
        );
    }
}

pub fn update_skill_recast_windows(
    time: Res<Time<Fixed>>,
    mut commands: Commands,
    mut q_skill_window: Query<(Entity, &mut super::SkillRecastWindow)>,
) {
    for (entity, mut window) in q_skill_window.iter_mut() {
        window.timer.tick(time.delta());
        if window.timer.is_finished() {
            commands.entity(entity).remove::<super::SkillRecastWindow>();
        }
    }
}
