use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::base::buff::BuffOf;
use crate::buffs::twitch_buffs::{BuffTwitchPassive, BuffTwitchW};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};

const TWITCH_Q_KEY: &str = "Characters/Twitch/Spells/TwitchQ/TwitchQ";
const TWITCH_W_KEY: &str = "Characters/Twitch/Spells/TwitchW/TwitchW";
const TWITCH_E_KEY: &str = "Characters/Twitch/Spells/TwitchE/TwitchE";
const TWITCH_R_KEY: &str = "Characters/Twitch/Spells/TwitchR/TwitchR";

#[derive(Default)]
pub struct PluginTwitch;

impl Plugin for PluginTwitch {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
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

    match skill.slot {
        SkillSlot::Q => cast_twitch_q(&mut commands, entity),
        SkillSlot::W => cast_twitch_w(&mut commands, entity),
        SkillSlot::E => cast_twitch_e(&mut commands, entity),
        SkillSlot::R => cast_twitch_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_twitch_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_Q_Cast"));
}

fn cast_twitch_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_W_Cast"));

    skill_damage(
        commands,
        entity,
        TWITCH_W_KEY,
        DamageShape::Circle { radius: 955.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Twitch_W_Hit")),
    );
}

fn cast_twitch_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Twitch_E_Cast"));

    skill_damage(
        commands,
        entity,
        TWITCH_E_KEY,
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

fn add_skills(
    mut commands: Commands,
    q_twitch: Query<Entity, (With<Twitch>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_twitch.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Twitch/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Twitch/Spells/TwitchPassive/TwitchPassive",
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
