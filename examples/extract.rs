use bevy::{
    asset::{io::Reader, Asset, AssetLoader, LoadContext, UnapprovedPathMode},
    image::Image,
    prelude::*,
    reflect::TypePath,
    render::mesh::Mesh,
};
use std::{thread, time::Duration};
use thiserror::Error;

#[derive(Asset, TypePath, Debug)]
pub struct TextAsset(pub String);

#[derive(Asset, TypePath, Debug, Default)]
pub struct WadAssetPack;

#[derive(Default)]
pub struct WadLoader;

impl AssetLoader for WadLoader {
    type Asset = WadAssetPack;
    type Settings = ();
    type Error = LeagueAssetLoaderError;

    async fn load<'a>(
        &self,
        _reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'a>,
    ) -> Result<Self::Asset, Self::Error> {
        info!("开始模拟加载 'test.wad.client' (总计 20 秒)...");

        for i in 0..20 {
            thread::sleep(Duration::from_secs(1));

            let asset_content = format!("这是在第 {} 秒加载的资产 #{}", i + 1, i);
            let text_asset = TextAsset(asset_content);

            let label = format!("asset_{}", i);
            info!("加载带标签的资产: '{}'", label);

            load_context.add_labeled_asset(label, text_asset);
        }

        info!("'test.wad.client' 加载完成。");

        Ok(WadAssetPack)
    }

    fn extensions(&self) -> &[&str] {
        &["wad.client"]
    }
}

#[derive(Resource)]
struct PrintTimer(Timer);

fn setup(asset_server: Res<AssetServer>) {
    info!("请求加载 'test.wad.client'...");

    asset_server.load::<WadAssetPack>("test.wad.client");
}

fn print_loaded_assets_system(
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,

    text_assets: Res<Assets<TextAsset>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        println!("\n{:-^50}", " 每3秒检查一次已加载的 TextAsset ");

        if text_assets.is_empty() {
            println!("尚未加载任何 TextAsset。");
        } else {
            for (id, asset) in text_assets.iter() {
                println!("  - Asset ID {:?}: '{}'", id, asset.0);
            }
        }
        println!("{:-^50}\n", " 检查结束 ");
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().set(AssetPlugin {
            unapproved_path_mode: UnapprovedPathMode::Allow,
            ..default()
        }))
        .init_asset::<WadAssetPack>()
        .init_asset::<TextAsset>()
        .init_asset_loader::<WadLoader>()
        .insert_resource(PrintTimer(Timer::from_seconds(3.0, TimerMode::Repeating)))
        .add_systems(Startup, setup)
        .add_systems(Update, print_loaded_assets_system)
        .run();
}

pub struct GeometryMesh {
    pub mesh: Mesh,
    pub material_image: Image,
}

#[derive(Error, Debug)]
pub enum LeagueAssetLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Default)]
pub struct LeagueAssetLoaderMesh;

impl AssetLoader for LeagueAssetLoaderMesh {
    type Asset = Mesh;

    type Settings = ();

    type Error = LeagueAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        todo!()
    }

    fn extensions(&self) -> &[&str] {
        &[".mesh"]
    }
}
