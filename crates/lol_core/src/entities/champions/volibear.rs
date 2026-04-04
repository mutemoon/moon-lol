use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::action::dash::{ActionDash, DashDamage, DashMoveType};
use crate::base::buff::BuffOf;
use crate::buffs::cc_debuffs::DebuffSlow;
use crate::buffs::common_buffs::BuffSelfHeal;
use crate::buffs::shield_white::BuffShieldWhite;
use crate::buffs::volibear_buffs::{BuffVolibearQ, DebuffVolibearWMark};
use crate::damage::{DamageType, EventDamageCreate};
use crate::entities::champion::Champion;
use crate::skill::{
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, skill_slot_from_index,
    spawn_skill_particle, CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode,
    SkillOf, SkillRecastWindow, SkillSlot, Skills,
};

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
        app.add_observer(on_volibear_damage_hit);
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
    reset_skill_attack(commands, entity);
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffVolibearQ::new(0.3, 1.0, 4.0));
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
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            2,
            2,
            VOLIBEAR_W_RECAST_WINDOW,
        ));
    } else {
        // Second cast: W detonates mark for bonus damage + heal
        spawn_skill_particle(commands, entity, hash_bin("Volibear_W2_Cast"));
        skill_damage(
            commands,
            entity,
            VOLIBEAR_W_KEY,
            DamageShape::Nearest {
                max_distance: 200.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Volibear_W_Hit")),
        );
        // W2 命中已标记目标时自我治疗
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffSelfHeal::new(50.0));
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
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
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(100.0));
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
        &ActionDash {
            skill: VOLIBEAR_R_KEY.into(),
            move_type: DashMoveType::Pointer { max: 400.0 },
            damage: Some(DashDamage {
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
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}

/// 监听 Volibear 造成的伤害，W1 标记目标，E 命中减速
fn on_volibear_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_volibear: Query<(), With<Volibear>>,
) {
    let source = trigger.source;
    if q_volibear.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // W1 标记目标
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffVolibearWMark::new(source, 4.0));
    // E 命中减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.4, 2.0));
}
