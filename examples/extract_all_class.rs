use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use bevy::prelude::*;
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;
use league_property::extract::{
    class_map_to_rust_code, extract_entry_class, get_hashes, merge_class_maps,
};
use league_utils::hash_bin;
use rayon::prelude::*;

fn main() {
    let root_dir = r"D:\WeGameApps\英雄联盟\Game";

    let start = Instant::now();

    let loader = LeagueLoader::from_relative_path(
        root_dir,
        vec![
            "DATA/FINAL/Global.wad.client",
            "DATA/FINAL/UI.wad.client",
            "DATA/FINAL/UI.zh_CN.wad.client",
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
        ],
    )
    .with_all_champions();

    println!("加载 wad 耗时: {:?}", start.elapsed());

    let start = Instant::now();

    let hash_paths = vec![
        "assets/hashes/hashes.binentries.txt",
        "assets/hashes/hashes.binfields.txt",
        "assets/hashes/hashes.binhashes.txt",
        "assets/hashes/hashes.bintypes.txt",
    ];

    let hashes = get_hashes(&hash_paths);

    let need_extract = HashSet::from([
        hash_bin("ItemData"),
        hash_bin("AnimationGraphData"),
        hash_bin("BarracksConfig"),
        hash_bin("CharacterRecord"),
        hash_bin("FloatingInfoBarViewController"),
        hash_bin("HeroFloatingInfoBarData"),
        hash_bin("MapContainer"),
        hash_bin("MapPlaceableContainer"),
        hash_bin("ResourceResolver"),
        hash_bin("SkinCharacterDataProperties"),
        hash_bin("SpellObject"),
        hash_bin("StaticMaterialDef"),
        hash_bin("StructureFloatingInfoBarData"),
        hash_bin("UiElementEffectAnimationData"),
        hash_bin("UiElementGroupButtonData"),
        hash_bin("UiElementIconData"),
        hash_bin("UiElementRegionData"),
        hash_bin("UiPropertyLoadable"),
        hash_bin("UISceneData"),
        hash_bin("UnitFloatingInfoBarData"),
        hash_bin("UnitStatusPriorityList"),
        hash_bin("VfxSystemDefinitionData"),
        0xad65d8c4,
    ]);

    let need_defaults = HashSet::from([hash_bin("SpellDataResource"), hash_bin("SpellObject")]);

    // 收集所有 (wad_index, entry_hash) 任务
    let tasks: Vec<_> = loader
        .wads
        .iter()
        .enumerate()
        .flat_map(|(wad_index, wad)| {
            wad.wad
                .entries
                .keys()
                .copied()
                .map(move |hash| (wad_index, hash))
        })
        .collect();

    let total_tasks = tasks.len();
    let processed_count = AtomicUsize::new(0);

    // 使用线程安全容器收集结果
    let class_map = Mutex::new(HashMap::new());

    tasks.par_iter().for_each(|(wad_index, hash)| {
        let wad = &loader.wads[*wad_index];

        // 无论成功与否都增加计数
        let current = processed_count.fetch_add(1, Ordering::Relaxed) + 1;
        if current % 10000 == 0 || current == total_tasks {
            println!("已处理 {} / {} 个 entry", current, total_tasks);
        }

        let Ok(bin) = wad.get_prop_bin_by_hash(*hash) else {
            return;
        };

        let mut bin_class_map = HashMap::new();
        for (class_hash, entry) in bin.iter_class_hash_and_entry() {
            if !need_extract.contains(&class_hash) {
                continue;
            }

            let class_map_entry = extract_entry_class(class_hash, entry).unwrap();
            merge_class_maps(&mut bin_class_map, class_map_entry);
        }

        // 合并到全局 class_map
        let mut class_map_guard = class_map.lock().unwrap();
        merge_class_maps(&mut class_map_guard, bin_class_map);
    });

    let mut class_map = class_map.into_inner().unwrap();

    let (rust_code, init_code) =
        class_map_to_rust_code(&mut class_map, &hashes, &need_extract, &need_defaults).unwrap();

    write("league.rs", rust_code).unwrap();
    write("init_league_asset.rs", init_code).unwrap();

    println!("耗时: {:?}", start.elapsed());
}
