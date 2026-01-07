use bevy::prelude::*;
use lol_config::HashKey;

use crate::{Buff, Buffs, CommandSkillBeforeStart, Riven, Skill, SkillEffect, Skills};

#[derive(Default)]
pub struct PluginRivenQ;

impl Plugin for PluginRivenQ {
    fn build(&self, app: &mut App) {
        app.add_observer(on_command_skill_start);
    }
}

#[derive(Component, Debug, Clone, Default)]
#[require(Buff = Buff { name: "RivenQ2" })]
pub struct BuffRivenQ2;

#[derive(Component, Debug, Clone, Default)]
#[require(Buff = Buff { name: "RivenQ3" })]
pub struct BuffRivenQ3;

const RIVEN_Q1_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleave";
const RIVEN_Q2_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleaveQ2";
const RIVEN_Q3_KEY: &str = "Characters/Riven/Spells/RivenTriCleaveAbility/RivenTriCleaveQ3";

fn on_command_skill_start(
    event: On<CommandSkillBeforeStart>,
    mut commands: Commands,
    q_buffs: Query<&Buffs>,
    q_buff_q2: Query<&BuffRivenQ2>,
    q_buff_q3: Query<&BuffRivenQ3>,
    q_skills: Query<&Skills, With<Riven>>,
    mut q_skill: Query<&mut Skill>,
) {
    if event.index != 0 {
        return;
    }

    let entity = event.event_target();

    let Ok(skills) = q_skills.get(entity) else {
        return;
    };
    let Some(&skill_entity) = skills.get(0) else {
        return;
    };
    let Ok(mut skill) = q_skill.get_mut(skill_entity) else {
        return;
    };

    let (stage, buff_entity) = get_riven_q_stage(entity, &q_buffs, &q_buff_q2, &q_buff_q3);
    debug!("{:?} 锐雯 Q 技能第 {} 段", entity, stage);

    let effect_key: HashKey<SkillEffect> = match stage {
        1 => RIVEN_Q1_KEY.into(),
        2 => RIVEN_Q2_KEY.into(),
        _ => RIVEN_Q3_KEY.into(),
    };

    debug!("{:?} 更新 Q 技能效果为: {:?}", entity, effect_key);
    skill.key_skill_effect = effect_key;

    if let Some(buff) = buff_entity {
        debug!("{:?} 移除旧的 Q 技能 Buff: {:?}", entity, buff);
        commands.entity(buff).despawn();
    }
}

fn get_riven_q_stage(
    entity: Entity,
    q_buffs: &Query<&Buffs>,
    q_buff_q2: &Query<&BuffRivenQ2>,
    q_buff_q3: &Query<&BuffRivenQ3>,
) -> (u8, Option<Entity>) {
    let Ok(buffs) = q_buffs.get(entity) else {
        return (1, None);
    };

    for buff in buffs.iter() {
        if q_buff_q3.get(buff).is_ok() {
            return (3, Some(buff));
        }
        if q_buff_q2.get(buff).is_ok() {
            return (2, Some(buff));
        }
    }

    (1, None)
}
