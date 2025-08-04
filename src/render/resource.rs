use crate::{config::GameConfig, render::LeagueLoader};
use bevy::prelude::*;
use cdragon_prop::{BinHash, BinMap, BinStruct};

#[derive(Resource)]
pub struct WadRes {
    pub loader: LeagueLoader,
}

pub struct PluginResource;

impl Plugin for PluginResource {
    fn build(&self, app: &mut App) {
        #[cfg(unix)]
        let loader = LeagueLoader::new(
            r"/mnt/c/Program Files (x86)/WeGameApps/英雄联盟/game",
            r"DATA/FINAL/Maps/Shipping/Map11.wad.client",
            r"data/maps/mapgeometry/map11/bloom.mapgeo",
        )
        .unwrap();
        #[cfg(windows)]
        let loader = LeagueLoader::new(
            r"C:\Program Files (x86)\WeGameApps\英雄联盟\game",
            r"DATA\FINAL\Maps\Shipping\Map11.wad.client",
            r"data/maps/mapgeometry/map11/bloom.mapgeo",
        )
        .unwrap();

        app.insert_resource::<GameConfig>(GameConfig {
            minion_paths: loader
                .map_materials
                .0
                .entries
                .iter()
                .filter(|v| v.ctype.hash == LeagueLoader::hash_bin("MapPlaceableContainer"))
                .map(|v| {
                    v.getv::<BinMap>(LeagueLoader::hash_bin("items").into())
                        .unwrap()
                })
                .flat_map(|v| v.downcast::<BinHash, BinStruct>().unwrap())
                .filter(|v| v.1.ctype.hash == 0x3c995caf)
                .map(|v| (&v.1).into())
                .collect(),
        });

        app.insert_resource::<WadRes>(WadRes { loader });
    }
}
