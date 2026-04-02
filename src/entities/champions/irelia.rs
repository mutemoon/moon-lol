use bevy::prelude::*;
use league_core::CharacterRecord;
use league_utils::hash_bin;
use lol_config::LoadHashKeyTrait;

use crate::core::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, DamageShape, EventSkillCast, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills, TargetDamage, TargetFilter,
};
use crate::entities::champion::Champion;
use crate::PassiveSkillOf;
use crate::DamageType;

const IRELIA_Q_KEY: &str = "Characters/Irelia/Spells/IreliaQ/IreliaQ";
const IRELIA_E_KEY: &str = "Characters/Irelia/Spells/IreliaE/IreliaE";
const IRELIA_R_KEY: &str = "Characters/Irelia/Spells/IreliaR/IreliaR";
const IRELIA_E_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginIrelia;

impl Plugin for PluginIrelia {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_irelia_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Irelia"))]
#[reflect(Component)]
pub struct Irelia;

fn on_irelia_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<(), With<Irelia>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_irelia.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_irelia_q(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::W => cast_irelia_w(&mut commands, entity),
        SkillSlot::E => cast_irelia_e(
            &mut commands,
            entity,
            trigger.skill_entity,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_irelia_r(&mut commands, entity, trigger.point),
        _ => {}
    }
}

fn cast_irelia_q(commands: &mut Commands, q_transform: &Query<&Transform>, entity: Entity, point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Irelia_Q_Cast"));
    // Q is a dash that resets on kill and marks enemies as Unsteady
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: IRELIA_Q_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 250.0 },
            damage: Some(crate::DashDamage {
                radius_end: 80.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );
    // FUTURE: Reset on kill/unsteady mark, mark enemies as Unsteady
}

fn cast_irelia_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Irelia_W_Cast"));
    // W is a channel that grants damage reduction then releases damage
    // FUTURE: Add channeled defense buff, then release damage on release
}

fn cast_irelia_e(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Throws a blade forward
        spawn_skill_particle(commands, entity, hash_bin("Irelia_E_Cast"));
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, IRELIA_E_RECAST_WINDOW));
        // FUTURE: Create blade projectile that marks enemies as Unsteady
    } else {
        // Second cast: Throws second blade and stuns marked enemies
        spawn_skill_particle(commands, entity, hash_bin("Irelia_E2_Cast"));
        skill_damage(
            commands,
            entity,
            IRELIA_E_KEY,
            DamageShape::Circle { radius: 200.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Magic,
            }],
            Some(hash_bin("Irelia_E_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Irelia E", stage);
    }
}

fn cast_irelia_r(commands: &mut Commands, entity: Entity, _point: Vec2) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Irelia_R_Cast"));
    // R is a long-range blade wave that creates a zone and marks enemies
    skill_damage(
        commands,
        entity,
        IRELIA_R_KEY,
        DamageShape::Circle { radius: 350.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Irelia_R_Hit")),
    );
    // FUTURE: Create zone that marks enemies as Unsteady, reduce Q cooldown
}

fn add_skills(
    mut commands: Commands,
    q_irelia: Query<Entity, (With<Irelia>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_irelia.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Irelia/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Irelia/Spells/IreliaPassiveAbility/IreliaPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // E uses manual cooldown mode for recast window
            if index == 2 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
