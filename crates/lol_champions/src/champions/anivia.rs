use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::buffs::anivia_buffs::BuffAniviaR;

const ANIVIA_Q_KEY: &str = "Characters/Anivia/Spells/AniviaFlashFrost/AniviaFlashFrost";
#[allow(dead_code)]
const ANIVIA_W_KEY: &str = "Characters/Anivia/Spells/AniviaCrystallize/AniviaCrystallize";
const ANIVIA_E_KEY: &str = "Characters/Anivia/Spells/AniviaFrostbite/AniviaFrostbite";
const ANIVIA_R_KEY: &str = "Characters/Anivia/Spells/AniviaGlacialStorm/AniviaGlacialStorm";
const ANIVIA_Q_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginAnivia;

impl Plugin for PluginAnivia {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_anivia_skill_cast);
        app.add_observer(on_anivia_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Anivia"))]
#[reflect(Component)]
pub struct Anivia;

fn on_anivia_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
    _q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_anivia.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_anivia_q(
            &mut commands,
            entity,
            trigger.skill_entity,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_anivia_w(&mut commands, entity),
        SkillSlot::E => cast_anivia_e(&mut commands, entity),
        SkillSlot::R => cast_anivia_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_anivia_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell1"));

    if stage == 1 {
        // First cast: launch the crystal
        spawn_skill_particle(commands, entity, hash_bin("Anivia_Q_Cast"));
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, ANIVIA_Q_RECAST_WINDOW));
    } else {
        // Second cast: detonate for extra damage and stun
        spawn_skill_particle(commands, entity, hash_bin("Anivia_Q_Explode"));
        skill_damage(
            commands,
            entity,
            ANIVIA_Q_KEY,
            DamageShape::Circle { radius: 150.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            Some(hash_bin("Anivia_Q_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    }
}

fn cast_anivia_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Anivia_W_Cast"));
    // W creates a wall that blocks movement
}

fn cast_anivia_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Anivia_E_Cast"));

    // E deals extra damage to frozen targets
    skill_damage(
        commands,
        entity,
        ANIVIA_E_KEY,
        DamageShape::Circle { radius: 600.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Anivia_E_Hit")),
    );
}

fn cast_anivia_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Anivia_R_Cast"));

    // R is a continuous storm
    skill_damage(
        commands,
        entity,
        ANIVIA_R_KEY,
        DamageShape::Circle { radius: 750.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Anivia_R_Hit")),
    );

    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAniviaR::new());
}

fn on_anivia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_anivia: Query<(), With<Anivia>>,
) {
    let source = trigger.source;
    if q_anivia.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Q and R slow
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.2, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_anivia: Query<Entity, (With<Anivia>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_anivia.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Anivia/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Anivia/Spells/AniviaPassive/AniviaPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // Q uses manual cooldown mode for recast window
            if index == 0 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
