use crate::combat::damage::{DamageEvent, DamageType};
use crate::combat::life::Health;
use crate::combat::target::Target;
use crate::combat::ActionSetMoveTarget;
use crate::{system_debug, system_info, system_warn};
use bevy::prelude::*;

pub struct PluginAttack;

impl Plugin for PluginAttack {
    fn build(&self, app: &mut App) {
        app.add_event::<AttackLock>();
        app.add_event::<AttackAttack>();
        app.add_event::<AttackRecover>();
        app.add_event::<CommandAttack>();
        app.add_event::<TargetInAttackRange>();
        app.add_systems(
            FixedUpdate,
            (
                attack_range_check,
                trigger_lock,
                trigger_attack,
                trigger_recover,
            ),
        );
        app.add_observer(action_set_lock_time);
        app.add_observer(action_set_attack_time);
        app.add_observer(action_attack_damage);
        app.add_observer(action_attack_reset);
        app.add_observer(action_set_recover_time);
        app.add_observer(on_command_attack);
        app.add_observer(on_attack_lock);
        app.add_observer(on_attack_attack);
        app.add_observer(on_attack_recover);
        app.add_observer(on_move);
    }
}

#[derive(Component, PartialEq, Debug, Default)]
pub enum AttackState {
    #[default]
    Idle,
    Locking,
    Attacking,
    Recovering,
}

#[derive(Component, Default)]
#[require(Health)]
pub struct Attackable;

#[derive(Event, Debug)]
pub struct ActionSetAttackTime;
#[derive(Event, Debug)]
pub struct AttackLock;
#[derive(Event, Debug)]
pub struct AttackAttack {
    pub target: Entity,
}
#[derive(Event, Debug)]
pub struct AttackRecover;
#[derive(Event, Debug)]
pub struct TargetInAttackRange {
    pub target: Entity,
}
#[derive(Event, Debug)]
pub struct ActionSetLockTime {
    pub target: Entity,
}
#[derive(Event, Debug)]
pub struct ActionAttackDamage;

#[derive(Event, Debug)]
pub struct ActionAttackReset;

#[derive(Event, Debug)]
pub struct ActionSetRecoverTime;

#[derive(Event, Debug)]
pub struct CommandAttack {
    pub target: Entity,
}

#[derive(Component)]
pub struct AttackInfo {
    pub lock_time: f32,
    pub attack_time: f32,
    pub recover_time: f32,
    pub target: Option<Entity>,
}

#[derive(Component)]
pub struct AttackRange(pub f32);

impl Default for AttackInfo {
    fn default() -> Self {
        Self {
            lock_time: f32::MAX,
            attack_time: f32::MAX,
            recover_time: f32::MAX,
            target: None,
        }
    }
}

impl AttackInfo {
    pub fn set_lock_time(&mut self, target: Entity, time: f32) {
        self.lock_time = time;
        self.attack_time = f32::MAX;
        self.recover_time = f32::MAX;
        self.target = Some(target);
    }

    pub fn set_attack_time(&mut self, time: f32) {
        self.attack_time = time;
        self.lock_time = f32::MAX;
        self.recover_time = f32::MAX;
    }

    pub fn set_recover_time(&mut self, time: f32) {
        self.recover_time = time;
        self.lock_time = f32::MAX;
        self.attack_time = f32::MAX;
    }

    pub fn reset(&mut self) {
        self.lock_time = f32::MAX;
        self.attack_time = f32::MAX;
        self.recover_time = f32::MAX;
        self.target = None;
    }
}

pub fn attack_range_check(
    mut commands: Commands,
    q_attacker: Query<(Entity, &Transform, &Target, &AttackRange)>,
    q_attackable: Query<&Transform, With<Attackable>>,
) {
    let attacker_count = q_attacker.iter().count();
    if attacker_count > 0 {
        system_debug!(
            "attack_range_check",
            "Checking attack range for {} attackers",
            attacker_count
        );
    }

    for (attacker, transform, target, attack_range) in q_attacker.iter() {
        let Ok(target_transform) = q_attackable.get(target.0) else {
            system_warn!(
                "attack_range_check",
                "Attacker {:?} has invalid target {:?}",
                attacker,
                target.0
            );
            continue;
        };
        let distance = transform.translation.distance(target_transform.translation);

        system_debug!(
            "attack_range_check",
            "Attacker {:?} -> Target {:?}: distance={:.1}, range={:.1}",
            attacker,
            target.0,
            distance,
            attack_range.0
        );

        if distance <= attack_range.0 {
            system_info!(
                "attack_range_check",
                "Target {:?} is in range of attacker {:?} (distance={:.1} <= range={:.1})",
                target.0,
                attacker,
                distance,
                attack_range.0
            );
            commands.trigger_targets(TargetInAttackRange { target: target.0 }, attacker);
        }
    }
}

