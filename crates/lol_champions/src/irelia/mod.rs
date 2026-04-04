pub mod buffs;

use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;
use lol_core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use lol_core::action::dash::{ActionDash, DashDamage, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::cc_debuffs::DebuffStun;
use lol_core::buffs::damage_reduction::BuffDamageReduction;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{
    play_skill_animation, skill_damage, skill_dash, skill_slot_from_index, spawn_skill_particle,
    CoolDown, EventSkillCast, PassiveSkillOf, Skill, SkillCooldownMode, SkillOf, SkillRecastWindow,
    SkillSlot, Skills,
};

use crate::irelia::buffs::{DebuffIreliaUnsteady, IreliaBuff};

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
        app.add_observer(on_irelia_damage_hit);
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

fn cast_irelia_q(
    commands: &mut Commands,
    q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Irelia_Q_Cast"));
    // Q is a dash that resets on kill and marks enemies as Unsteady
    skill_dash(
        commands,
        q_transform,
        entity,
        point,
        &ActionDash {
            skill: IRELIA_Q_KEY.into(),
            move_type: DashMoveType::Pointer { max: 250.0 },
            damage: Some(DashDamage {
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
}

fn cast_irelia_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Irelia_W_Cast"));
    // W is a channel that grants damage reduction then releases damage
    let (buff_irelia_w, buff_damage_reduction) = BuffDamageReduction::irelia_w(0.5, 1.5);
    commands
        .entity(entity)
        .with_related::<BuffOf>(buff_irelia_w);
    commands
        .entity(entity)
        .with_related::<BuffOf>(buff_damage_reduction);
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
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}

/// 监听 Irelia 造成的伤害，E/R 命中给目标标记不稳 + 眩晕
fn on_irelia_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_irelia: Query<(), With<Irelia>>,
) {
    let source = trigger.source;
    if q_irelia.get(source).is_err() {
        return;
    }
    let target = trigger.event_target();
    // E/R 命中给目标标记不稳 + 眩晕
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffIreliaUnsteady::new(5.0));
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffStun::new(0.75));
}
