use bevy::prelude::*;
use league_utils::hash_bin;
use lol_base::render_cmd::{CommandAnimationPlay, CommandSkinParticleSpawn};
use lol_core::action::dash::{ActionDash, DashMoveType};
use lol_core::base::buff::BuffOf;
use lol_core::buffs::shield_white::BuffShieldWhite;

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
        hash: "spell3".to_string(),
        repeat: false,
        duration: None,
    });
    commands
        .entity(entity)
        .with_related::<BuffOf>(BuffShieldWhite::new(shield_value));
    commands.trigger(ActionDash {
        entity,
        point,
        skill: Handle::default(),
        move_type: DashMoveType::Fixed(250.0),
        damage: None,
        speed: 1000.0,
    });
}
