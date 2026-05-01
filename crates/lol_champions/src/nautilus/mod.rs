pub mod buffs;

use bevy::prelude::{Handle, *};
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

use crate::nautilus::buffs::{BuffNautilusE, BuffNautilusW};

#[derive(Default)]
pub struct PluginNautilus;

impl Plugin for PluginNautilus {
    fn build(&self, app: &mut App) {
        app.add_observer(on_nautilus_skill_cast);
        app.add_observer(on_nautilus_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Nautilus"))]
#[reflect(Component)]
pub struct Nautilus;

fn on_nautilus_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_nautilus.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_nautilus_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_nautilus_w(&mut commands, entity),
        SkillSlot::E => cast_nautilus_e(&mut commands, entity, skill_spell),
        SkillSlot::R => cast_nautilus_r(&mut commands, entity, skill_spell),
        _ => {}
    }
}

fn cast_nautilus_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell1".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_Q_Cast"));

    // Q is a hook that drags
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 1122.0,
            angle: 10.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nautilus_Q_Hit")),
    );
}

fn cast_nautilus_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, "spell2".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_W_Cast"));

    // W is a shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffNautilusW::new(100.0, 6.0));
}

fn cast_nautilus_e(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell3".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_E_Cast"));

    // E is a three-hit wave that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nautilus_E_Hit")),
    );
}

fn cast_nautilus_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, "spell4".to_string());
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_R_Cast"));

    // R is a targeted knockup
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Nearest {
            max_distance: 825.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nautilus_R_Hit")),
    );
}

fn on_nautilus_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_nautilus: Query<(), With<Nautilus>>,
) {
    let source = trigger.source;
    if q_nautilus.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // E slows
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffNautilusE::new(0.4, 1.5));
}
