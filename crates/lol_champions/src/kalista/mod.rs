pub mod buffs;

use bevy::prelude::*;
use lol_base::animation_names::{ANIM_SPELL1, ANIM_SPELL2, ANIM_SPELL3, ANIM_SPELL4};
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::kalista::buffs::{BuffKalistaE, BuffKalistaR};

#[derive(Default)]
pub struct PluginKalista;

impl Plugin for PluginKalista {
    fn build(&self, app: &mut App) {
        app.add_observer(on_kalista_skill_cast);
        app.add_observer(on_kalista_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Kalista"))]
#[reflect(Component)]
pub struct Kalista;

fn on_kalista_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_kalista: Query<(), With<Kalista>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_kalista.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_kalista_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_kalista_w(&mut commands, entity),
        SkillSlot::E => cast_kalista_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_kalista_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_kalista_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL1.to_string(),
        repeat: false,
        duration: None,
    });

    // Q is a spear that passes through enemies
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 1200.0,
                angle: 10.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_kalista_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    // W is a sentinel that provides vision and deals damage on basic attacks
}

fn cast_kalista_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    // E deals damage to speared enemies and slows
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 1100.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
        }],
    });
}

fn cast_kalista_r(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL4.to_string(),
        repeat: false,
        duration: None,
    });

    // R pulls oathsworn ally and grants invulnerability
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffKalistaR::new(4.0));

    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 1200.0 },
        damage: None,
        speed: 1000.0,
    });
}

fn on_kalista_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_kalista: Query<(), With<Kalista>>,
) {
    let source = trigger.source;
    if q_kalista.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E applies slow
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffKalistaE::new(0.3, 2.0));
}
