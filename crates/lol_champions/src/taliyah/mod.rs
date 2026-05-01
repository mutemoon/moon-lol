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

use crate::taliyah::buffs::BuffTaliyahW;

#[derive(Default)]
pub struct PluginTaliyah;

impl Plugin for PluginTaliyah {
    fn build(&self, app: &mut App) {
        app.add_observer(on_taliyah_skill_cast);
        app.add_observer(on_taliyah_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Taliyah"))]
#[reflect(Component)]
pub struct Taliyah;

fn on_taliyah_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_taliyah.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_taliyah_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_taliyah_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_taliyah_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_taliyah_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_taliyah_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_Q_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Taliyah_Q_Hit")),
    );
}

fn cast_taliyah_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_W_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Taliyah_W_Hit")),
    );
}

fn cast_taliyah_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_E_Cast"));

    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 800.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Taliyah_E_Hit")),
    );
}

fn cast_taliyah_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_R_Cast"));
}

fn on_taliyah_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_taliyah: Query<(), With<Taliyah>>,
) {
    let source = trigger.source;
    if q_taliyah.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffTaliyahW::new(0.75, 1.0));
}
