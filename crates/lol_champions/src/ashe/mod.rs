pub mod buffs;

use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillSlot, play_skill_animation, skill_damage,
    spawn_skill_particle,
};

use crate::ashe::buffs::BuffAsheQ;

#[derive(Default)]
pub struct PluginAshe;

impl Plugin for PluginAshe {
    fn build(&self, app: &mut App) {
        app.add_observer(on_ashe_skill_cast);
        app.add_observer(on_ashe_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ashe"))]
#[reflect(Component)]
pub struct Ashe;

fn on_ashe_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_ashe.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_ashe_q(&mut commands, entity),
        SkillSlot::W => cast_ashe_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_ashe_e(&mut commands, entity),
        SkillSlot::R => cast_ashe_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_ashe_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Ashe_Q_Cast"));

    // Q grants attack speed buff and fires multiple arrows
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAsheQ::new());
}

fn cast_ashe_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Ashe_W_Cast"));

    // W is a cone volley
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1200.0,
            angle: 40.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Ashe_W_Hit")),
    );
}

fn cast_ashe_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Ashe_E_Cast"));
    // E is global vision - no damage
}

fn cast_ashe_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Ashe_R_Cast"));

    // R is a global arrow that stuns - use large sector to simulate global range
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 20000.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ashe_R_Hit")),
    );
}

fn on_ashe_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ashe: Query<(), With<Ashe>>,
) {
    let source = trigger.source;
    if q_ashe.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply frost slow on all damage
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}
