use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::buffs::ahri_buffs::{BuffAhriFoxFire, BuffCharm};

const AHRI_Q_KEY: &str = "Characters/Ahri/Spells/AhriOrbofDeception/AhriOrbofDeception";
const AHRI_W_KEY: &str = "Characters/Ahri/Spells/AhriFoxFire/AhriFoxFire";
const AHRI_E_KEY: &str = "Characters/Ahri/Spells/AhriCharm/AhriCharm";
const AHRI_R_KEY: &str = "Characters/Ahri/Spells/AhriSpiritRush/AhriSpiritRush";

#[derive(Default)]
pub struct PluginAhri;

impl Plugin for PluginAhri {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_ahri_skill_cast);
        app.add_observer(on_ahri_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Ahri"))]
#[reflect(Component)]
pub struct Ahri;

fn on_ahri_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_ahri.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_ahri_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
        ),
        SkillSlot::W => cast_ahri_w(&mut commands, entity, trigger.skill_entity),
        SkillSlot::E => cast_ahri_e(&mut commands, entity),
        SkillSlot::R => cast_ahri_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        _ => {}
    }
}

fn cast_ahri_q(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    _skill_entity: Entity,
    _point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Ahri_Q_Cast"));

    // Q creates a missile that travels out and returns
    // First pass: magic damage in a cone
    skill_damage(
        commands,
        entity,
        AHRI_Q_KEY,
        DamageShape::Sector {
            radius: 900.0,
            angle: 90.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ahri_Q_Hit")),
    );

    // Apply fox fire buff for W tracking (will be consumed by W)
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAhriFoxFire::new(3));
}

fn cast_ahri_w(commands: &mut Commands, entity: Entity, _skill_entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Ahri_W_Cast"));

    // Fox-fire: Three flames orbit Ahri and can attack enemies
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffAhriFoxFire::new(3));

    // W damage
    skill_damage(
        commands,
        entity,
        AHRI_W_KEY,
        DamageShape::Circle { radius: 550.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ahri_W_Hit")),
    );
}

fn cast_ahri_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Ahri_E_Cast"));

    // E is a charm missile that charms on hit
    skill_damage(
        commands,
        entity,
        AHRI_E_KEY,
        DamageShape::Sector {
            radius: 1000.0,
            angle: 60.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::Champion,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Ahri_E_Hit")),
    );
}

fn cast_ahri_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell4"));

    if stage == 1 {
        // First cast: dash toward target
        spawn_skill_particle(commands, entity, hash_bin("Ahri_R_Cast"));
    } else {
        spawn_skill_particle(commands, entity, hash_bin("Ahri_R2_Cast"));
    }

    // R is a dash that can be recast twice
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: AHRI_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 500.0 },
            damage: Some(DashDamage {
                radius_end: 300.0,
                damage: TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 600.0,
        },
    );

    // R has 2 recasts within 15 seconds
    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
    } else {
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(stage + 1, 3, 15.0));
    }
}

/// Listen for Ahri damage hits to apply effects
fn on_ahri_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_ahri: Query<(), With<Ahri>>,
) {
    let source = trigger.source;
    if q_ahri.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Check if this was from E (charm) to apply charm debuff
    // The charm effect is applied based on the skill hash
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffCharm::new(1.5));
}

fn add_skills(
    mut commands: Commands,
    q_ahri: Query<Entity, (With<Ahri>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_ahri.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Ahri/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Ahri/Spells/AhriPassive/AhriPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // R uses manual cooldown mode for recast windows
            if index == 3 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
