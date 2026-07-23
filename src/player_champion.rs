//! 根据 `--champion` 参数在 `Startup` 直接 spawn 自机英雄。
//!
//! `classic.ron` 仅保留敌方英雄；自机英雄（带 `SelfPlayer`/`Controller`/`Focus`）
//! 由命令行 `--champion` 指定身份，在启动时直接生成。初始位置参考 `classic.ron`
//! 原首位英雄。

use bevy::prelude::*;
use lol_base::character::{ConfigCharacterRecord, ConfigSkin};
use lol_base::particle::ConfigVfx;
use lol_champions::aatrox::Aatrox;
use lol_champions::camille::Camille;
use lol_champions::darius::Darius;
use lol_champions::fiora::Fiora;
use lol_champions::irelia::Irelia;
use lol_champions::mordekaiser::Mordekaiser;
use lol_champions::riven::Riven;
use lol_champions::sett::Sett;
use lol_champions::volibear::Volibear;
use lol_core::team::Team;
use lol_render::camera::Focus;
use lol_render::controller::{Controller, SelfPlayer};
use lol_render::particle::VfxHandle;

/// 命令行指定的自机英雄名（小写），如 `"darius"`。
#[derive(Resource, Default)]
pub struct PlayerChampion(pub String);

pub struct PluginPlayerChampion;

impl Plugin for PluginPlayerChampion {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_champion);
    }
}

/// `classic.ron` 原首位（自机）英雄的初始位置。
const PLAYER_SPAWN_POSITION: Vec3 = Vec3::new(1981.0, 0.0, 11441.0);

fn spawn_player_champion(
    player: Res<PlayerChampion>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let name = player.0.trim().to_lowercase();
    if name.is_empty() {
        return;
    }
    info!("spawn 自机英雄: {name}");
    match name.as_str() {
        "aatrox" => spawn(
            &mut commands,
            &asset_server,
            Aatrox,
            "characters/aatrox/config.ron",
            "characters/aatrox/skins/skin0.ron",
        ),
        "camille" => spawn(
            &mut commands,
            &asset_server,
            Camille,
            "characters/camille/config.ron",
            "characters/camille/skins/skin0.ron",
        ),
        "darius" => spawn(
            &mut commands,
            &asset_server,
            Darius,
            "characters/darius/config.ron",
            "characters/darius/skins/skin0.ron",
        ),
        "fiora" => spawn(
            &mut commands,
            &asset_server,
            Fiora,
            "characters/fiora/config.ron",
            "characters/fiora/skins/skin0.ron",
        ),
        "irelia" => spawn(
            &mut commands,
            &asset_server,
            Irelia,
            "characters/irelia/config.ron",
            "characters/irelia/skins/skin0.ron",
        ),
        "mordekaiser" => spawn(
            &mut commands,
            &asset_server,
            Mordekaiser,
            "characters/mordekaiser/config.ron",
            "characters/mordekaiser/skins/skin0.ron",
        ),
        "riven" => spawn(
            &mut commands,
            &asset_server,
            Riven,
            "characters/riven/config.ron",
            "characters/riven/skins/skin0.ron",
        ),
        "sett" => spawn(
            &mut commands,
            &asset_server,
            Sett,
            "characters/sett/config.ron",
            "characters/sett/skins/skin0.ron",
        ),
        "volibear" => spawn(
            &mut commands,
            &asset_server,
            Volibear,
            "characters/volibear/config.ron",
            "characters/volibear/skins/skin0.ron",
        ),
        other => {
            error!(
                "未知或不支持的自机英雄: {other}（支持: aatrox/camille/darius/fiora/irelia/mordekaiser/riven/sett/volibear）"
            );
        }
    }
}

fn spawn<C: Component>(
    commands: &mut Commands,
    asset_server: &AssetServer,
    marker: C,
    config_path: &'static str,
    skin_path: &'static str,
) {
    // 加载 vfx.ron 并持有 handle，防止资产被卸载，触发 inject_vfx_assets 注入粒子系统定义和解析器
    let vfx_path = config_path.replace("config.ron", "vfx.ron");
    let vfx_handle = asset_server.load::<ConfigVfx>(&vfx_path);

    commands.spawn((
        VfxHandle(vfx_handle),
        marker,
        Transform::from_translation(PLAYER_SPAWN_POSITION),
        Team::Order,
        Controller::default(),
        SelfPlayer,
        Focus,
        ConfigCharacterRecord {
            character_record: asset_server.load::<DynamicWorld>(config_path),
        },
        ConfigSkin {
            skin: asset_server.load::<DynamicWorld>(skin_path),
        },
    ));
}
