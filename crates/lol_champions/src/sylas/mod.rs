use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::BuffSelfHeal;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot};

const SYLAS_E_RECAST_WINDOW: f32 = 4.0;

#[derive(Default)]
pub struct PluginSylas;

impl Plugin for PluginSylas {
    fn build(&self, app: &mut App) {
        app.add_observer(on_sylas_skill_cast);
        app.add_observer(on_sylas_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Sylas"))]
#[reflect(Component)]
pub struct Sylas;

fn on_sylas_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_sylas.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_sylas_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_sylas_w(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.point,
        ),
        SkillSlot::E => cast_sylas_e(
            &mut commands,
            &q_transform,
            entity,
            skill_spell,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::R => cast_sylas_r(&mut commands, entity, skill_spell, trigger.point),
        _ => {}
    }
}

fn cast_sylas_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sylas_Q_Cast"),
    });
    // Q is a lash that slows enemies in the center
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Sector {
                radius: 350.0,
                angle: 60.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Sylas_Q_Hit")),
        }],
    });
}

fn cast_sylas_w(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    point: Vec2,
) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sylas_W_Cast"),
    });
    // W is a dash to target that deals damage and heals based on missing health
    commands.trigger(ActionDash {
        entity,
        point: point,
        skill: skill_spell,
        move_type: DashMoveType::Pointer { max: 200.0 },
        damage: Some(DashDamage {
            radius_end: 100.0,
            damage: TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            },
        }),
        speed: 900.0,
    });
    // Heal based on missing health
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSelfHeal::new(60.0));
}

fn cast_sylas_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    skill_spell: Handle<Spell>,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        // First cast: Throws chain toward enemy - damage in narrow cone
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: hash_bin("Sylas_E_Cast"),
        });
        commands.trigger(ActionDamage {
            entity,
            skill: skill_spell,
            effects: vec![ActionDamageEffect {
                shape: DamageShape::Sector {
                    radius: 400.0,
                    angle: 20.0,
                },
                damage_list: vec![TargetDamage {
                    filter: TargetFilter::All,
                    amount: "TotalDamage".to_string(),
                    damage_type: DamageType::Magic,
                }],
                particle: Some(hash_bin("Sylas_E_Hit")),
            }],
        });
        commands
            .entity(skill_entity)
            .insert(SkillRecastWindow::new(2, 2, SYLAS_E_RECAST_WINDOW));
    } else {
        // Second cast: Dash to enemy and pull
        commands.trigger(CommandSkinParticleSpawn {
            entity,
            hash: hash_bin("Sylas_E2_Cast"),
        });
        commands.trigger(ActionDash {
            entity,
            point: point,
            skill: skill_spell,
            move_type: DashMoveType::Pointer { max: 300.0 },
            damage: Some(DashDamage {
                radius_end: 100.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: "TotalDamage".to_string(),
                    damage_type: DamageType::Magic,
                },
            }),
            speed: 800.0,
        });
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        commands.entity(skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

fn cast_sylas_r(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>, _point: Vec2) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Sylas_R_Cast"),
    });
    // R 对最近敌方英雄造成魔法伤害
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 400.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::Champion,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::Magic,
            }],
            particle: Some(hash_bin("Sylas_R_Hit")),
        }],
    });
}

/// 监听 Sylas 造成的伤害，Q 命中减速
fn on_sylas_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_sylas: Query<(), With<Sylas>>,
) {
    let source = trigger.source;
    if q_sylas.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // Q 命中减速
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.5, 1.5));
}
