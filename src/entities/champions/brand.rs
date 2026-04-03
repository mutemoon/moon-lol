use bevy::prelude::*;
use league_core::extract::CharacterRecord;
use league_utils::hash_bin;
use lol_config::prop::LoadHashKeyTrait;

use crate::buffs::brand_buffs::BuffBrandPassive;
use crate::buffs::cc_debuffs::DebuffSlow;
use crate::core::action::damage::{DamageShape, TargetDamage, TargetFilter};
use crate::core::base::buff::BuffOf;
use crate::core::damage::{DamageType, EventDamageCreate};
use crate::core::skill::{
    play_skill_animation, skill_damage, skill_slot_from_index, spawn_skill_particle, CoolDown,
    EventSkillCast, PassiveSkillOf, Skill, SkillOf, SkillSlot, Skills,
};
use crate::entities::champion::Champion;

const BRAND_Q_KEY: &str = "Characters/Brand/Spells/BrandQ/BrandQ";
const BRAND_W_KEY: &str = "Characters/Brand/Spells/BrandW/BrandW";
const BRAND_E_KEY: &str = "Characters/Brand/Spells/BrandE/BrandE";
const BRAND_R_KEY: &str = "Characters/Brand/Spells/BrandR/BrandR";

#[derive(Default)]
pub struct PluginBrand;

impl Plugin for PluginBrand {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, add_skills);
        app.add_observer(on_brand_skill_cast);
        app.add_observer(on_brand_damage_hit);
    }
}

#[derive(Component, Reflect)]
#[require(Champion, Name = Name::new("Brand"))]
#[reflect(Component)]
pub struct Brand;

fn on_brand_skill_cast(
    trigger: On<EventSkillCast>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
    q_skill: Query<(&Skill, &CoolDown)>,
) {
    let entity = trigger.event_target();
    if q_brand.get(entity).is_err() {
        return;
    }

    let Ok((skill, _cooldown)) = q_skill.get(trigger.skill_entity) else {
        return;
    };

    match skill.slot {
        SkillSlot::Q => cast_brand_q(&mut commands, entity),
        SkillSlot::W => cast_brand_w(&mut commands, entity),
        SkillSlot::E => cast_brand_e(&mut commands, entity),
        SkillSlot::R => cast_brand_r(&mut commands, entity),
        _ => {}
    }
}

fn cast_brand_q(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell1"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_Q_Cast"));

    // Q is a fireball that stuns ablazed targets
    skill_damage(
        commands,
        entity,
        BRAND_Q_KEY,
        DamageShape::Sector {
            radius: 1100.0,
            angle: 30.0,
        },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_Q_Hit")),
    );
}

fn cast_brand_w(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell2"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_W_Cast"));

    // W is a ground targeted area
    skill_damage(
        commands,
        entity,
        BRAND_W_KEY,
        DamageShape::Circle { radius: 250.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_W_Hit")),
    );
}

fn cast_brand_e(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell3"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_E_Cast"));

    // E spreads to nearby enemies
    skill_damage(
        commands,
        entity,
        BRAND_E_KEY,
        DamageShape::Circle { radius: 675.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_E_Hit")),
    );
}

fn cast_brand_r(commands: &mut Commands, entity: Entity) {
    play_skill_animation(commands, entity, hash_bin("Spell4"));
    spawn_skill_particle(commands, entity, hash_bin("Brand_R_Cast"));

    // R bounces between enemies
    skill_damage(
        commands,
        entity,
        BRAND_R_KEY,
        DamageShape::Circle { radius: 750.0 },
        vec![TargetDamage {
            filter: TargetFilter::All,
            amount: hash_bin("TotalDamage"),
            damage_type: DamageType::Magic,
        }],
        Some(hash_bin("Brand_R_Hit")),
    );
}

fn on_brand_damage_hit(
    trigger: On<EventDamageCreate>,
    mut commands: Commands,
    q_brand: Query<(), With<Brand>>,
) {
    let source = trigger.source;
    if q_brand.get(source).is_err() {
        return;
    }

    let target = trigger.event_target();

    // Apply blaze passive stacks
    commands
        .entity(target)
        .with_related::<BuffOf>(BuffBrandPassive::new());
    // R slows
    commands
        .entity(target)
        .with_related::<BuffOf>(DebuffSlow::new(0.3, 2.0));
}

fn add_skills(
    mut commands: Commands,
    q_brand: Query<Entity, (With<Brand>, Without<Skills>)>,
    res_assets_character_record: Res<Assets<CharacterRecord>>,
) {
    for entity in q_brand.iter() {
        let Some(character_record) =
            res_assets_character_record.load_hash("Characters/Brand/CharacterRecords/Root")
        else {
            continue;
        };

        commands.entity(entity).with_related::<PassiveSkillOf>((
            Skill::new(
                SkillSlot::Passive,
                "Characters/Brand/Spells/BrandPassive/BrandPassive",
            ),
            CoolDown::default(),
        ));

        for (index, &skill) in character_record.spells.as_ref().unwrap().iter().enumerate() {
            let skill_component = Skill::new(skill_slot_from_index(index), skill);
            commands
                .entity(entity)
                .with_related::<SkillOf>((skill_component, CoolDown::default()));
        }
    }
}
