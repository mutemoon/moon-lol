pub mod buffs;

use bevy::prelude::{Handle, *};
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::damage::DamageType;
use lol_core::entities::champion::Champion;
use lol_core::skill::{EventSkillCast, Skill, SkillSlot};

use crate::olaf::buffs::{BuffOlafR, BuffOlafW};

// Olaf W parameters
const OLAF_W_ATTACK_SPEED_BONUS: f32 = 0.4; // 40% attack speed
const OLAF_W_SHIELD: f32 = 80.0; // shield amount
const OLAF_W_DURATION: f32 = 5.0; // 5 seconds

// Olaf R parameters
const OLAF_R_DURATION: f32 = 6.0; // 6 seconds CC immunity

#[derive(Default)]
pub struct PluginOlaf;

impl Plugin for PluginOlaf {
    fn build(&self, app: &mut App) {
        app.add_observer(on_olaf_skill_cast);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Olaf"))]
#[reflect(Component)]
pub struct Olaf;

fn on_olaf_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_olaf: Query<(), With<Olaf>>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.event_target();
    if q_olaf.get(entity).is_err() {
        return;
    }

    let Ok(skill) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_olaf_q(&mut commands, entity, trigger.point),
        SkillSlot::W => cast_olaf_w(&mut commands, entity),
        SkillSlot::E => cast_olaf_e(&mut commands, entity, trigger.point, skill_spell),
        SkillSlot::R => cast_olaf_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_olaf_q(commands: &mut Commands, entity: Entity, _point: Vec2) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Olaf_Q_Cast"),
    });
    // Q is a linear axe throw - could add dash damage or just particle
}

fn cast_olaf_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Olaf_W_Cast"),
    });

    // W provides attack speed buff and shield
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafW::new(
            OLAF_W_ATTACK_SPEED_BONUS,
            OLAF_W_SHIELD,
            OLAF_W_DURATION,
        ));

    debug!(
        "{:?} 释放了 {} 技能，获得 {}% 攻速加成和 {} 护盾",
        entity,
        "Olaf W",
        (OLAF_W_ATTACK_SPEED_BONUS * 100.0) as i32,
        OLAF_W_SHIELD as i32
    );
}

fn cast_olaf_e(commands: &mut Commands, entity: Entity, _point: Vec2, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Olaf_E_Cast"),
    });
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Nearest {
                max_distance: 200.0,
            },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "TotalDamage".to_string(),
                damage_type: DamageType::True,
            }],
            particle: Some(hash_bin("Olaf_E_Hit")),
        }],
    });
    // E also deals self-damage
}

fn cast_olaf_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Olaf_R_Cast"),
    });

    // R provides CC immunity
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffOlafR::new(OLAF_R_DURATION));

    debug!(
        "{:?} 释放了 {} 技能，免疫控制效果持续 {} 秒",
        entity, "Olaf R", OLAF_R_DURATION
    );
}
