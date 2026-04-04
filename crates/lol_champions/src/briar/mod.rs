pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::briar::buffs::{BuffBriarPassive, BuffBriarQ, BuffBriarW};

const BRIAR_Q_KEY: &str = "Characters/Briar/Spells/BriarQ/BriarQ";
const BRIAR_W_KEY: &str = "Characters/Briar/Spells/BriarW/BriarW";
const BRIAR_E_KEY: &str = "Characters/Briar/Spells/BriarE/BriarE";
const BRIAR_R_KEY: &str = "Characters/Briar/Spells/BriarR/BriarR";

#[derive(Default)]
pub struct PluginBriar;

impl Plugin for PluginBriar {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_briar_skill_cast);
        app.add_observer(on_briar_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Briar"))]
#[reflect(Component)]
pub struct Briar;

fn on_briar_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_briar.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_briar_q(&mut commands, entity),
        SkillSlot::W => cast_briar_w(&mut commands, entity),
        SkillSlot::E => cast_briar_e(&mut commands, entity),
        SkillSlot::R => cast_briar_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_briar_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Briar_Q_Cast"));

    skill_damage(
        commands,
        entity,
        BRIAR_Q_KEY,
        DamageShape::Circle { radius: 475.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Briar_Q_Hit")),
    );
}

fn cast_briar_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Briar_W_Cast"));
}

fn cast_briar_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Briar_E_Cast"));

    skill_damage(
        commands,
        entity,
        BRIAR_E_KEY,
        DamageShape::Sector {
            radius: 500.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Briar_E_Hit")),
    );
}

fn cast_briar_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Briar_R_Cast"));

    skill_damage(
        commands,
        entity,
        BRIAR_R_KEY,
        DamageShape::Nearest {
            max_distance: 12000.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Briar_R_Hit")),
    );
}

fn on_briar_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_briar: Query<(), With<Briar>>,
) {
    let source = trigger.source;
    if q_briar.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarPassive::new(1, 10.0, 6.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarQ::new(0.85, 15.0, 2.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBriarW::new(0.75, 0.4, 4.0));
}

fn add_skills(
    mut commands: Commands,
    q_briar: Query<Entity, (With<Briar>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_briar.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Briar/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Briar/Spells/BriarPassive/BriarPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
