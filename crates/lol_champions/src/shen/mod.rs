pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::shen::buffs::BuffShenW;

#[derive(Default)]
pub struct PluginShen;

impl Plugin for PluginShen {
    fn build(&self, app: &mut App) {
        app.add_observer(on_shen_skill_cast);
        app.add_observer(on_shen_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Shen"))]
#[reflect(Component)]
pub struct Shen;

fn on_shen_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_shen.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_shen_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_shen_w(&mut commands, entity),
        SkillSlot::E => cast_shen_e(&mut commands, entity),
        SkillSlot::R => cast_shen_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_shen_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Shen_Q_Cast"));

    // Q is shadow dash - damage
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 600.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Shen_Q_Hit")),
    );
}

fn cast_shen_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Shen_W_Cast"));

    // W is spirits refuge - dodge
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}

fn cast_shen_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Shen_E_Cast"));

    // E is leap - dash
}

fn cast_shen_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Shen_R_Cast"));

    // R is stand united - global shield
}

fn on_shen_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_shen: Query<(), With<Shen>>,
) {
    let source = trigger.source;
    if q_shen.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q marks with spirit
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffShenW::new(1.0, 1.5));
}
