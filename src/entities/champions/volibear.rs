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
use crate::{BuffOf, BuffShieldWhite, DamageType, PassiveSkillOf};

const VOLIBEAR_W_KEY: &str = "Characters/Volibear/Spells/VolibearW/VolibearW";
const VOLIBEAR_E_KEY: &str = "Characters/Volibear/Spells/VolibearE/VolibearE";
const VOLIBEAR_R_KEY: &str = "Characters/Volibear/Spells/VolibearR/VolibearR";
const VOLIBEAR_W_RECAST_WINDOW: f32 = 2.0;

#[derive(Default)]
pub struct PluginVolibear;

impl Plugin for PluginVolibear {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_volibear_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Volibear"))]
#[reflect(Component)]
pub struct Volibear;

fn on_volibear_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_volibear.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_volibear_q(&mut commands, entity),
        SkillSlot::W => cast_volibear_w(
            &mut commands,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::E => cast_volibear_e(&mut commands, entity),
        SkillSlot::R => cast_volibear_r(&mut commands, &q_transform, entity, trigger.point),
        _ => {}
    }
}

fn cast_volibear_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Volibear_Q_Cast"));
    // Q is movement speed boost + stun on contact
    // TODO: Add movement speed buff with stun on collision
}

fn cast_volibear_w(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    _point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell2"));

    if stage == 1 {
        // First cast: W marks target
        spawn_skill_particle(commands, entity, hash_bin("Volibear_W_Cast"));
        commands.entity(skill_entity).insert(SkillRecastWindow::new(2, 2, VOLIBEAR_W_RECAST_WINDOW));
        // TODO: Add mark buff to target
        debug!("{:?} 释放了 {} 技能，当前阶段 {}", entity, "Volibear W", stage);
    } else {
        // Second cast: W detonates mark for bonus damage + heal
        spawn_skill_particle(commands, entity, hash_bin("Volibear_W2_Cast"));
        skill_damage(
            commands,
            entity,
            VOLIBEAR_W_KEY,
            DamageShape::Nearest { max_distance: 200.0 },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Volibear_W_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        // TODO: Heal based on missing health
        debug!("{:?} 释放了 {} 技能，当前阶段 {}，开始冷却", entity, "Volibear W", stage);
    }
}

fn cast_volibear_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Volibear_E_Cast"));
    // E is AoE damage + slow + shield
    skill_damage(
        commands,
        entity,
        VOLIBEAR_E_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Volibear_E_Hit")),
    );
    commands.entity(entity).with_related::<BuffOf>(BuffShieldWhite::new(100.0));
    // TODO: Add slow debuff to enemies
}

fn cast_volibear_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Volibear_R_Cast"));
    // R is a leap that deals damage and marks towers as vulnerable
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &crate::ActionDash {
            skill: VOLIBEAR_R_KEY.into(),
            move_type: crate::DashMoveType::Pointer { max: 400.0 },
            damage: Some(crate::DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );
    // TODO: Add tower debuff
}

fn add_skills(
    mut commands: Commands,
    q_volibear: Query<Entity, (With<Volibear>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_volibear.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Volibear/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Volibear/Spells/VolibearPassiveAbility/VolibearPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let mut skill_component = Skill::new(skill_slot_from_index(index), skill);
            // W uses manual cooldown mode for recast window
            if index == 1 {
                skill_component = skill_component.with_cooldown_mode(SkillCooldownMode::Manual);
            }
            commands.entity(entity).with_related::<SkillOf>((
                skill_component,
                CoolDown::default(),
            ));
        }
    }
}
