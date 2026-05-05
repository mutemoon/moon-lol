use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::debug_sphere::DebugSphere;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;

use crate::riven::buffs::ShieldVisual;

pub struct PluginRivenE;

impl Plugin for PluginRivenE {
    fn build(&self, _app: &mut App) {}
}

pub fn cast_riven_e(
    commands: &mut Commands,
    _q_transform: &Query<&Transform>,
    entity: Entity,
    point: Vec2,
    shield_value: f32,
) {
    commands.trigger(CommandSkinParticleSpawn {
        entity,
        hash: hash_bin("Riven_E_Mis"),
    });
    commands.trigger(CommandAnimationPlay {
        entity,
        hash: "Spell3".to_string(),
        repeat: false,
        duration: None,
    });

    // 创建护盾 buff 实体并建立关系
    let buff_entity = commands.spawn(BuffShieldWhite::new(shield_value)).id();
    commands
        .entity(entity)
        .add_related::<BuffOf>(&[buff_entity]);

    // 创建 3 个环绕球体
    let mut c0 = Entity::PLACEHOLDER;
    let mut c1 = Entity::PLACEHOLDER;
    let mut c2 = Entity::PLACEHOLDER;
    for (i, child) in [&mut c0, &mut c1, &mut c2].into_iter().enumerate() {
        let orb = commands
            .spawn((
                DebugSphere {
                    radius: 8.0,
                    color: Color::WHITE,
                },
                Transform::from_translation(Vec3::new(
                    100.0 * (i as f32 * core::f32::consts::TAU / 3.0).cos(),
                    50.0,
                    100.0 * (i as f32 * core::f32::consts::TAU / 3.0).sin(),
                )),
            ))
            .id();
        commands.entity(entity).add_child(orb);
        *child = orb;
    }
    commands.entity(entity).insert(ShieldVisual {
        children: [c0, c1, c2],
        angle: 0.0,
        buff_entity,
    });

    commands.trigger(ActionDash {
        entity,
        point,
        skill: Handle::default(),
        move_type: DashMoveType::Fixed(250.0),
        damage: None,
        speed: 1000.0,
    });
}