pub fn trigger_lock(
    mut commands: Commands,
    q_attack_info: Query<(Entity, &AttackInfo)>,
    time: Res<Time<Fixed>>,
) {
    let current_time = time.elapsed_secs();
    let mut triggered_count = 0;

    for (entity, attack_info) in q_attack_info.iter() {
        if current_time >= attack_info.lock_time {
            system_debug!(
                "trigger_lock",
                "Triggering attack lock for entity {:?} (time={:.3} >= lock_time={:.3})",
                entity,
                current_time,
                attack_info.lock_time
            );
            commands.trigger_targets(AttackLock, entity);
            triggered_count += 1;
        }
    }

    if triggered_count > 0 {
        system_info!(
            "trigger_lock",
            "Triggered attack lock for {} entities",
            triggered_count
        );
    }
}

pub fn trigger_attack(
    mut commands: Commands,
    q_attack_info: Query<(Entity, &AttackInfo)>,
    time: Res<Time<Fixed>>,
) {
    let current_time = time.elapsed_secs();
    let mut triggered_count = 0;

    for (entity, attack_info) in q_attack_info.iter() {
        if current_time >= attack_info.attack_time {
            if let Some(target) = attack_info.target {
                system_debug!("trigger_attack",
                    "Triggering attack for entity {:?} -> target {:?} (time={:.3} >= attack_time={:.3})",
                    entity, target, current_time, attack_info.attack_time
                );
                commands.trigger_targets(AttackAttack { target }, entity);
                triggered_count += 1;
            } else {
                system_warn!(
                    "trigger_attack",
                    "Entity {:?} ready to attack but has no target",
                    entity
                );
            }
        }
    }

    if triggered_count > 0 {
        system_info!(
            "trigger_attack",
            "Triggered attack for {} entities",
            triggered_count
        );
    }
}

pub fn trigger_recover(
    mut commands: Commands,
    q_attack_info: Query<(Entity, &AttackInfo)>,
    time: Res<Time<Fixed>>,
) {
    let current_time = time.elapsed_secs();
    let mut triggered_count = 0;

    for (entity, attack_info) in q_attack_info.iter() {
        if current_time >= attack_info.recover_time {
            system_debug!(
                "trigger_recover",
                "Triggering attack recovery for entity {:?} (time={:.3} >= recover_time={:.3})",
                entity,
                current_time,
                attack_info.recover_time
            );
            commands.trigger_targets(AttackRecover, entity);
            triggered_count += 1;
        }
    }

    if triggered_count > 0 {
        system_info!(
            "trigger_recover",
            "Triggered attack recovery for {} entities",
            triggered_count
        );
    }
}

pub fn action_set_lock_time(
    trigger: Trigger<ActionSetLockTime>,
    mut q_attack_info: Query<&mut AttackInfo>,
    time: Res<Time<Fixed>>,
) {
    let Ok(mut attack_info) = q_attack_info.get_mut(trigger.target()) else {
        return;
    };
    attack_info.set_lock_time(trigger.target, time.elapsed_secs() + 0.3);
}

pub fn action_set_attack_time(
    trigger: Trigger<ActionSetAttackTime>,
    time: Res<Time<Fixed>>,
    mut q_attack_info: Query<&mut AttackInfo>,
) {
    let Ok(mut attack_info) = q_attack_info.get_mut(trigger.target()) else {
        return;
    };
    attack_info.set_attack_time(time.elapsed_secs() + 0.3);
}

pub fn action_attack_damage(
    trigger: Trigger<ActionAttackDamage>,
    mut commands: Commands,
    q_attack_info: Query<&AttackInfo>,
) {
    let entity = trigger.target();
    system_debug!(
        "action_attack_damage",
        "Processing attack damage for entity {:?}",
        entity
    );

    let Ok(attack_info) = q_attack_info.get(entity) else {
        system_warn!(
            "action_attack_damage",
            "Failed to find AttackInfo for entity {:?}",
            entity
        );
        return;
    };

    if let Some(target) = attack_info.target {
        // 发送伤害事件而不是直接扣血
        // 这里使用物理伤害作为默认类型，实际游戏中可以根据攻击者的属性来决定
        let damage_amount = 10.0; // 基础攻击伤害，实际游戏中应该从攻击者的属性中获取

        system_info!(
            "action_attack_damage",
            "Entity {:?} dealing {:.1} physical damage to target {:?}",
            entity,
            damage_amount,
            target
        );

        let damage_event = DamageEvent {
            source: entity,
            target,
            damage_type: DamageType::Physical,
            amount: damage_amount,
        };
        commands.trigger_targets(damage_event, target);
    } else {
        system_warn!(
            "action_attack_damage",
            "Entity {:?} has no target to attack",
            entity
        );
    }
}

