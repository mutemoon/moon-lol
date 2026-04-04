use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_base::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffSlow;
use lol_core::buffs::common_buffs::{BuffMoveSpeed, BuffSelfHeal};
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

const AATROX_Q_KEY: &str = "Characters/Aatrox/Spells/AatroxQ/AatroxQ";
const AATROX_W_KEY: &str = "Characters/Aatrox/Spells/AatroxW/AatroxW";
const AATROX_E_KEY: &str = "Characters/Aatrox/Spells/AatroxE/AatroxE";
const AATROX_R_KEY: &str = "Characters/Aatrox/Spells/AatroxR/AatroxR";
const AATROX_Q_RECAST_WINDOW: f32 = 3.0;

#[derive(Default)]
pub struct PluginAatrox;

impl Plugin for PluginAatrox {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_aatrox_skill_cast);
        app.add_observer(on_aatrox_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Aatrox"))]
#[reflect(Component)]
pub struct Aatrox;

fn on_aatrox_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
    q_transform: Query<&Transform>,
    q_skill: Query<(&Skill, &CoolDown, Option<&SkillRecastWindow>)>,
) {
    let entity = trigger.event_target();
    if q_aatrox.get(entity).is_err() {
        return;
    }

    let Ok((skill, cooldown, recast)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_aatrox_q(
            &mut commands,
            &q_transform,
            entity,
            trigger.skill_entity,
            trigger.point,
            cooldown,
            recast,
        ),
        SkillSlot::W => cast_aatrox_w(&mut commands, entity),
        SkillSlot::E => cast_aatrox_e(&mut commands, &q_transform, entity, trigger.point),
        SkillSlot::R => cast_aatrox_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_aatrox_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    skill_entity: Entity,
    point: Vec2,
    cooldown: &CoolDown,
    recast: Option<&SkillRecastWindow>,
) {
    // Q has 3 stages with different animations
    let stage = recast.map(|w| w.stage).unwrap_or(1);

    let animation_hash = match stage {
        1 => hash_bin("Spell1A"),
        2 => hash_bin("Spell1B"),
        _ => hash_bin("Spell1C"),
    };

    play_skill_animation(commands, entity, animation_hash);
    spawn_skill_particle(commands, entity, hash_bin("Aatrox_Q_Cast"));

    // Q is a 3-hit combo, each hit has different damage shape
    // Using circular damage similar to Riven
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: AATROX_Q_KEY.into(),
            move_type: DashMoveType::Fixed(200.0),
            damage: Some(DashDamage {
                radius_end: 200.0,
                damage: TargetDamage {
                    filter: TargetFilter::All,
                    amount: hash_bin("TotalDamage"),
                    damage_type: DamageType::Physical,
                },
            }),
            speed: 800.0,
        },
    );

    if stage >= 3 {
        commands.entity(skill_entity).remove::<SkillRecastWindow>();
        // After 3rd Q, start cooldown
        commands.entity(skill_entity).insert(CoolDown {
            timer: Timer::from_seconds(cooldown.duration, TimerMode::Once),
            duration: cooldown.duration,
        });
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}，开始冷却",
            entity, "Aatrox Q", stage
        );
    } else {
        commands.entity(skill_entity).insert(SkillRecastWindow::new(
            stage + 1,
            3,
            AATROX_Q_RECAST_WINDOW,
        ));
        debug!(
            "{:?} 释放了 {} 技能，当前阶段 {}",
            entity, "Aatrox Q", stage
        );
    }
}

fn cast_aatrox_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Aatrox_W_Cast"));
    // W is a chain that pulls enemies back after delay
    skill_damage(
        commands,
        entity,
        AATROX_W_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aatrox_W_Hit")),
    );
}

fn cast_aatrox_e(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Aatrox_E_Cast"));
    // E is a dash that heals based on damage dealt
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: AATROX_E_KEY.into(),
            move_type: DashMoveType::Pointer { max: 250.0 },
            damage: None,
            speed: 900.0,
        },
    );
    // Self-heal based on damage
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffSelfHeal::new(30.0));
}

fn cast_aatrox_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Aatrox_R_Cast"));
    // R is a self-cast that makes Aatrox unstoppable and deals damage
    skill_damage(
        commands,
        entity,
        AATROX_R_KEY,
        DamageShape::Circle { radius: 300.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Physical,
        }],
        Some(hash_bin("Aatrox_R_Hit")),
    );
    // Movement speed buff
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.5, 8.0));
}

/// 监听 Aatrox 造成的伤害，W 命中施加减速
fn on_aatrox_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_aatrox: Query<(), With<Aatrox>>,
) {
    if q_aatrox.get(trigger.source).is_err() {
        return;
    }
    let target = trigger.event_target();
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.25, 1.5));
}

fn add_skills(
    mut commands: Commands,
    q_aatrox: Query<Entity, (With<Aatrox>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_aatrox.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Aatrox/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Aatrox/Spells/AatroxPassiveAbility/AatroxPassive",
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
