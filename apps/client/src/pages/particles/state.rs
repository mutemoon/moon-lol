//! 粒子系统编辑器：全局页面状态 + 树数据结构 + Rayon 扫描 + 树操作。

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use gpui::prelude::*;
use gpui::*;
use lol_share::{ConfigVfx, ConfigVfxSystemDefinition};
use rayon::prelude::*;

use crate::components::sidebar::AppSidebar;
use crate::services::particle_service::{self, ParticleSystemDef, ParticleWsHandle};

// ── 树节点数据结构 ──

#[derive(Debug, Clone)]
pub struct ParticleTreeNode {
    pub name: String,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub children: Vec<ParticleTreeNode>,
    pub hero_name: String,
    pub hash: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct FlatParticleNode {
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub children_count: usize,
    pub hero_name: String,
    pub hash: Option<u32>,
}

// ── 页面本地状态 ──

pub struct ParticlesPageState {
    pub ws_url: String,
    pub connected: bool,
    pub error: Option<String>,
    pub tree_roots: Vec<ParticleTreeNode>,
    pub hero_systems: HashMap<String, Vec<ParticleSystemDef>>,
    pub is_initialized: bool,
    pub is_scanning: bool,
    pub selected_hero: Option<String>,
    pub selected_system: Option<ParticleSystemDef>,
    pub active_tab: usize,
    pub auto_play: bool,
    pub ws_handle: Option<ParticleWsHandle>,
    /// 左侧搜索框（英雄 / 粒子名 / hash）
    pub search_query: String,
    /// 当前可编辑的工作副本（来自 selected_system 的深拷贝）
    pub working_def: Option<ConfigVfxSystemDefinition>,
    /// 选中时的原始定义，用于「重置单个 / 重置系统」
    pub initial_def_backup: Option<ConfigVfxSystemDefinition>,
}

impl Default for ParticlesPageState {
    fn default() -> Self {
        Self {
            ws_url: String::from("ws://127.0.0.1:9002"),
            connected: false,
            error: None,
            tree_roots: Vec::new(),
            hero_systems: HashMap::new(),
            is_initialized: false,
            is_scanning: false,
            selected_hero: None,
            selected_system: None,
            active_tab: 0,
            auto_play: true,
            ws_handle: None,
            search_query: String::new(),
            working_def: None,
            initial_def_backup: None,
        }
    }
}

thread_local! {
    pub(super) static STATE: RefCell<ParticlesPageState> = RefCell::new(ParticlesPageState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&ParticlesPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut ParticlesPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 与 update_state 相同但返回闭包结果。
pub(super) fn update_state_returns<R>(f: impl FnOnce(&mut ParticlesPageState) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

pub(super) fn hash_hex(hash: u32) -> String {
    format!("0x{:08x}", hash)
}

pub(super) fn format_number(v: f32) -> String {
    if v == v.trunc() && v.abs() < 1e7 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Rayon 多线程并行扫描英雄与粒子系统定义
pub(super) fn scan_particles_rayon() -> (
    Vec<ParticleTreeNode>,
    HashMap<String, Vec<ParticleSystemDef>>,
) {
    let Ok(base) = particle_service::characters_dir() else {
        return (Vec::new(), HashMap::new());
    };
    let Ok(read_dir) = std::fs::read_dir(&base) else {
        return (Vec::new(), HashMap::new());
    };
    let hero_paths: Vec<PathBuf> = read_dir
        .flatten()
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|e| e.path())
        .collect();

    let results: Vec<(ParticleTreeNode, (String, Vec<ParticleSystemDef>))> = hero_paths
        .into_par_iter()
        .filter_map(|path| {
            let hero_name = path.file_name()?.to_string_lossy().to_string();
            let vfx_path = path.join("skins").join("skin0_vfx.ron");
            if !vfx_path.is_file() {
                return None;
            }
            let content = std::fs::read_to_string(&vfx_path).ok()?;
            let config: ConfigVfx = ron::from_str(&content).ok()?;
            let mut systems = Vec::with_capacity(config.systems.len());
            for (&hash, def) in &config.systems {
                if let Ok(def_ron) = ron::ser::to_string(def) {
                    systems.push(ParticleSystemDef {
                        name: def.particle_name.clone(),
                        hash,
                        def_ron,
                        def: def.clone(),
                    });
                }
            }
            systems.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            let children: Vec<ParticleTreeNode> = systems
                .iter()
                .map(|sys| ParticleTreeNode {
                    name: sys.name.clone(),
                    is_dir: false,
                    is_expanded: false,
                    children: Vec::new(),
                    hero_name: hero_name.clone(),
                    hash: Some(sys.hash),
                })
                .collect();

            let tree_node = ParticleTreeNode {
                name: hero_name.clone(),
                is_dir: true,
                is_expanded: false,
                children,
                hero_name: hero_name.clone(),
                hash: None,
            };

            Some((tree_node, (hero_name, systems)))
        })
        .collect();

    let mut tree_nodes = Vec::with_capacity(results.len());
    let mut systems_map = HashMap::with_capacity(results.len());
    for (node, (hero, systems)) in results {
        tree_nodes.push(node);
        systems_map.insert(hero, systems);
    }
    tree_nodes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    (tree_nodes, systems_map)
}

/// 切换树节点的折叠/展开状态
pub(super) fn toggle_node_expansion(nodes: &mut [ParticleTreeNode], target_hero: &str) -> bool {
    for node in nodes.iter_mut() {
        if node.is_dir && node.name == target_hero {
            node.is_expanded = !node.is_expanded;
            return true;
        }
    }
    false
}

fn has_matching_child(node: &ParticleTreeNode, query_lower: &str) -> bool {
    for child in &node.children {
        if child.name.to_lowercase().contains(query_lower) {
            return true;
        }
        if let Some(hash) = child.hash {
            if format!("0x{:08x}", hash).contains(query_lower) {
                return true;
            }
        }
        if child.is_dir && has_matching_child(child, query_lower) {
            return true;
        }
    }
    false
}

/// 平铺处于展开显示状态的树节点（折叠的节点绝对不占用 DOM 资源）
pub(super) fn collect_flat_visible_nodes(
    nodes: &[ParticleTreeNode],
    depth: usize,
    query_lower: &str,
    acc: &mut Vec<FlatParticleNode>,
) {
    let is_empty_query = query_lower.is_empty();
    for node in nodes {
        let name_matches = is_empty_query || node.name.to_lowercase().contains(query_lower);
        let hash_matches = !is_empty_query
            && node
                .hash
                .map_or(false, |h| format!("0x{:08x}", h).contains(query_lower));

        if node.is_dir {
            let child_matches = !is_empty_query && has_matching_child(node, query_lower);
            let should_show = name_matches || child_matches;

            if should_show {
                acc.push(FlatParticleNode {
                    name: node.name.clone(),
                    is_dir: true,
                    depth,
                    is_expanded: node.is_expanded || child_matches,
                    children_count: node.children.len(),
                    hero_name: node.hero_name.clone(),
                    hash: None,
                });

                if node.is_expanded || child_matches {
                    collect_flat_visible_nodes(&node.children, depth + 1, query_lower, acc);
                }
            }
        } else if name_matches || hash_matches {
            acc.push(FlatParticleNode {
                name: node.name.clone(),
                is_dir: false,
                depth,
                is_expanded: false,
                children_count: 0,
                hero_name: node.hero_name.clone(),
                hash: node.hash,
            });
        }
    }
}

/// 手动刷新：保留已展开目录的展开状态，重新 Rayon 扫描。
pub(super) fn start_async_rescan(cx: &mut Context<AppSidebar>) {
    update_state(|s| {
        s.is_scanning = true;
    });
    cx.notify();

    let weak_entity = cx.entity().downgrade();
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let res = crate::services::runtime::tokio_runtime()
                .spawn_blocking(scan_particles_rayon)
                .await;
            let (mut roots, systems_map) = res.unwrap_or_default();

            let _ = weak_entity.update(&mut cx, |_, cx| {
                update_state(|state| {
                    let old_expanded: HashSet<String> = state
                        .tree_roots
                        .iter()
                        .filter(|n| n.is_dir && n.is_expanded)
                        .map(|n| n.name.clone())
                        .collect();

                    for node in &mut roots {
                        if node.is_dir && old_expanded.contains(&node.name) {
                            node.is_expanded = true;
                        }
                    }
                    state.tree_roots = roots;
                    state.hero_systems = systems_map;
                    state.is_scanning = false;
                });
                cx.notify();
            });
        }
    })
    .detach();
}
