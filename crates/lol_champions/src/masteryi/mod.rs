pub mod buffs;

use bevy::asset::Handle;
use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_base::spell::Spell;
use lol_core::action::damage::{
    ActionDamage, ActionDamageEffect, DamageShape, TargetDamage, TargetFilter,
};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::common_buffs::BuffMoveSpeed;
use lol_core::damage::{DamageType, EventDamageCreate};
use lol_core::entities::champion::Champion;
use lol_core::skill::{CoolDown, EventSkillCast, Skill, SkillSlot};

use crate::masteryi::buffs::{BuffMasterYiE, BuffMasterYiR};

#[derive(Default)]
pub struct PluginMasterYi;

impl Plugin for PluginMasterYi {
    fn build(&self, app: &mut App) {
        app.add_observer(on_masteryi_skill_cast);
        app.add_observer(on_masteryi_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("MasterYi"))]
#[reflect(Component)]
pub struct MasterYi;

fn on_masteryi_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_masteryi.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    let skill_spell = skill.spell.clone();

    match skill.slot {
        SkillSlot::Q => cast_masteryi_q(&mut commands, entity, skill_spell),
        SkillSlot::W => cast_masteryi_w(&mut commands, entity),
        SkillSlot::E => cast_masteryi_e(&mut commands, entity),
        SkillSlot::R => cast_masteryi_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_masteryi_q(commands: &mut Commands, entity: Entity, skill_spell: Handle<Spell>) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell1".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("MasterYi_Q_Cast"),
    });

    // Q is a dash that damages multiple targets
    commands.trigger(ActionDamage {
        entity,
        skill: skill_spell,
        effects: vec![ActionDamageEffect {
            shape: DamageShape::Circle { radius: 400.0 },
            damage_list: vec![TargetDamage {
                filter: TargetFilter::All,
                amount: "total_damage".to_string(),
                damage_type: DamageType::Physical,
            }],
            particle: Some(hash_bin("MasterYi_Q_Hit")),
        }],
    });
}

fn cast_masteryi_w(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell2".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("MasterYi_W_Cast"),
    });

    // W is meditate (heal and damage reduction)
}

fn cast_masteryi_e(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("MasterYi_E_Cast"),
    });

    // E grants bonus true damage on attacks
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiE::new(0.3, 5.0));
}

fn cast_masteryi_r(commands: &mut Commands, entity: Entity) {
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "spell4".to_string(),
        repeat: false,
        duration: None,
    });
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("MasterYi_R_Cast"),
    });

    // R grants attackspeed and movespeed
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMasterYiR::new(0.8, 0.45, 10.0));
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffMoveSpeed::new(0.45, 10.0));
}

fn on_masteryi_damage_hit(
    trigger: On<EventDamageCreate>,
    _commands: Commands,
    q_masteryi: Query<(), With<MasterYi>>,
) {
    let source = trigger.source;
    if q_masteryi.get(source).is_err() {
        return;
    }

    // Passive: Double Strike on 4th attack
}
