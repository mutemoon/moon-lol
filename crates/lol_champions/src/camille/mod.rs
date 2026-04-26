use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::spell::Spell;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, CoolDownState, EventSkillCast, Skill, SkillRecastWindow, SkillSlot,
    play_skill_animation, reset_skill_attack, skill_damage, skill_dash, spawn_skill_particle,
};

const CAMILLE_Q_RECAST_WINDOW: f32 = 3.0;
const CAMILLE_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginCamille;

impl Plugin for PluginCamille {
    fn build(&self, app: &mut App) {
        app.add_observer(on_camille_skill_cast);
        app.add_observer(on_camille_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Camille"))]
#[reflect(Component)]
pub struct Camille;

fn on_camille_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_camille_q(
            &mut commands,
            entity,
            trigger.skill_entity,
            cooldown,
            recast,
            skill_spell,
        ),
        SkillSlot::W => cast_camille_w(&mut commands, entity, skill_spell),
        SkillSlot::E => cast_camille_e(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
            skill_spell,
        ),
        SkillSlot::R => cast_camille_r(
            &mut commands,
            &q_transform,
            entity,
            trigger.point,
            skill_spell,
        ),
        _ => {}
    }
}

fn cast_camille_q(
    commands: &mut Commands,
    entity: Entity,
    skill_entity: Entity,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell1"));

    if stage == 1 {
        // First cast: Prepares the hookshot
        spawn_skill_particle(commands, entity, hash_bin("Camille_Q_Cast"));
        // Q1 doesn't deal damage, just marks for second cast
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_Q_RECAST_WINDOW));
    } else {
        // Second cast: Deals bonus damage and resets attack
        spawn_skill_particle(commands, entity, hash_bin("Camille_Q2_Cast"));
        reset_skill_attack(commands, entity);
        skill_damage(
            commands,
            entity,
            skill_spell,
            DamageShape::Nearest {
                max_distance: 150.0,
            },
            vec![TargetDamage {
                filter: TargetFilter::All,
                amount: hash_bin("TotalDamage"),
                damage_type: DamageType::Physical,
            }],
            Some(hash_bin("Camille_Q2_Hit")),
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((
            CoolDown {
                duration: cooldown.duration,
            },
            CoolDownState {
                timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            },
        ));
    }
}

fn cast_camille_w(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Camille_W_Cast"));
    // W is a swept cone that slows
    skill_damage(
        commands,
        entity,
        skill_spell,
        DamageShape::Sector {
            radius: 300.0,
            angle: 90.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Camille_W_Hit")),
    );
}

fn cast_camille_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
    skill_spell: Handle<Spell>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    play_skill_animation(commands, entity, hash_bin("Spell3"));

    if stage == 1 {
        // First cast: Hookshot - launches toward terrain
        spawn_skill_particle(commands, entity, hash_bin("Camille_E_Cast"));
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash toward hooked terrain
        spawn_skill_particle(commands, entity, hash_bin("Camille_E2_Cast"));
        skill_dash(
            commands,
            q_transform,
            entity,
            point,
            &ActionDash {
                skill: skill_spell,
                move_type: DashMoveType::Pointer { max: 400.0 },
                damage: Some(DashDamage {
                    radius_end: 150.0,
                    damage: TargetDamage {
                        filter: TargetFilter::All,
                        amount: hash_bin("TotalDamage"),
                        damage_type: DamageType::Physical,
                    },
                }),
                speed: 900.0,
            },
        );
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((
            CoolDown {
                duration: cooldown.duration,
            },
            CoolDownState {
                timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            },
        ));
    }
}

fn cast_camille_r(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    skill_spell: Handle<Spell>,
) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Camille_R_Cast"));
    // R is a hookshot-like leap that marks and traps target champion
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 350.0 },
            damage: Some(DashDamage {
                radius_end: 150.0,
                damage: TargetDamage {
                    filter: TargetFilter::Champion,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );
}

/// 监听 Camille 造成的伤害，给目标施加减速
fn on_camille_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
) {
    let source = trigger.source;
    if q_camille.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.6, 2.0));
}
