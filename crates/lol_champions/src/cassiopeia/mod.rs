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
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

use crate::cassiopeia::buffs::BuffCassioPoison;

const CASSIO_Q_KEY: &str = "Characters/Cassiopeia/Spells/CassiopeiaQ/CassiopeiaQ";
const CASSIO_W_KEY: &str = "Characters/Cassiopeia/Spells/CassiopeiaW/CassiopeiaW";
const CASSIO_E_KEY: &str = "Characters/Cassiopeia/Spells/CassiopeiaE/CassiopeiaE";
const CASSIO_R_KEY: &str = "Characters/Cassiopeia/Spells/CassiopeiaR/CassiopeiaR";

#[derive(Default)]
pub struct PluginCassiopeia;

impl Plugin for PluginCassiopeia {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_cassiopeia_skill_cast);
        app.add_observer(on_cassiopeia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Cassiopeia"))]
#[reflect(Component)]
pub struct Cassiopeia;

fn on_cassiopeia_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_cassio: Query<(), With<Cassiopeia>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_cassio.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_cassio_q(&mut commands, entity),
        SkillSlot::W => cast_cassio_w(&mut commands, entity),
        SkillSlot::E => cast_cassio_e(&mut commands, entity),
        SkillSlot::R => cast_cassio_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_cassio_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Cassio_Q_Cast"));

    // Q is ground targeted area
    skill_damage(
        commands,
        entity,
        CASSIO_Q_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Cassio_Q_Hit")),
    );
}

fn cast_cassio_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Cassio_W_Cast"));

    // W is a poison cloud
    skill_damage(
        commands,
        entity,
        CASSIO_W_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Cassio_W_Hit")),
    );
}

fn cast_cassio_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Cassio_E_Cast"));

    // E is targeted damage to poisoned enemies
    skill_damage(
        commands,
        entity,
        CASSIO_E_KEY,
        DamageShape::Circle { radius: 700.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Cassio_E_Hit")),
    );
}

fn cast_cassio_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Cassio_R_Cast"));

    // R is a cone that stuns facing enemies
    skill_damage(
        commands,
        entity,
        CASSIO_R_KEY,
        DamageShape::Sector {
            radius: 850.0,
            angle: 80.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Cassio_R_Hit")),
    );
}

fn on_cassiopeia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_cassio: Query<(), With<Cassiopeia>>,
) {
    let source = trigger.source;
    if q_cassio.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply poison
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCassioPoison::new());
}

fn add_skills(
    mut commands: Commands,
    q_cassio: Query<Entity, (With<Cassiopeia>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_cassio.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Cassiopeia/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Cassiopeia/Spells/CassiopeiaPassive/CassiopeiaPassive",
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
