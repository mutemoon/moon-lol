pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle,
};

use crate::taliyah::buffs::BuffTaliyahW;

const TALIYAH_Q_KEY: &str = "Characters/Taliyah/Spells/TaliyahQ/TaliyahQ";
const TALIYAH_W_KEY: &str = "Characters/Taliyah/Spells/TaliyahW/TaliyahW";
const TALIYAH_E_KEY: &str = "Characters/Taliyah/Spells/TaliyahE/TaliyahE";
#[allow(dead_code)]
const TALIYAH_R_KEY: &str = "Characters/Taliyah/Spells/TaliyahR/TaliyahR";

#[derive(Default)]
pub struct PluginTaliyah;

impl Plugin for PluginTaliyah {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
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

    match skill.slot {
        SkillSlot::Q => cast_taliyah_q(&mut commands, entity),
        SkillSlot::W => cast_taliyah_w(&mut commands, entity),
        SkillSlot::E => cast_taliyah_e(&mut commands, entity),
        SkillSlot::R => cast_taliyah_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_taliyah_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_Q_Cast"));

    skill_damage(
        commands,
        entity,
        TALIYAH_Q_KEY,
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

fn cast_taliyah_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_W_Cast"));

    skill_damage(
        commands,
        entity,
        TALIYAH_W_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Taliyah_W_Hit")),
    );
}

fn cast_taliyah_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Taliyah_E_Cast"));

    skill_damage(
        commands,
        entity,
        TALIYAH_E_KEY,
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
    play_skill_animation(commands, entity, hash_bin("Spell4"));
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

fn add_skills(
    mut commands: Commands,
    q_taliyah: Query<Entity, (With<Taliyah>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_taliyah.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Taliyah/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Taliyah/Spells/TaliyahPassive/TaliyahPassive",
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
