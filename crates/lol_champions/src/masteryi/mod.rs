pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::masteryi::buffs::{BuffMasterYiE, BuffMasterYiR};

const MASTERYI_Q_KEY: &str = "Characters/MasterYi/Spells/MasterYiQ/MasterYiQ";
#[allow(dead_code)]
const MASTERYI_W_KEY: &str = "Characters/MasterYi/Spells/MasterYiW/MasterYiW";
#[allow(dead_code)]
const MASTERYI_E_KEY: &str = "Characters/MasterYi/Spells/MasterYiE/MasterYiE";
#[allow(dead_code)]
const MASTERYI_R_KEY: &str = "Characters/MasterYi/Spells/MasterYiR/MasterYiR";

#[derive(Default)]
pub struct PluginMasterYi;

impl Plugin for PluginMasterYi {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_masteryi_skill_cast);
        app.add_observer(on_masteryi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MasterYi"))]
#[reflect(Component)]
pub struct MasterYi;

fn on_masteryi_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_masteryi_q(&mut commands, entity),
        SkillSlot::W => cast_masteryi_w(&mut commands, entity),
        SkillSlot::E => cast_masteryi_e(&mut commands, entity),
        SkillSlot::R => cast_masteryi_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_masteryi_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("MasterYi_Q_Cast"));

    // Q is a dash that damages multiple targets
    skill_damage(
        commands,
        entity,
        MASTERYI_Q_KEY,
        DamageShape::Circle { radius: 400.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("MasterYi_Q_Hit")),
    );
}

fn cast_masteryi_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("MasterYi_W_Cast"));

    // W is meditate (heal and damage reduction)
}

fn cast_masteryi_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("MasterYi_E_Cast"));

    // E grants bonus true damage on attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiE::new(0.3, 5.0));
}

fn cast_masteryi_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("MasterYi_R_Cast"));

    // R grants attackspeed and movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiR::new(0.8, 0.45, 10.0));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.45, 10.0));
}

fn on_masteryi_damage_hit(
    trigger: On<EventDamageCreate>,
    _commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
) {
    let source = trigger.source;
    if q_masteryi.get(source).is_err() {
        return;
    }

    // Passive: Double Strike on 4th attack
}

fn add_skills(
    mut commands: Commands,
    q_masteryi: Query<Entity, (With<MasterYi>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_masteryi.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/MasterYi/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/MasterYi/Spells/MasterYiPassive/MasterYiPassive",
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
