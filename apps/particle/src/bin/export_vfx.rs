use std::fs;
use std::path::PathBuf;

use lol_share::{ConfigVfx, ConfigVfxSystemDefinition};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebParticleSystem {
    pub name: String,
    pub hash: u32,
    pub hash_hex: String,
    pub emitter_count: usize,
    pub ron_str: String,
    pub def: ConfigVfxSystemDefinition,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebHeroSummary {
    pub id: String,
    pub name: String,
    pub category: String, // "five_tigers", "four_sisters", "other"
    pub count: usize,
    pub systems: Vec<WebSystemSummaryItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebSystemSummaryItem {
    pub name: String,
    pub hash: u32,
    pub hash_hex: String,
    pub emitter_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebHeroDetail {
    pub hero_id: String,
    pub systems: Vec<WebParticleSystem>,
}

fn categorize_hero(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    match lower.as_str() {
        "aatrox" | "darius" | "mordekaiser" | "sett" | "volibear" => "five_tigers",
        "camille" | "fiora" | "irelia" | "riven" => "four_sisters",
        _ => "other",
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 扫描 League 英雄粒子系统资产 ===");

    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("无法找到工程根目录")
        .to_path_buf();

    let characters_dir = workspace_root.join("assets").join("characters");
    let output_dir = workspace_root.join("apps").join("particle").join("web").join("public").join("data");
    fs::create_dir_all(&output_dir)?;

    let mut hero_dirs: Vec<PathBuf> = fs::read_dir(&characters_dir)?
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    hero_dirs.sort();

    println!("发现 {} 个英雄目录，开始多线程解析 skin0_vfx.ron...", hero_dirs.len());

    let results: Vec<(String, String, Vec<WebParticleSystem>)> = hero_dirs
        .into_par_iter()
        .filter_map(|path| {
            let hero_name = path.file_name()?.to_string_lossy().to_string();
            let vfx_path = path.join("skins").join("skin0_vfx.ron");
            if !vfx_path.is_file() {
                return None;
            }
            let content = fs::read_to_string(&vfx_path).ok()?;
            let config: ConfigVfx = match ron::from_str(&content) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("解析 {}/skin0_vfx.ron 失败: {}", hero_name, e);
                    return None;
                }
            };

            let category = categorize_hero(&hero_name).to_string();
            let mut systems = Vec::with_capacity(config.systems.len());
            for (&hash, def) in &config.systems {
                if let Ok(ron_str) = ron::ser::to_string(def) {
                    let emitter_count = def.complex_emitter_definition_data.as_ref().map(|l| l.len()).unwrap_or(0)
                        + def.simple_emitter_definition_data.as_ref().map(|l| l.len()).unwrap_or(0);

                    systems.push(WebParticleSystem {
                        name: def.particle_name.clone(),
                        hash,
                        hash_hex: format!("0x{:08x}", hash),
                        emitter_count,
                        ron_str,
                        def: def.clone(),
                    });
                }
            }
            systems.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            Some((hero_name, category, systems))
        })
        .collect();

    println!("成功解析 {} 个英雄的粒子系统", results.len());

    // 生成目录摘要 heroes_catalog.json
    let mut catalog: Vec<WebHeroSummary> = results
        .iter()
        .map(|(hero_name, category, systems)| {
            let items: Vec<WebSystemSummaryItem> = systems
                .iter()
                .map(|s| WebSystemSummaryItem {
                    name: s.name.clone(),
                    hash: s.hash,
                    hash_hex: s.hash_hex.clone(),
                    emitter_count: s.emitter_count,
                })
                .collect();

            WebHeroSummary {
                id: hero_name.to_lowercase(),
                name: hero_name.clone(),
                category: category.clone(),
                count: systems.len(),
                systems: items,
            }
        })
        .collect();

    // 排序：上单五虎 / 四姐妹置顶，其余字母排序
    catalog.sort_by(|a, b| {
        let cat_rank = |c: &str| match c {
            "five_tigers" => 0,
            "four_sisters" => 1,
            _ => 2,
        };
        let ra = cat_rank(&a.category);
        let rb = cat_rank(&b.category);
        if ra != rb {
            ra.cmp(&rb)
        } else {
            a.name.cmp(&b.name)
        }
    });

    let catalog_json = serde_json::to_string_pretty(&catalog)?;
    fs::write(output_dir.join("heroes_catalog.json"), catalog_json)?;
    println!("已生成 heroes_catalog.json ({} 个英雄)", catalog.len());

    // 为每个英雄保存独立的 JSON 文件：public/data/heroes/{hero_name}.json
    let heroes_dir = output_dir.join("heroes");
    fs::create_dir_all(&heroes_dir)?;

    for (hero_name, _, systems) in &results {
        let detail = WebHeroDetail {
            hero_id: hero_name.to_lowercase(),
            systems: systems.clone(),
        };
        let file_path = heroes_dir.join(format!("{}.json", hero_name.to_lowercase()));
        let json_str = serde_json::to_string(&detail)?;
        fs::write(file_path, json_str)?;
    }
    println!("已为全部英雄生成独立系统详情数据至 public/data/heroes/*.json");

    Ok(())
}
