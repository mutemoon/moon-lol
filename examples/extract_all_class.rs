use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use bevy::prelude::*;
use rayon::prelude::*;

use league_loader::{LeagueLoader, PropBinLoader};
use league_property::{class_map_to_rust_code, extract_all_class, get_hashes, merge_class_maps};

fn main() {
    let root_dir = r"D:\Program Files\Riot Games\League of Legends\Game";

    let start = Instant::now();

    let loader = LeagueLoader::full(root_dir).unwrap();

    println!("加载 wad 耗时: {:?}", start.elapsed());

    let start = Instant::now();

    let hash_paths = vec![
        "assets/hashes/hashes.binentries.txt",
        "assets/hashes/hashes.binfields.txt",
        "assets/hashes/hashes.binhashes.txt",
        "assets/hashes/hashes.bintypes.txt",
    ];

    let rust_code = {
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
        let entry_hashes = Mutex::new(HashSet::new());

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

            // 收集 entry_hashes
            let mut entry_hashes_guard = entry_hashes.lock().unwrap();
            for class_hash in &bin.entry_classes {
                entry_hashes_guard.insert(*class_hash);
            }

            // 提取 class_map
            let bin_class_map = extract_all_class(&bin).unwrap();

            // 合并到全局 class_map
            let mut class_map_guard = class_map.lock().unwrap();
            merge_class_maps(&mut class_map_guard, bin_class_map);
        });

        let mut class_map = class_map.into_inner().unwrap();
        let entry_hashes = entry_hashes.into_inner().unwrap();
        let hashes = get_hashes(&hash_paths);

        let rust_code = class_map_to_rust_code(&mut class_map, &hashes, &entry_hashes).unwrap();

        rust_code
    };

    write("league.rs", rust_code).unwrap();

    println!("耗时: {:?}", start.elapsed());
}
