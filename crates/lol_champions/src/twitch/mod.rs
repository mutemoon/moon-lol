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

use crate::twitch::buffs::{BuffTwitchPassive, BuffTwitchW};

#[derive(Default)]
pub struct PluginTwitch;

impl Plugin for PluginTwitch {
    fn build(&self, app: &mut App) {
        app.add_observer(on_twitch_skill_cast);
        app.add_observer(on_twitch_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Twitch"))]
#[reflect(Component)]
pub struct Twitch;

fn on_twitch_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_twitch.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_twitch_q(&mut commands, entity),
        SkillSlot::W => cast_twitch_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_twitch_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_twitch_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_twitch_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_Q_Cast"));
}

fn cast_twitch_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 955.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Twitch_W_Hit")),
    );
}

fn cast_twitch_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 1200.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Twitch_E_Hit")),
    );
}

fn cast_twitch_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_R_Cast"));
}

fn on_twitch_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_twitch: Query<(), With<Twitch>>,
) {
    let source = trigger.source;
    if q_twitch.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwitchPassive::new(1, 2.0, 6.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTwitchW::new(0.35, 3.0));
}
