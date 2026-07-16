//! Camille E（钩索 / Hookshot）完整实现。
//!
//! E1：朝向地形发射钩索，挂重施窗口。
//! E2：冲刺 + DashDamageIntent + 攻速加成。

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL3;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::action::dash::{ActionDash, DashDamage, DashDamageIntent, DashMoveType};
use lol_core::attack::BuffAttack;
use lol_core::base::buff::{Buff, BuffOf};
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_data_value};

use crate::camille::Camille;

const CAMILLE_E_RECAST_WINDOW: f32 = 1.0;

/// E 攻速加成持续时间（ron ASDuration = 5s）。
pub const CAMILLE_E_AS_DURATION: f32 = 5.0;

/// E 攻速加成计时器：到期回收 `BuffAttack`。
#[derive(Component, Debug, Clone)]
#[require(Buff = Buff { name: "CamilleE" })]
pub struct BuffCamilleE {
    pub timer: Timer,
}

impl BuffCamilleE {
    pub fn new(duration: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration, TimerMode::Once),
        }
    }
}

/// E2 施法：挂攻速加成 + 计时 buff。
pub fn apply_camille_e_as(commands: &mut Commands, entity: Entity, as_percent: f32, duration: f32) {
    commands.entity(entity).insert(BuffAttack {
        bonus_attack_speed: as_percent,
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffCamilleE::new(duration));
}

pub fn on_camille_e(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_camille: Query<(), With<Camille>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    if q_camille.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::E) {
        return;
    }
    let Some(spell_obj) = res_spells.get(&skill.spell) else {
        return;
    };
    let level = skill.level;
    let as_buff = get_skill_data_value(spell_obj, "ASBuff", level).unwrap_or(0.35);
    let as_duration = get_skill_data_value(spell_obj, "ASDuration", level).unwrap_or(5.0);

    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL3.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(2, 2, CAMILLE_E_RECAST_WINDOW));
    } else {
        commands.entity(entity).insert(DashDamageIntent {
            damage: DashDamage {
                radius_end: 150.0,
                damage: lol_core::action::damage::TargetDamage {
                    filter: lol_core::action::damage::TargetFilter::All,
                    amount: "total_damage".to_string(),
                    damage_type: lol_core::damage::DamageType::Physical,
                    ..Default::default()
                },
            },
            skill: skill.spell.clone(),
        });
        commands.trigger(ActionDash {
            entity,
            point: trigger.point,
            move_type: DashMoveType::Pointer { max: 400.0 },
            speed: 900.0,
        });
        apply_camille_e_as(&mut commands, entity, as_buff, as_duration);
        commands.entity(trigger.skill_entity).remove::<SkillRecastWindow>();
        commands.entity(trigger.skill_entity).insert((CoolDown {
            duration: cooldown.duration,
            timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
        },));
    }
}

/// E 攻速计时：到期移除 `BuffAttack` 与计时 buff。
pub fn update_camille_e(
    mut commands: Commands,
    mut q: Query<(Entity, &BuffOf, &mut BuffCamilleE)>,
    time: Res<Time<Fixed>>,
) {
    for (e, bo, mut buff) in q.iter_mut() {
        buff.timer.tick(time.delta());
        if !buff.timer.is_finished() {
            continue;
        }
        commands.entity(bo.0).remove::<BuffAttack>();
        commands.entity(e).despawn();
    }
}