pub fn action_attack_reset(
    trigger: Trigger<ActionAttackReset>,
    mut q_attack_info: Query<&mut AttackInfo>,
) {
    let Ok(mut attack_info) = q_attack_info.get_mut(trigger.target()) else {
        return;
    };
    attack_info.reset();
}

pub fn action_set_recover_time(
    trigger: Trigger<ActionSetRecoverTime>,
    mut q_attack_info: Query<&mut AttackInfo>,
    time: Res<Time<Fixed>>,
) {
    let Ok(mut attack_info) = q_attack_info.get_mut(trigger.target()) else {
        return;
    };
    attack_info.set_recover_time(time.elapsed_secs() + 0.3);
}

fn on_command_attack(
    trigger: Trigger<CommandAttack>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
) {
    let entity = trigger.target();
    let event = trigger.event();

    if let Ok(mut attack_state) = q_attack_state.get_mut(entity) {
        match *attack_state {
            AttackState::Idle => {
                *attack_state = AttackState::Locking;
                println!(
                    "entity: {:?}, attack_state: {:?} -> {:?} event: {:?}",
                    entity,
                    AttackState::Idle,
                    AttackState::Locking,
                    event
                );
                let action = ActionAttackReset;
                println!("entity: {:?}, action: {:?}", entity, action);
                commands.trigger_targets(action, trigger.target());

                let action = ActionSetLockTime {
                    target: event.target,
                };
                println!("entity: {:?}, action: {:?}", entity, action);
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }
}

fn on_attack_lock(
    trigger: Trigger<AttackLock>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
) {
    let entity = trigger.target();
    let event = trigger.event();

    if let Ok(mut attack_state) = q_attack_state.get_mut(entity) {
        match *attack_state {
            AttackState::Locking => {
                *attack_state = AttackState::Attacking;
                println!(
                    "entity: {:?}, attack_state: {:?} -> {:?} event: {:?}",
                    entity,
                    AttackState::Locking,
                    AttackState::Attacking,
                    event
                );
                let action = ActionSetAttackTime;
                println!("entity: {:?}, action: {:?}", entity, action);
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }
}

fn on_attack_attack(
    trigger: Trigger<AttackAttack>,
    mut commands: Commands,
    mut q_attack_state: Query<&mut AttackState>,
) {
    let entity = trigger.target();
    let event = trigger.event();

    if let Ok(mut attack_state) = q_attack_state.get_mut(entity) {
        match *attack_state {
            AttackState::Attacking => {
                *attack_state = AttackState::Recovering;
                println!(
                    "entity: {:?}, attack_state: {:?} -> {:?} event: {:?}",
                    entity,
                    AttackState::Attacking,
                    AttackState::Recovering,
                    event
                );
                let action = ActionAttackDamage;
                println!("entity: {:?}, action: {:?}", entity, action);
                commands.trigger_targets(action, trigger.target());

                let action = ActionSetRecoverTime;
                println!("entity: {:?}, action: {:?}", entity, action);
                commands.trigger_targets(action, trigger.target());
            }
            _ => (),
        }
    }
}

fn on_attack_recover(trigger: Trigger<AttackRecover>, mut q_attack_state: Query<&mut AttackState>) {
    let entity = trigger.target();
    let event = trigger.event();

    if let Ok(mut attack_state) = q_attack_state.get_mut(entity) {
        match *attack_state {
            AttackState::Recovering => {
                *attack_state = AttackState::Idle;
                println!(
                    "entity: {:?}, attack_state: {:?} -> {:?} event: {:?}",
                    entity,
                    AttackState::Recovering,
                    AttackState::Idle,
                    event
                );
            }
            _ => (),
        }
    }
}

fn on_move(trigger: Trigger<ActionSetMoveTarget>, mut q_attack_state: Query<&mut AttackState>) {
    let entity = trigger.target();

    let Ok(mut attack_state) = q_attack_state.get_mut(entity) else {
        return;
    };

    if *attack_state != AttackState::Locking {
        return;
    };

    *attack_state = AttackState::Idle;
}
