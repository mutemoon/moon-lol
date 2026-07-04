pub mod e;
pub mod passive;
pub mod q;
pub mod r;

#[cfg(test)]
mod passive_tests;
#[cfg(test)]
mod q_tests;
#[cfg(test)]
mod tests;

use bevy::prelude::*;
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_core::attack::{BuffAttack, CommandAttackReset};
use lol_core::base::buff::BuffOf;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::fiora::e::BuffFioraE;
use crate::fiora::r::BuffFioraR;

#[derive(Default)]
pub struct PluginFiora;

impl Plugin for PluginFiora {
    fn build(&self, app: &mut App) {
        app.init_resource::<passive::FioraVitalLastDirection>();
        app.add_systems(
            FixedUpdate,
            (
                passive::attach_fiora_passive_ability,
                passive::update_add_vital,
                passive::update_remove_vital,
                r::fixed_update,
                passive::update_vital_visuals,
            ),
        );
        app.add_observer(on_fiora_skill_cast);
        app.add_observer(q::on_fiora_q_dash_end);
        app.add_observer(passive::on_passive_damage_create);
        app.add_observer(e::on_event_attack_end);
        app.add_observer(r::on_r_damage_create);
    }
}

#[derive(Component, Default, Reflect)]
#[require(Champion, Name = Name::new("Fiora"))]
#[reflect(Component)]
pub struct Fiora;

fn on_fiora_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_fiora: Query<(), With<Fiora>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_fiora.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => q::cast_fiora_q(
            &mut commands,
            entity,
            trigger.point,
            skill.spell.clone(),
            skill.level,
        ),
        SkillSlot::W => cast_fiora_w(&mut commands, entity),
        SkillSlot::E => cast_fiora_e(&mut commands, entity),
        SkillSlot::R => cast_fiora_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_fiora_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2_in".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });
}

fn cast_fiora_e(commands: &mut Commands, entity: Entity) {
    commands.entity(entity).insert((BuffAttack {
        bonus_attack_speed: 0.5,
    },));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraE::default());
    commands.trigger(CommandAttackReset { entity });
}

fn cast_fiora_r(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffFioraR::default());
}
