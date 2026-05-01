use bevy::prelude::*;
use league_utils::hash_bin;
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    EventSkillCast, Skill, SkillSlot, play_skill_animation, spawn_skill_particle,
};

#[derive(Default)]
pub struct PluginHwei;

impl Plugin for PluginHwei {
    fn build(&self, app: &mut App) {
        app.add_observer(on_hwei_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Hwei"))]
#[reflect(Component)]
pub struct Hwei;

fn on_hwei_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_hwei: Query<(), With<Hwei>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_hwei.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    play_skill_animation(&mut commands, entity, "spell1".to_string());

    match skill.slot {
        SkillSlot::Q => spawn_skill_particle(&mut commands, entity, hash_bin("Hwei_Q_Q_Tar")),
        SkillSlot::W => spawn_skill_particle(&mut commands, entity, hash_bin("Hwei_Q_W_AoE")),
        SkillSlot::E => spawn_skill_particle(&mut commands, entity, hash_bin("Hwei_Q_Q_Tar")),
        SkillSlot::R => spawn_skill_particle(&mut commands, entity, hash_bin("Hwei_Q_Q_Tar")),
        _ => {}
    }
}
