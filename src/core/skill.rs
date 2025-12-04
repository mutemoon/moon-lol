use bevy::prelude::*;
use bevy_behave::{
    prelude::{BehaveCtx, BehavePlugin, BehaveTree, Tree},
    Behave,
};

use crate::{EventLevelUp, Level};

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.add_plugins(BehavePlugin::default());

        app.add_observer(on_skill_cast);
        app.add_observer(on_skill_level_up);
        app.add_observer(on_level_up);
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Skills)]
pub struct SkillOf(pub Entity);

use std::ops::Deref;

#[derive(Component, Debug)]
#[relationship_target(relationship = SkillOf, linked_spawn)]
pub struct Skills(Vec<Entity>);

impl Deref for Skills {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Default)]
pub struct CoolDown {
    pub timer: Timer,
}

#[derive(Component)]
#[require(CoolDown)]
pub struct Skill {
    pub key: u32,
    pub effect: Option<Tree<Behave>>,
    pub level: u32,
}

#[derive(Component)]
pub struct SkillPoints(pub u32);

impl Default for SkillPoints {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component)]
pub struct SkillEffectContext {
    pub point: Vec2,
}

#[derive(Component)]
pub struct SkillEffectBehaveCtx(pub BehaveCtx);

#[derive(EntityEvent)]
pub struct CommandSkillStart {
    pub entity: Entity,
    pub index: usize,
    pub point: Vec2,
}

fn on_skill_cast(
    trigger: On<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    let skills = skills.get(entity).unwrap();
    let skill_entity = skills.0[trigger.index];
    let skill = q_skill.get(skill_entity).unwrap();

    if let Some(effect) = &skill.effect {
        commands.entity(entity).with_child((
            BehaveTree::new(effect.clone()),
            SkillEffectContext {
                point: trigger.point,
            },
        ));
    }
}

#[derive(EntityEvent)]
pub struct CommandSkillLevelUp {
    pub entity: Entity,
    pub index: usize,
}

fn on_skill_level_up(
    trigger: On<CommandSkillLevelUp>,
    skills: Query<&Skills>,
    mut q_skill: Query<&mut Skill>,
    mut q_skill_points: Query<(&Level, &mut SkillPoints)>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.0.get(trigger.index) else {
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

fn on_level_up(event: On<EventLevelUp>, mut q_skill_points: Query<&mut SkillPoints>) {
    let entity = event.event_target();
    if let Ok(mut skill_points) = q_skill_points.get_mut(entity) {
        skill_points.0 += event.delta;
        debug!(
            "{} 升级: 获得 {} 技能点，当前技能点 {}",
            entity, event.delta, skill_points.0
        );
    }
}
