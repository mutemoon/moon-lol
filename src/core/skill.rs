use std::ops::Deref;

use bevy::prelude::*;
use bevy_behave::prelude::{BehaveCtx, BehavePlugin, BehaveTree, Tree};
use bevy_behave::Behave;
use league_core::SpellObject;
use lol_config::{HashKey, LoadHashKeyTrait};

use crate::{AbilityResource, EventLevelUp, Level};

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.init_asset::<SkillEffect>();

        app.add_plugins(BehavePlugin::default());

        app.add_observer(on_skill_cast);
        app.add_observer(on_skill_level_up);
        app.add_observer(on_level_up);
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Skills)]
pub struct SkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = SkillOf, linked_spawn)]
pub struct Skills(Vec<Entity>);

#[derive(Component, Debug)]
#[relationship(relationship_target = PassiveSkill)]
pub struct PassiveSkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = PassiveSkillOf, linked_spawn)]
pub struct PassiveSkill(Entity);

impl Deref for Skills {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Component, Default)]
pub struct CoolDown {
    pub timer: Timer,
    pub duration: f32,
}

#[derive(Component)]
#[require(CoolDown)]
pub struct Skill {
    pub key_spell_object: HashKey<SpellObject>,
    pub key_skill_effect: HashKey<SkillEffect>,
    pub level: usize,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            key_spell_object: 0.into(),
            key_skill_effect: 0.into(),
            level: 0,
        }
    }
}

#[derive(Asset, TypePath)]
pub struct SkillEffect(pub Tree<Behave>);

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
    res_assets_spell_object: Res<Assets<SpellObject>>,
    res_assets_skill_effect: Res<Assets<SkillEffect>>,
    mut q_skill: Query<(&Skill, &mut CoolDown)>,
    mut q_ability_resource: Query<&mut AbilityResource>,
) {
    let entity = trigger.event_target();
    let Ok(skills) = skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.0.get(trigger.index) else {
        return;
    };
    let Ok((skill, mut cooldown)) = q_skill.get_mut(skill_entity) else {
        return;
    };

    if !cooldown.timer.is_finished() {
        debug!(
            "{} 技能 {} 冷却中，剩余 {:.2}s",
            entity,
            trigger.index,
            cooldown.timer.remaining_secs()
        );
        return;
    }

    let spell_object = res_assets_spell_object
        .load_hash(skill.key_spell_object)
        .unwrap();

    if skill.level == 0 {
        debug!("{} 技能 {} 未学习，无法释放", entity, trigger.index);
        return;
    }

    let Ok(mut ability_resource) = q_ability_resource.get_mut(entity) else {
        return;
    };

    if let Some(ref mana) = spell_object.m_spell.as_ref().unwrap().mana {
        let &current_mana = mana.get(skill.level as usize).unwrap();

        if ability_resource.value < current_mana {
            debug!(
                "{} 技能 {} 蓝量不足，需要 {:.0}，当前 {:.0}",
                entity, trigger.index, current_mana, ability_resource.value
            );
            return;
        }

        ability_resource.value -= current_mana;
        debug!(
            "{} 技能 {} 消耗 {:.0} 蓝量，剩余 {:.0}",
            entity, trigger.index, current_mana, ability_resource.value
        );
    }

    if let Some(effect) = res_assets_skill_effect.load_hash(skill.key_skill_effect) {
        debug!("{} 技能 {} 开始执行行为树", entity, trigger.index);
        commands.entity(entity).with_child((
            BehaveTree::new(effect.0.clone()),
            SkillEffectContext {
                point: trigger.point,
            },
        ));
    }

    cooldown.timer = Timer::from_seconds(cooldown.duration, TimerMode::Once);
    debug!(
        "{} 技能 {} 开始冷却 {}s",
        entity, trigger.index, cooldown.duration
    );
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
