use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use bevy::prelude::*;
use league_loader::{LeagueLoader, LeagueWadLoaderTrait};
use league_property::{class_map_to_rust_code, extract_entry_class, get_hashes, merge_class_maps};
use league_utils::hash_bin;
use rayon::prelude::*;

fn main() {
    let root_dir = r"D:\WeGameApps\英雄联盟\Game";

    let start = Instant::now();

    let loader = LeagueLoader::from_relative_path(
        root_dir,
        vec![
            "DATA/FINAL/UI.wad.client",
            "DATA/FINAL/UI.zh_MY.wad.client",
            "DATA/FINAL/Maps/Shipping/Map11.wad.client",
            "DATA/FINAL/Champions/Ziggs.wad.client",
            "DATA/FINAL/Champions/Zac.wad.client",
            "DATA/FINAL/Champions/Zed.wad.client",
            "DATA/FINAL/Champions/Zaahen.wad.client",
            "DATA/FINAL/Champions/Yuumi.wad.client",
            "DATA/FINAL/Champions/Yunara.wad.client",
            "DATA/FINAL/Champions/Yorick.wad.client",
            "DATA/FINAL/Champions/Yone.wad.client",
            "DATA/FINAL/Champions/Yasuo.wad.client",
            "DATA/FINAL/Champions/Xerath.wad.client",
            "DATA/FINAL/Champions/XinZhao.wad.client",
            "DATA/FINAL/Champions/Xayah.wad.client",
            "DATA/FINAL/Champions/Warwick.wad.client",
            "DATA/FINAL/Champions/Volibear.wad.client",
            "DATA/FINAL/Champions/Viktor.wad.client",
            "DATA/FINAL/Champions/Vladimir.wad.client",
            "DATA/FINAL/Champions/Viego.wad.client",
            "DATA/FINAL/Champions/Velkoz.wad.client",
            "DATA/FINAL/Champions/Vex.wad.client",
            "DATA/FINAL/Champions/Vi.wad.client",
            "DATA/FINAL/Champions/Veigar.wad.client",
            "DATA/FINAL/Champions/Varus.wad.client",
            "DATA/FINAL/Champions/Vayne.wad.client",
            "DATA/FINAL/Champions/Udyr.wad.client",
            "DATA/FINAL/Champions/Urgot.wad.client",
            "DATA/FINAL/Champions/Twitch.wad.client",
            "DATA/FINAL/Champions/TwistedFate.wad.client",
            "DATA/FINAL/Champions/Tryndamere.wad.client",
            "DATA/FINAL/Champions/Tristana.wad.client",
            "DATA/FINAL/Champions/Trundle.wad.client",
            "DATA/FINAL/Champions/Thresh.wad.client",
            "DATA/FINAL/Champions/Teemo.wad.client",
            "DATA/FINAL/Champions/Talon.wad.client",
            "DATA/FINAL/Champions/Taric.wad.client",
            "DATA/FINAL/Champions/TahmKench.wad.client",
            "DATA/FINAL/Champions/Taliyah.wad.client",
            "DATA/FINAL/Champions/Syndra.wad.client",
            "DATA/FINAL/Champions/Sylas.wad.client",
            "DATA/FINAL/Champions/Smolder.wad.client",
            "DATA/FINAL/Champions/Sona.wad.client",
            "DATA/FINAL/Champions/Soraka.wad.client",
            "DATA/FINAL/Champions/Swain.wad.client",
            "DATA/FINAL/Champions/Skarner.wad.client",
            "DATA/FINAL/Champions/Sion.wad.client",
            "DATA/FINAL/Champions/Sivir.wad.client",
            "DATA/FINAL/Champions/Singed.wad.client",
            "DATA/FINAL/Champions/Shyvana.wad.client",
            "DATA/FINAL/Champions/Shen.wad.client",
            "DATA/FINAL/Champions/Shaco.wad.client",
            "DATA/FINAL/Champions/Sett.wad.client",
            "DATA/FINAL/Champions/Rumble.wad.client",
            "DATA/FINAL/Champions/Senna.wad.client",
            "DATA/FINAL/Champions/Seraphine.wad.client",
            "DATA/FINAL/Champions/Sejuani.wad.client",
            "DATA/FINAL/Champions/Ryze.wad.client",
            "DATA/FINAL/Champions/Samira.wad.client",
            "DATA/FINAL/Champions/Riven.wad.client",
            "DATA/FINAL/Champions/Rengar.wad.client",
            "DATA/FINAL/Champions/Renekton.wad.client",
            "DATA/FINAL/Champions/Renata.wad.client",
            "DATA/FINAL/Champions/Rell.wad.client",
            "DATA/FINAL/Champions/RekSai.wad.client",
            "DATA/FINAL/Champions/Rammus.wad.client",
            "DATA/FINAL/Champions/Rakan.wad.client",
            "DATA/FINAL/Champions/Quinn.wad.client",
            "DATA/FINAL/Champions/Qiyana.wad.client",
            "DATA/FINAL/Champions/Pyke.wad.client",
            "DATA/FINAL/Champions/Poppy.wad.client",
            "DATA/FINAL/Champions/Pantheon.wad.client",
            "DATA/FINAL/Champions/Ornn.wad.client",
            "DATA/FINAL/Champions/Orianna.wad.client",
            "DATA/FINAL/Champions/Olaf.wad.client",
            "DATA/FINAL/Champions/Nunu.wad.client",
            "DATA/FINAL/Champions/Nidalee.wad.client",
            "DATA/FINAL/Champions/Nilah.wad.client",
            "DATA/FINAL/Champions/Nocturne.wad.client",
            "DATA/FINAL/Champions/Neeko.wad.client",
            "DATA/FINAL/Champions/Nautilus.wad.client",
            "DATA/FINAL/Champions/Nami.wad.client",
            "DATA/FINAL/Champions/Nasus.wad.client",
            "DATA/FINAL/Champions/Morgana.wad.client",
            "DATA/FINAL/Champions/Naafiri.wad.client",
            "DATA/FINAL/Champions/MonkeyKing.wad.client",
            "DATA/FINAL/Champions/Mordekaiser.wad.client",
            "DATA/FINAL/Champions/Milio.wad.client",
            "DATA/FINAL/Champions/MissFortune.wad.client",
            "DATA/FINAL/Champions/MasterYi.wad.client",
            "DATA/FINAL/Champions/Mel.wad.client",
            "DATA/FINAL/Champions/Lux.wad.client",
            "DATA/FINAL/Champions/Malzahar.wad.client",
            "DATA/FINAL/Champions/Malphite.wad.client",
            "DATA/FINAL/Champions/Maokai.wad.client",
            "DATA/FINAL/Champions/Lulu.wad.client",
            "DATA/FINAL/Champions/Lissandra.wad.client",
            "DATA/FINAL/Champions/Lucian.wad.client",
            "DATA/FINAL/Champions/Lillia.wad.client",
            "DATA/FINAL/Champions/Leona.wad.client",
            "DATA/FINAL/Champions/LeeSin.wad.client",
            "DATA/FINAL/Champions/Leblanc.wad.client",
            "DATA/FINAL/Champions/Kled.wad.client",
            "DATA/FINAL/Champions/KogMaw.wad.client",
            "DATA/FINAL/Champions/Khazix.wad.client",
            "DATA/FINAL/Champions/Kindred.wad.client",
            "DATA/FINAL/Champions/Kennen.wad.client",
            "DATA/FINAL/Champions/Kayn.wad.client",
            "DATA/FINAL/Champions/Kayle.wad.client",
            "DATA/FINAL/Champions/Katarina.wad.client",
            "DATA/FINAL/Champions/Kassadin.wad.client",
            "DATA/FINAL/Champions/Karthus.wad.client",
            "DATA/FINAL/Champions/Karma.wad.client",
            "DATA/FINAL/Champions/Kaisa.wad.client",
            "DATA/FINAL/Champions/Kalista.wad.client",
            "DATA/FINAL/Champions/Jinx.wad.client",
            "DATA/FINAL/Champions/KSante.wad.client",
            "DATA/FINAL/Champions/Jhin.wad.client",
            "DATA/FINAL/Champions/Jayce.wad.client",
            "DATA/FINAL/Champions/JarvanIV.wad.client",
            "DATA/FINAL/Champions/Jax.wad.client",
            "DATA/FINAL/Champions/Irelia.wad.client",
            "DATA/FINAL/Champions/Janna.wad.client",
            "DATA/FINAL/Champions/Ivern.wad.client",
            "DATA/FINAL/Champions/Gwen.wad.client",
            "DATA/FINAL/Champions/Heimerdinger.wad.client",
            "DATA/FINAL/Champions/Illaoi.wad.client",
            "DATA/FINAL/Champions/Hwei.wad.client",
            "DATA/FINAL/Champions/Hecarim.wad.client",
            "DATA/FINAL/Champions/Graves.wad.client",
            "DATA/FINAL/Champions/Gragas.wad.client",
            "DATA/FINAL/Champions/Gnar.wad.client",
            "DATA/FINAL/Champions/Garen.wad.client",
            "DATA/FINAL/Champions/Gangplank.wad.client",
            "DATA/FINAL/Champions/FiddleSticks.wad.client",
            "DATA/FINAL/Champions/Fizz.wad.client",
            "DATA/FINAL/Champions/Galio.wad.client",
            "DATA/FINAL/Champions/Fiora.wad.client",
            "DATA/FINAL/Champions/Evelynn.wad.client",
            "DATA/FINAL/Champions/Ezreal.wad.client",
            "DATA/FINAL/Champions/Elise.wad.client",
            "DATA/FINAL/Champions/Ekko.wad.client",
            "DATA/FINAL/Champions/Draven.wad.client",
            "DATA/FINAL/Champions/DrMundo.wad.client",
            "DATA/FINAL/Champions/Diana.wad.client",
            "DATA/FINAL/Champions/Darius.wad.client",
            "DATA/FINAL/Champions/Corki.wad.client",
            "DATA/FINAL/Champions/Caitlyn.wad.client",
            "DATA/FINAL/Champions/Camille.wad.client",
            "DATA/FINAL/Champions/Cassiopeia.wad.client",
            "DATA/FINAL/Champions/Chogath.wad.client",
            "DATA/FINAL/Champions/Briar.wad.client",
            "DATA/FINAL/Champions/Braum.wad.client",
            "DATA/FINAL/Champions/Blitzcrank.wad.client",
            "DATA/FINAL/Champions/Brand.wad.client",
            "DATA/FINAL/Champions/Aurora.wad.client",
            "DATA/FINAL/Champions/Azir.wad.client",
            "DATA/FINAL/Champions/Bard.wad.client",
            "DATA/FINAL/Champions/Belveth.wad.client",
            "DATA/FINAL/Champions/AurelionSol.wad.client",
            "DATA/FINAL/Champions/Ashe.wad.client",
            "DATA/FINAL/Champions/Aphelios.wad.client",
            "DATA/FINAL/Champions/Annie.wad.client",
            "DATA/FINAL/Champions/Anivia.wad.client",
            "DATA/FINAL/Champions/Akali.wad.client",
            "DATA/FINAL/Champions/Ambessa.wad.client",
            "DATA/FINAL/Champions/Amumu.wad.client",
            "DATA/FINAL/Champions/Alistar.wad.client",
            "DATA/FINAL/Champions/Akshan.wad.client",
            "DATA/FINAL/Champions/Ahri.wad.client",
            "DATA/FINAL/Champions/Aatrox.wad.client",
            "DATA/FINAL/Champions/Zyra.wad.client",
            "DATA/FINAL/Champions/Zoe.wad.client",
            "DATA/FINAL/Champions/Zilean.wad.client",
            "DATA/FINAL/Champions/Zeri.wad.client",
        ],
    );

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
