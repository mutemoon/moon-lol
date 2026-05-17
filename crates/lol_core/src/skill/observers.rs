use bevy::ecs::event::EntityEvent;
use bevy::log::debug;
use bevy::prelude::{
    Assets, Commands, Entity, Fixed, On, Query, Res, ResMut, Time, Timer, TimerMode, With,
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
use crate::life::Death;
use crate::movement::CastBlock;
use crate::skill::Skill;

pub fn on_skill_cast(
    trigger: On<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    q_cast_block: Query<(), With<CastBlock>>,
    res_assets_spell_object: Res<Assets<Spell>>,
    mut q_skill: Query<(&Skill, &mut CoolDown, Option<&SkillRecastWindow>)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
    mut log: ResMut<SkillCastLog>,
    q_death: Query<(), With<Death>>,
) {
    let entity = trigger.event_target();
    let mut record = SkillCastRecord {
        caster: entity,
        skill_entity: None,
        index: trigger.index,
        slot: None,
        point: trigger.point,
        result: SkillCastResult::Started,
    };

    let Ok(skills) = skills.get(entity) else {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::MissingSkills);
        log.push(record);
        return;
    };

    // 检查是否已死亡
    if q_death.get(entity).is_ok() {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::CasterDead);
        log.push(record);
        return;
    }

    // 检查是否处于施法阻塞状态
    if q_cast_block.get(entity).is_ok() {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::Blocked);
        log.push(record);
        return;
    }

    let Some(&skill_entity_id) = skills.get(trigger.index) else {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::InvalidSkillIndex);
        log.push(record);
        return;
    };
    record.skill_entity = Some(skill_entity_id);

    let Ok((skill, mut cooldown_state, recast_window)) = q_skill.get_mut(skill_entity_id) else {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::MissingSkillEntity);
        log.push(record);
        return;
    };
    record.slot = Some(skill.slot);

    // Skip cooldown check if there's an active recast window (e.g., Riven Q stages)
    let can_cast_despite_cooldown = recast_window
        .map(|w| !w.timer.is_finished())
        .unwrap_or(false);

    if !can_cast_despite_cooldown {
        if let Some(ref timer) = cooldown_state.timer {
            if !timer.is_finished() {
                record.result = SkillCastResult::Failed(SkillCastFailureReason::CoolingDown);
                log.push(record);
                return;
            }
        }
    }

    let Some(spell_object) = res_assets_spell_object.get(&skill.spell) else {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::MissingSpellObject);
        log.push(record);
        return;
    };

    if skill.level == 0 {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::NotLearned);
        log.push(record);
        return;
    }

    let Ok(mut ability_resource) = q_ability_resource.get_mut(entity) else {
        record.result = SkillCastResult::Failed(SkillCastFailureReason::MissingAbilityResource);
        log.push(record);
        return;
    };

    if let Some(ref mana) = spell_object.spell_data.as_ref().unwrap().mana {
        let &current_mana = mana.get(skill.level as usize).unwrap();

        if ability_resource.value < current_mana {
            record.result =
                SkillCastResult::Failed(SkillCastFailureReason::InsufficientAbilityResource);
            log.push(record);
            return;
        }

        ability_resource.value -= current_mana;
    }

    record.result = SkillCastResult::Started;
    log.push(record);

    let cast_event = EventSkillCast {
        entity,
        skill_entity: skill_entity_id,
        index: trigger.index,
        point: trigger.point,
    };

    commands.trigger(cast_event);

    // 通过连招窗口释放的技能不重置冷却（如锐雯 Q 从第一段开始计时）
    if skill.cooldown_mode == SkillCooldownMode::AfterCast && !can_cast_despite_cooldown {
        cooldown_state.timer = Some(Timer::from_seconds(
            cooldown_state.duration,
            TimerMode::Once,
        ));
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
