use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, BuffOf,
    CoolDown, DamageShape, EventDamageCreate, EventSkillCast, Skill, SkillOf, SkillSlot, Skills,
    TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::{BuffNautilusE, BuffNautilusW, DamageType, PassiveSkillOf};

const NAUTILUS_Q_KEY: &str = "Characters/Nautilus/Spells/NautilusQ/NautilusQ";
#[allow(dead_code)]
const NAUTILUS_W_KEY: &str = "Characters/Nautilus/Spells/NautilusW/NautilusW";
const NAUTILUS_E_KEY: &str = "Characters/Nautilus/Spells/NautilusE/NautilusE";
const NAUTILUS_R_KEY: &str = "Characters/Nautilus/Spells/NautilusR/NautilusR";

#[derive(Default)]
pub struct PluginNautilus;

impl Plugin for PluginNautilus {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
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

    match skill.slot {
        SkillSlot::Q => cast_nautilus_q(&mut commands, entity),
        SkillSlot::W => cast_nautilus_w(&mut commands, entity),
        SkillSlot::E => cast_nautilus_e(&mut commands, entity),
        SkillSlot::R => cast_nautilus_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_nautilus_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_Q_Cast"));

    // Q is a hook that drags
    skill_damage(
        commands,
        entity,
        NAUTILUS_Q_KEY,
        DamageShape::Sector { radius: 1122.0, angle: 10.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nautilus_Q_Hit")),
    );
}

fn cast_nautilus_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_W_Cast"));

    // W is a shield
    commands.entity(entity).with_related::<BuffOf>(BuffNautilusW::new(100.0, 6.0));
}

fn cast_nautilus_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_E_Cast"));

    // E is a three-hit wave that slows
    skill_damage(
        commands,
        entity,
        NAUTILUS_E_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Nautilus_E_Hit")),
    );
}

fn cast_nautilus_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Nautilus_R_Cast"));

    // R is a targeted knockup
    skill_damage(
        commands,
        entity,
        NAUTILUS_R_KEY,
        DamageShape::Nearest { max_distance: 825.0 },
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
    commands.entity(target).with_related::<BuffOf>(BuffNautilusE::new(0.4, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_nautilus: Query<Entity, (With<Nautilus>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_nautilus.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Nautilus/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Nautilus/Spells/NautilusPassive/NautilusPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
