//! W - 防御之舞 (Defiant Dance)

use bevy::prelude::*;
use bevy::time::{Timer, TimerMode};
use lol_base::animation_names::ANIM_SPELL2;
use lol_base::render_cmd::CommandAnimationPlay;
use lol_base::spell::Spell;
use lol_core::base::buff::{Buff, BuffOf, Buffs};
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{CommandDamageCreate, Damage, DamageType};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    CoolDown, EventSkillCast, Skill, SkillRecastWindow, SkillSlot, get_skill_value,
};
use lol_core::team::Team;

use crate::irelia::Irelia;

pub const IRELIA_W_DR: f32 = 0.5;
pub const IRELIA_W_MAX_DURATION: f32 = 1.5;
pub const IRELIA_W_CHARGE_FOR_MAX: f32 = 0.75;
pub const IRELIA_W_RADIUS: f32 = 300.0;

#[derive(Component, Clone, Debug)]
#[require(Buff = Buff { name: "IreliaW" })]
pub struct BuffIreliaW {
    pub timer: Timer,
}

impl BuffIreliaW {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(IRELIA_W_MAX_DURATION, TimerMode::Once),
        }
    }
}

fn clear_irelia_w_buffs(
    commands: &mut Commands,
    entity: Entity,
    q_buffs: &Query<&Buffs>,
    q_w: &Query<&BuffIreliaW>,
    q_dr: &Query<&BuffDamageReduction>,
) {
    let Some(buffs) = q_buffs.get(entity).ok() else {
        return;
    };
    let mut to_despawn = Vec::new();
    for buff in buffs.iter() {
        if q_w.get(buff).is_ok() || q_dr.get(buff).is_ok() {
            to_despawn.push(buff);
        }
    }
    for buff in to_despawn {
        commands.entity(buff).despawn();
    }
}

pub fn on_irelia_w(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_irelia: Query<&Team, With<Irelia>>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
    q_enemies: Query<(Entity, &Transform, &Team), With<Champion>>,
    q_buffs: Query<&Buffs>,
    q_w: Query<&BuffIreliaW>,
    q_dr: Query<&BuffDamageReduction>,
    q_damage: Query<&Damage>,
    res_spells: Res<Assets<Spell>>,
) {
    let entity = trigger.event_target();
    let Ok(team) = q_irelia.get(entity) else {
        return;
    };
    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };
    if !matches!(skill.slot, SkillSlot::W) {
        return;
    }
    let Some(spell) = res_spells.get(&skill.spell) else {
        return;
    };
    let ad = q_damage.get(entity).map(|d| d.0).unwrap_or(0.0);
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    commands.trigger(CommandAnimationPlay {
        entity,
        hash: ANIM_SPELL2.to_string(),
        repeat: false,
        duration: None,
    });

    if stage == 1 {
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffDamageReduction::new(IRELIA_W_DR, None));
        commands
            .entity(entity)
            .with_related::<BuffOf>(BuffIreliaW::new());
        commands
            .entity(trigger.skill_entity)
            .insert(SkillRecastWindow::new(2, 2, IRELIA_W_MAX_DURATION));
        return;
    }

    let charge = q_buffs
        .get(entity)
        .ok()
        .and_then(|buffs| buffs.iter().find_map(|b| q_w.get(b).ok()))
        .map(|w| w.timer.elapsed().as_secs_f32())
        .unwrap_or(0.0);
    let frac = (charge / IRELIA_W_CHARGE_FOR_MAX).clamp(0.0, 1.0);

    let stat_getter = |stat: u8| if stat == 2 { ad } else { 0.0 };
    let min = get_skill_value(spell, "min_damage_calc", skill.level, stat_getter).unwrap_or(0.0);
    let max = get_skill_value(spell, "max_damage_calc", skill.level, stat_getter).unwrap_or(0.0);
    let amount = min + frac * (max - min);

    for (target, tf, t) in q_enemies.iter() {
        if *t == *team {
            continue;
        }
        if tf.translation.xz().distance(trigger.point) > IRELIA_W_RADIUS {
            continue;
        }
        commands.entity(target).trigger(|e| CommandDamageCreate {
            entity: e,
            source: entity,
            damage_type: DamageType::Physical,
            amount,
            tag: None,
        });
    }

    clear_irelia_w_buffs(&mut commands, entity, &q_buffs, &q_w, &q_dr);
    commands
        .entity(trigger.skill_entity)
        .remove::<SkillRecastWindow>();
    commands.entity(trigger.skill_entity).insert(CoolDown {
        duration: cooldown.duration,
        timer: Some(Timer::from_seconds(cooldown.duration, TimerMode::Once)),
    });
}

pub fn update_irelia_w(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut q_w: Query<(Entity, &mut BuffIreliaW, &BuffOf)>,
    q_buffs: Query<&Buffs>,
    q_dr: Query<&BuffDamageReduction>,
) {
    let mut expired: Vec<(Entity, Entity)> = Vec::new();
    for (entity, mut w, bo) in q_w.iter_mut() {
        w.timer.tick(time.delta());
        if w.timer.is_finished() {
            expired.push((entity, bo.0));
        }
    }
    for (w_entity, parent) in expired {
        commands.entity(w_entity).despawn();
        if let Ok(buffs) = q_buffs.get(parent) {
            for b in buffs.iter() {
                if q_dr.get(b).is_ok() {
                    commands.entity(b).despawn();
                }
            }
        }
    }
}
