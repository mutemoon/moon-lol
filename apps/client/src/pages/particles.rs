use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::menu::{DropdownMenu, PopupMenuItem};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
use lol_share::{
    ConfigVfx, ConfigVfxEmitterDefinition, ConfigVfxSystemDefinition, Sampler, StochasticSampler,
    VfxTexture,
};
use rayon::prelude::*;

use crate::components::sidebar::AppSidebar;
use crate::services::particle_service::{
    self, ParticleSystemDef, ParticleWsEvent, ParticleWsHandle,
};

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
    static STATE: RefCell<ParticlesPageState> = RefCell::new(ParticlesPageState::default());
}

fn with_state<R>(f: impl FnOnce(&ParticlesPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

fn update_state(f: impl FnOnce(&mut ParticlesPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

fn hash_hex(hash: u32) -> String {
    format!("0x{:08x}", hash)
}

fn format_number(v: f32) -> String {
    if v == v.trunc() && v.abs() < 1e7 {
        format!("{}", v as i64)
    } else {
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// Rayon 多线程并行扫描英雄与粒子系统定义
pub fn scan_particles_rayon() -> (
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
fn toggle_node_expansion(nodes: &mut [ParticleTreeNode], target_hero: &str) -> bool {
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
fn collect_flat_visible_nodes(
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

fn start_async_rescan(cx: &mut Context<AppSidebar>) {
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
                    let old_expanded: std::collections::HashSet<String> = state
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

// ── 手写输入框：焦点 / 光标 / 文本缓冲（跨渲染保持） ──

#[derive(Clone)]
struct EditMeta {
    cursor: usize,
    focus: FocusHandle,
}

thread_local! {
    static EDITS: RefCell<HashMap<String, EditMeta>> = RefCell::new(HashMap::new());
    static BUFS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::new());
}

fn edit_meta(id: &str, cx: &App) -> EditMeta {
    EDITS.with(|m| {
        let mut m = m.borrow_mut();
        if let Some(meta) = m.get(id) {
            return meta.clone();
        }
        let meta = EditMeta {
            cursor: 0,
            focus: cx.focus_handle(),
        };
        m.insert(id.to_string(), meta.clone());
        meta
    })
}

fn edit_cursor(id: &str) -> usize {
    EDITS.with(|m| m.borrow().get(id).map_or(0, |e| e.cursor))
}

fn set_edit_cursor(id: &str, cursor: usize) {
    EDITS.with(|m| {
        if let Some(e) = m.borrow_mut().get_mut(id) {
            e.cursor = cursor;
        }
    })
}

fn input_buffer(id: &str) -> Option<String> {
    BUFS.with(|b| b.borrow().get(id).cloned())
}

fn set_input_buffer(id: &str, val: String) {
    BUFS.with(|b| {
        b.borrow_mut().insert(id.to_string(), val);
    })
}

fn clear_input_buffer(id: &str) {
    BUFS.with(|b| {
        b.borrow_mut().remove(id);
    })
}

fn clear_all_input_buffers() {
    BUFS.with(|b| b.borrow_mut().clear());
}

/// 处理单个按键，返回（新文本，新光标）。无变化返回 None。
fn apply_key(value: &str, cursor: usize, event: &KeyDownEvent) -> Option<(String, usize)> {
    let ks = &event.keystroke;
    let mods = &ks.modifiers;
    let mut chars: Vec<char> = value.chars().collect();
    let cursor = cursor.min(chars.len());

    if mods.control || mods.platform {
        return None;
    }

    if let Some(ch) = ks.key_char.as_deref() {
        let insert_chars: Vec<char> = ch.chars().collect();
        if !mods.alt && !insert_chars.is_empty() && !insert_chars.iter().any(|c| c.is_control()) {
            for (i, c) in insert_chars.iter().enumerate() {
                chars.insert(cursor + i, *c);
            }
            return Some((chars.into_iter().collect(), cursor + insert_chars.len()));
        }
    }

    match ks.key.as_str() {
        "backspace" => {
            if cursor > 0 {
                chars.remove(cursor - 1);
                Some((chars.into_iter().collect(), cursor - 1))
            } else {
                None
            }
        }
        "delete" => {
            if cursor < chars.len() {
                chars.remove(cursor);
                Some((chars.into_iter().collect(), cursor))
            } else {
                None
            }
        }
        "left" => Some((value.to_string(), cursor.saturating_sub(1))),
        "right" => Some((value.to_string(), (cursor + 1).min(chars.len()))),
        "home" => Some((value.to_string(), 0)),
        "end" => Some((value.to_string(), chars.len())),
        "space" => {
            chars.insert(cursor, ' ');
            Some((chars.into_iter().collect(), cursor + 1))
        }
        _ => None,
    }
}

// ── 工作副本访问 ──

/// 取「主发射器列表」：complex 非空用 complex，否则用 simple。
fn primary_list_ref(wd: &ConfigVfxSystemDefinition) -> Option<&Vec<ConfigVfxEmitterDefinition>> {
    if let Some(l) = wd.complex_emitter_definition_data.as_ref() {
        if !l.is_empty() {
            return Some(l);
        }
    }
    wd.simple_emitter_definition_data.as_ref()
}

fn primary_list_mut(
    wd: &mut ConfigVfxSystemDefinition,
) -> Option<&mut Vec<ConfigVfxEmitterDefinition>> {
    if let Some(l) = wd.complex_emitter_definition_data.as_mut() {
        if !l.is_empty() {
            return Some(l);
        }
    }
    wd.simple_emitter_definition_data.as_mut()
}

fn read_emitter(idx: usize) -> Option<ConfigVfxEmitterDefinition> {
    with_state(|s| {
        let wd = s.working_def.as_ref()?;
        primary_list_ref(wd)?.get(idx).cloned()
    })
}

fn mutate_emitter(idx: usize, f: impl FnOnce(&mut ConfigVfxEmitterDefinition)) {
    update_state(|s| {
        let Some(wd) = &mut s.working_def else {
            return;
        };
        if let Some(list) = primary_list_mut(wd) {
            if let Some(em) = list.get_mut(idx) {
                f(em);
            }
        }
    });
}

// ── 发射器字段编辑 ──

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum NumField {
    Lifetime,
    NumFrames,
    BlendMode,
    AlphaRef,
}

fn get_num_field(em: &ConfigVfxEmitterDefinition, field: NumField) -> f32 {
    match field {
        NumField::Lifetime => em.lifetime.unwrap_or(0.0),
        NumField::NumFrames => em.num_frames.map(|v| v as f32).unwrap_or(1.0),
        NumField::BlendMode => em.blend_mode.map(|v| v as f32).unwrap_or(0.0),
        NumField::AlphaRef => em.alpha_ref.map(|v| v as f32).unwrap_or(0.0),
    }
}

fn set_num_field(idx: usize, field: NumField, v: f32) {
    mutate_emitter(idx, |em| match field {
        NumField::Lifetime => em.lifetime = Some(v),
        NumField::NumFrames => em.num_frames = Some(v.max(1.0) as u16),
        NumField::BlendMode => em.blend_mode = Some(v.clamp(0.0, 255.0) as u8),
        NumField::AlphaRef => em.alpha_ref = Some(v.clamp(0.0, 255.0) as u8),
    });
}

fn set_name_idx(idx: usize, name: String) {
    mutate_emitter(idx, |em| {
        let trimmed = name.trim().to_string();
        em.emitter_name = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        };
    });
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FlagField {
    IsSingleParticle,
    IsUniformScale,
    IsRandomStartFrame,
    IsLocalOrientation,
    IsDirectionOriented,
    SoftParticle,
}

const FLAGS: &[(FlagField, &str)] = &[
    (FlagField::IsSingleParticle, "单粒子"),
    (FlagField::IsUniformScale, "等比缩放"),
    (FlagField::IsRandomStartFrame, "随机起始帧"),
    (FlagField::IsLocalOrientation, "局部朝向"),
    (FlagField::IsDirectionOriented, "方向对齐"),
    (FlagField::SoftParticle, "软粒子"),
];

fn get_flag(em: &ConfigVfxEmitterDefinition, flag: FlagField) -> bool {
    match flag {
        FlagField::IsSingleParticle => em.is_single_particle.unwrap_or(false),
        FlagField::IsUniformScale => em.is_uniform_scale.unwrap_or(false),
        FlagField::IsRandomStartFrame => em.is_random_start_frame.unwrap_or(false),
        FlagField::IsLocalOrientation => em.is_local_orientation.unwrap_or(false),
        FlagField::IsDirectionOriented => em.is_direction_oriented.unwrap_or(false),
        FlagField::SoftParticle => em.soft_particle_definition.unwrap_or(false),
    }
}

fn set_flag(em: &mut ConfigVfxEmitterDefinition, flag: FlagField, on: bool) {
    let f = Some(on);
    match flag {
        FlagField::IsSingleParticle => em.is_single_particle = f,
        FlagField::IsUniformScale => em.is_uniform_scale = f,
        FlagField::IsRandomStartFrame => em.is_random_start_frame = f,
        FlagField::IsLocalOrientation => em.is_local_orientation = f,
        FlagField::IsDirectionOriented => em.is_direction_oriented = f,
        FlagField::SoftParticle => em.soft_particle_definition = f,
    }
}

fn set_flag_idx(idx: usize, flag: FlagField, on: bool) {
    mutate_emitter(idx, |em| set_flag(em, flag, on));
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TexField {
    Texture,
    ParticleColorTexture,
    Palette,
    Reflection,
}

const TEX_ITEMS: &[(TexField, &str, &str)] = &[
    (
        TexField::Texture,
        "主贴图 texture",
        "ASSETS/Textures/particles/fire.dds",
    ),
    (
        TexField::ParticleColorTexture,
        "粒子颜色贴图 particle_color_texture",
        "ASSETS/Textures/particles/color.dds",
    ),
    (
        TexField::Palette,
        "调色板 palette_definition",
        "ASSETS/Textures/particles/palette.dds",
    ),
    (
        TexField::Reflection,
        "反射贴图 reflection_definition",
        "ASSETS/Textures/particles/reflection.dds",
    ),
];

fn get_texture(em: &ConfigVfxEmitterDefinition, f: TexField) -> String {
    match f {
        TexField::Texture => em
            .texture
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::ParticleColorTexture => em
            .particle_color_texture
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::Palette => em
            .palette_definition
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
        TexField::Reflection => em
            .reflection_definition
            .as_ref()
            .map(|t| t.path.clone())
            .unwrap_or_default(),
    }
}

fn set_texture(em: &mut ConfigVfxEmitterDefinition, f: TexField, path: String) {
    let tex = if path.trim().is_empty() {
        None
    } else {
        Some(VfxTexture::from_path(path.trim().to_string()))
    };
    match f {
        TexField::Texture => em.texture = tex,
        TexField::ParticleColorTexture => em.particle_color_texture = tex,
        TexField::Palette => em.palette_definition = tex,
        TexField::Reflection => em.reflection_definition = tex,
    }
}

fn set_texture_idx(idx: usize, f: TexField, path: String) {
    mutate_emitter(idx, |em| set_texture(em, f, path));
}

fn tex_div_values(em: &ConfigVfxEmitterDefinition) -> [f32; 2] {
    em.tex_div.map(|v| v.to_array()).unwrap_or([1.0, 1.0])
}

fn set_tex_div_comp(idx: usize, comp: usize, v: f32) {
    mutate_emitter(idx, |em| {
        let mut vals = tex_div_values(em);
        if comp < 2 {
            vals[comp] = v;
        }
        match &mut em.tex_div {
            Some(vv) => {
                vv.x = vals[0];
                vv.y = vals[1];
            }
            None => em.tex_div = Some(vals.into()),
        }
    });
}

// ── 采样器（StochasticSampler）编辑 ──
//
// 简化：常量模式直接编辑常量值；曲线模式编辑首采样点值；通过「常量/曲线」下拉切换，
// 不实现 SVG 拖拽曲线。数值写入只改 base_sampler，prob_curves 原样保留。

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SamplerKind {
    Rate,
    ParticleLifetime,
    BindWeight,
    EmitterPosition,
    BirthVelocity,
    BirthAcceleration,
    BirthRotation0,
    BirthScale0,
    Scale0,
    BirthColor,
    Color,
    BirthUvOffset,
    BirthUvScrollRate,
}

impl SamplerKind {
    fn label(&self) -> &'static str {
        match self {
            SamplerKind::Rate => "生成速率 rate",
            SamplerKind::ParticleLifetime => "粒子寿命 particle_lifetime",
            SamplerKind::BindWeight => "绑定权重 bind_weight",
            SamplerKind::EmitterPosition => "发射器位置 emitter_position",
            SamplerKind::BirthVelocity => "出生速度 birth_velocity",
            SamplerKind::BirthAcceleration => "出生加速度 birth_acceleration",
            SamplerKind::BirthRotation0 => "出生旋转 birth_rotation0",
            SamplerKind::BirthScale0 => "出生缩放 birth_scale0",
            SamplerKind::Scale0 => "缩放 scale0",
            SamplerKind::BirthColor => "出生颜色 birth_color",
            SamplerKind::Color => "颜色 color",
            SamplerKind::BirthUvOffset => "出生UV偏移 birth_uv_offset",
            SamplerKind::BirthUvScrollRate => "出生UV滚动 birth_uv_scroll_rate",
        }
    }

    fn dims(&self) -> usize {
        match self {
            SamplerKind::Rate | SamplerKind::ParticleLifetime | SamplerKind::BindWeight => 1,
            SamplerKind::BirthUvOffset | SamplerKind::BirthUvScrollRate => 2,
            SamplerKind::BirthColor | SamplerKind::Color => 4,
            _ => 3,
        }
    }

    fn all() -> [SamplerKind; 13] {
        [
            SamplerKind::Rate,
            SamplerKind::ParticleLifetime,
            SamplerKind::BindWeight,
            SamplerKind::EmitterPosition,
            SamplerKind::BirthVelocity,
            SamplerKind::BirthAcceleration,
            SamplerKind::BirthRotation0,
            SamplerKind::BirthScale0,
            SamplerKind::Scale0,
            SamplerKind::BirthColor,
            SamplerKind::Color,
            SamplerKind::BirthUvOffset,
            SamplerKind::BirthUvScrollRate,
        ]
    }
}

macro_rules! read_scalar_sampler {
    ($s:expr) => {{
        let curve = matches!(&$s.base_sampler, Sampler::Curve { .. });
        let v = match &$s.base_sampler {
            Sampler::Constant(v) => *v,
            Sampler::Curve { samples } => samples.first().map(|(_, v)| *v).unwrap_or(0.0),
        };
        (vec![v], curve)
    }};
}

macro_rules! read_vec_sampler {
    ($s:expr, $dims:expr) => {{
        let curve = matches!(&$s.base_sampler, Sampler::Curve { .. });
        let vals = match &$s.base_sampler {
            Sampler::Constant(v) => v.to_array().to_vec(),
            Sampler::Curve { samples } => samples
                .first()
                .map(|(_, v)| v.to_array().to_vec())
                .unwrap_or_else(|| vec![0.0; $dims]),
        };
        (vals, curve)
    }};
}

/// 读取采样器当前值（常量或曲线首点）与是否为曲线模式。
fn read_sampler(em: &ConfigVfxEmitterDefinition, kind: SamplerKind) -> (Vec<f32>, bool) {
    match kind {
        SamplerKind::Rate => read_scalar_sampler!(em.rate),
        SamplerKind::ParticleLifetime => read_scalar_sampler!(em.particle_lifetime),
        SamplerKind::BindWeight => read_scalar_sampler!(em.bind_weight),
        SamplerKind::EmitterPosition => read_vec_sampler!(em.emitter_position, 3),
        SamplerKind::BirthVelocity => read_vec_sampler!(em.birth_velocity, 3),
        SamplerKind::BirthAcceleration => read_vec_sampler!(em.birth_acceleration, 3),
        SamplerKind::BirthRotation0 => read_vec_sampler!(em.birth_rotation0, 3),
        SamplerKind::BirthScale0 => read_vec_sampler!(em.birth_scale0, 3),
        SamplerKind::Scale0 => read_vec_sampler!(em.scale0, 3),
        SamplerKind::BirthColor => read_vec_sampler!(em.birth_color, 4),
        SamplerKind::Color => read_vec_sampler!(em.color, 4),
        SamplerKind::BirthUvOffset => read_vec_sampler!(em.birth_uv_offset, 2),
        SamplerKind::BirthUvScrollRate => read_vec_sampler!(em.birth_uv_scroll_rate, 2),
    }
}

macro_rules! write_scalar_sampler {
    ($s:expr, $v:expr) => {{
        let v = $v;
        match &mut $s.base_sampler {
            Sampler::Constant(c) => *c = v,
            Sampler::Curve { samples } => {
                if let Some((_, c)) = samples.first_mut() {
                    *c = v;
                }
            }
        }
    }};
}

macro_rules! write_vec_sampler {
    ($s:expr, $vals:expr, [$($i:expr),*]) => {{
        let vals = $vals;
        match &mut $s.base_sampler {
            Sampler::Constant(v) => {
                let mut arr = v.to_array();
                $( arr[$i] = vals[$i]; )*
                *v = arr.into();
            }
            Sampler::Curve { samples } => {
                if let Some((_, v)) = samples.first_mut() {
                    let mut arr = v.to_array();
                    $( arr[$i] = vals[$i]; )*
                    *v = arr.into();
                }
            }
        }
    }};
}

/// 回写采样器整组数值（常量或曲线首点）。
fn write_sampler_values(em: &mut ConfigVfxEmitterDefinition, kind: SamplerKind, vals: Vec<f32>) {
    match kind {
        SamplerKind::Rate => write_scalar_sampler!(em.rate, vals[0]),
        SamplerKind::ParticleLifetime => write_scalar_sampler!(em.particle_lifetime, vals[0]),
        SamplerKind::BindWeight => write_scalar_sampler!(em.bind_weight, vals[0]),
        SamplerKind::EmitterPosition => write_vec_sampler!(em.emitter_position, vals, [0, 1, 2]),
        SamplerKind::BirthVelocity => write_vec_sampler!(em.birth_velocity, vals, [0, 1, 2]),
        SamplerKind::BirthAcceleration => {
            write_vec_sampler!(em.birth_acceleration, vals, [0, 1, 2])
        }
        SamplerKind::BirthRotation0 => write_vec_sampler!(em.birth_rotation0, vals, [0, 1, 2]),
        SamplerKind::BirthScale0 => write_vec_sampler!(em.birth_scale0, vals, [0, 1, 2]),
        SamplerKind::Scale0 => write_vec_sampler!(em.scale0, vals, [0, 1, 2]),
        SamplerKind::BirthColor => write_vec_sampler!(em.birth_color, vals, [0, 1, 2, 3]),
        SamplerKind::Color => write_vec_sampler!(em.color, vals, [0, 1, 2, 3]),
        SamplerKind::BirthUvOffset => write_vec_sampler!(em.birth_uv_offset, vals, [0, 1]),
        SamplerKind::BirthUvScrollRate => write_vec_sampler!(em.birth_uv_scroll_rate, vals, [0, 1]),
    }
}

fn set_sampler_component(idx: usize, kind: SamplerKind, comp: usize, v: f32) {
    mutate_emitter(idx, |em| {
        let (mut vals, _) = read_sampler(em, kind);
        if comp < vals.len() {
            vals[comp] = v;
        }
        write_sampler_values(em, kind, vals);
    });
}

/// 通用模式切换：常量 ↔ 曲线（2 个关键帧），需要 T: Clone，不依赖具体向量类型。
fn set_mode<T: Clone>(s: &mut StochasticSampler<T>, curve: bool) {
    let cur = match &s.base_sampler {
        Sampler::Constant(v) => Some(v.clone()),
        Sampler::Curve { samples } => samples.first().map(|(_, v)| v.clone()),
    };
    let Some(v) = cur else {
        return;
    };
    if curve {
        if matches!(&s.base_sampler, Sampler::Constant(_)) {
            s.base_sampler = Sampler::Curve {
                samples: vec![(0.0, v.clone()), (1.0, v)],
            };
        }
    } else if matches!(&s.base_sampler, Sampler::Curve { .. }) {
        s.base_sampler = Sampler::Constant(v);
    }
}

fn set_sampler_mode(em: &mut ConfigVfxEmitterDefinition, kind: SamplerKind, curve: bool) {
    match kind {
        SamplerKind::Rate => set_mode(&mut em.rate, curve),
        SamplerKind::ParticleLifetime => set_mode(&mut em.particle_lifetime, curve),
        SamplerKind::BindWeight => set_mode(&mut em.bind_weight, curve),
        SamplerKind::EmitterPosition => set_mode(&mut em.emitter_position, curve),
        SamplerKind::BirthVelocity => set_mode(&mut em.birth_velocity, curve),
        SamplerKind::BirthAcceleration => set_mode(&mut em.birth_acceleration, curve),
        SamplerKind::BirthRotation0 => set_mode(&mut em.birth_rotation0, curve),
        SamplerKind::BirthScale0 => set_mode(&mut em.birth_scale0, curve),
        SamplerKind::Scale0 => set_mode(&mut em.scale0, curve),
        SamplerKind::BirthColor => set_mode(&mut em.birth_color, curve),
        SamplerKind::Color => set_mode(&mut em.color, curve),
        SamplerKind::BirthUvOffset => set_mode(&mut em.birth_uv_offset, curve),
        SamplerKind::BirthUvScrollRate => set_mode(&mut em.birth_uv_scroll_rate, curve),
    }
}

fn set_sampler_mode_idx(idx: usize, kind: SamplerKind, curve: bool) {
    mutate_emitter(idx, |em| set_sampler_mode(em, kind, curve));
}

// ── 播放 / 重播 ──

fn spawn_play_ron(cx: &mut Context<AppSidebar>, ron: String) {
    let _weak = cx.entity().downgrade();
    cx.spawn(
        move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let handle = with_state(|s| s.ws_handle.clone());
                if let Some(h) = handle {
                    if let Err(e) = h.play_particle(&ron).await {
                        update_state(|s| {
                            s.error = Some(e);
                        });
                        if let Some(e) = weak.upgrade() {
                            let _ = e.update(&mut cx, |_, cx| cx.notify());
                        }
                    }
                }
            }
        },
    )
    .detach();
}

/// 序列化工作副本并播放（改动后自动重播的落地点）。
fn play_working(cx: &mut Context<AppSidebar>) {
    let ron = with_state(|s| {
        s.working_def
            .as_ref()
            .map(particle_service::serialize_vfx_system)
    });
    match ron {
        Some(Ok(r)) => spawn_play_ron(cx, r),
        Some(Err(e)) => {
            update_state(|s| s.error = Some(e));
            cx.notify();
        }
        None => {}
    }
}

/// 编辑提交后：若开启「改动后自动播放」则重播。
fn replay_after_edit(cx: &mut Context<AppSidebar>) {
    if with_state(|s| s.auto_play) {
        play_working(cx);
    }
    cx.notify();
}

fn stop_playing(cx: &mut Context<AppSidebar>) {
    let handle = with_state(|s| s.ws_handle.clone());
    if let Some(h) = handle {
        let _weak = cx.entity().downgrade();
        cx.spawn(
            move |weak: gpui::WeakEntity<AppSidebar>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    if let Err(e) = h.stop_particle().await {
                        update_state(|s| {
                            s.error = Some(e);
                        });
                        if let Some(e) = weak.upgrade() {
                            let _ = e.update(&mut cx, |_, cx| cx.notify());
                        }
                    }
                }
            },
        )
        .detach();
    } else {
        update_state(|s| s.error = Some("粒子 server 未连接".to_string()));
        cx.notify();
    }
}

/// 仅播放单个发射器（保留其所在列表，其余清空）。
fn play_single_emitter(cx: &mut Context<AppSidebar>, idx: usize) {
    let ron = with_state(|s| {
        let wd = s.working_def.as_ref()?;
        let mut single = wd.clone();
        let (use_complex, em) = {
            let c = single
                .complex_emitter_definition_data
                .as_ref()
                .and_then(|l| l.get(idx).cloned());
            let c2 = single
                .simple_emitter_definition_data
                .as_ref()
                .and_then(|l| l.get(idx).cloned());
            if c.is_some() {
                (true, c)
            } else {
                (false, c2)
            }
        };
        let em = em?;
        if use_complex {
            single.complex_emitter_definition_data = Some(vec![em]);
            single.simple_emitter_definition_data = None;
        } else {
            single.simple_emitter_definition_data = Some(vec![em]);
            single.complex_emitter_definition_data = None;
        }
        particle_service::serialize_vfx_system(&single).ok()
    });
    if let Some(r) = ron {
        spawn_play_ron(cx, r);
    }
}

/// 重置单个发射器为初始备份值。
fn reset_single_emitter(cx: &mut Context<AppSidebar>, idx: usize) {
    let changed = update_state_returns(|s| {
        let backup = s.initial_def_backup.as_ref()?;
        let wd = s.working_def.as_mut()?;
        let backup_em = primary_list_ref(backup)?.get(idx).cloned()?;
        let list = primary_list_mut(wd)?;
        if idx < list.len() {
            list[idx] = backup_em;
            Some(())
        } else {
            None
        }
    });
    if changed.is_some() {
        clear_all_input_buffers();
        replay_after_edit(cx);
    }
}

/// 重置整个系统为初始备份定义并重播。
fn reset_system(cx: &mut Context<AppSidebar>) {
    update_state(|s| {
        if let Some(b) = &s.initial_def_backup {
            s.working_def = Some(b.clone());
        }
    });
    clear_all_input_buffers();
    play_working(cx);
}

/// 与 update_state 相同但返回闭包结果。
fn update_state_returns<R>(f: impl FnOnce(&mut ParticlesPageState) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

// ── 输入控件 ──

fn render_search_input(cx: &mut Context<AppSidebar>, value: String) -> AnyElement {
    let id = "particle-search".to_string();
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let empty = value.is_empty();
    let chars: Vec<char> = value.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = with_state(|s| s.search_query.clone());
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            update_state(|s| s.search_query = nv);
            set_edit_cursor(&id2, nc);
            cx.notify();
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| d.text_color(muted).child("搜索英雄 / 粒子"))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 手写数字输入框：回车提交（Enter）→ commit(v)；非法输入回车则回退。
fn render_number_input(
    cx: &mut Context<AppSidebar>,
    id: String,
    value: f32,
    commit: impl Fn(f32) + 'static,
) -> AnyElement {
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let buf = input_buffer(&id);
    let display = buf.unwrap_or_else(|| format_number(value));
    let empty = display.is_empty();
    let chars: Vec<char> = display.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = input_buffer(&id2).unwrap_or_else(|| format_number(value));
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_input_buffer(&id2, nv);
            set_edit_cursor(&id2, nc);
            cx.notify();
        } else if event.keystroke.key == "enter" {
            match live.trim().parse::<f32>() {
                Ok(v) => {
                    commit(v);
                    clear_input_buffer(&id2);
                    set_edit_cursor(&id2, 0);
                    replay_after_edit(cx);
                }
                Err(_) => {
                    clear_input_buffer(&id2);
                    set_edit_cursor(&id2, 0);
                    cx.notify();
                }
            }
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| d.text_color(muted).child("0"))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 手写文本输入框：回车提交（Enter）→ commit(text)。
fn render_text_input(
    cx: &mut Context<AppSidebar>,
    id: String,
    value: String,
    placeholder: &str,
    commit: impl Fn(String) + 'static,
) -> AnyElement {
    let meta = edit_meta(&id, cx);
    let focus_handle = meta.focus.clone();
    let buf = input_buffer(&id);
    let display = buf.unwrap_or_else(|| value.clone());
    let empty = display.is_empty();
    let chars: Vec<char> = display.chars().collect();
    let cursor = meta.cursor.min(chars.len());
    let before: String = chars[..cursor].iter().collect();
    let after: String = chars[cursor..].iter().collect();
    let accent = cx.theme().accent;
    let muted = cx.theme().muted_foreground;
    let id2 = id.clone();
    let placeholder_owned = placeholder.to_string();

    let listener = cx.listener(move |_this, event: &KeyDownEvent, _window, cx| {
        let live = input_buffer(&id2).unwrap_or_else(|| value.clone());
        let cur = edit_cursor(&id2);
        if let Some((nv, nc)) = apply_key(&live, cur, event) {
            set_input_buffer(&id2, nv);
            set_edit_cursor(&id2, nc);
            cx.notify();
        } else if event.keystroke.key == "enter" {
            commit(live.clone());
            clear_input_buffer(&id2);
            set_edit_cursor(&id2, 0);
            replay_after_edit(cx);
        }
    });

    div()
        .track_focus(&focus_handle)
        .px_2()
        .py_1()
        .w_full()
        .border_1()
        .border_color(cx.theme().border)
        .rounded_md()
        .bg(cx.theme().background)
        .text_sm()
        .on_key_down(listener)
        .child(
            h_flex()
                .items_center()
                .when(empty, |d| d.text_color(muted).child(placeholder_owned))
                .when(!empty, |d| {
                    d.child(before)
                        .child(div().w(px(1.)).h(rems(1.)).bg(accent))
                        .child(after)
                }),
        )
        .into_any_element()
}

/// 常量/曲线预设下拉。
fn render_sampler_mode_dropdown(
    cx: &mut Context<AppSidebar>,
    id: String,
    idx: usize,
    kind: SamplerKind,
    is_curve: bool,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    Button::new(id)
        .label(if is_curve { "曲线" } else { "常量" })
        .outline()
        .dropdown_menu(move |menu, _window, _cx| {
            let w1 = weak.clone();
            let w2 = weak.clone();
            menu.item(
                PopupMenuItem::new("常量")
                    .checked(!is_curve)
                    .on_click(move |_, _, cx| {
                        set_sampler_mode_idx(idx, kind, false);
                        let _ = w1.update(cx, |_, cx| replay_after_edit(cx));
                    }),
            )
            .item(
                PopupMenuItem::new("曲线")
                    .checked(is_curve)
                    .on_click(move |_, _, cx| {
                        set_sampler_mode_idx(idx, kind, true);
                        let _ = w2.update(cx, |_, cx| replay_after_edit(cx));
                    }),
            )
        })
        .into_any_element()
}

/// 布尔开关（渲染标志）。
fn render_flag_toggle(
    cx: &mut Context<AppSidebar>,
    id: String,
    idx: usize,
    flag: FlagField,
    label: &str,
    checked: bool,
) -> AnyElement {
    let weak = cx.entity().downgrade();
    Checkbox::new(id)
        .checked(checked)
        .label(label)
        .on_click(move |new_checked, _, cx| {
            set_flag_idx(idx, flag, *new_checked);
            let _ = weak.update(cx, |_, cx| replay_after_edit(cx));
        })
        .into_any_element()
}

// ── 发射器编辑器 ──

fn comp_labels(dims: usize) -> &'static [&'static str] {
    match dims {
        2 => &["X", "Y"],
        4 => &["R", "G", "B", "A"],
        _ => &["X", "Y", "Z"],
    }
}

fn render_section(
    cx: &mut Context<AppSidebar>,
    title: &str,
    children: Vec<AnyElement>,
) -> AnyElement {
    v_flex()
        .gap_2()
        .child(
            div()
                .border_b_1()
                .border_color(cx.theme().border.opacity(0.6))
                .pb_1()
                .text_xs()
                .font_bold()
                .text_color(cx.theme().accent)
                .child(title.to_string()),
        )
        .children(children)
        .into_any_element()
}

fn render_sampler_row(
    cx: &mut Context<AppSidebar>,
    hash: u32,
    idx: usize,
    kind: SamplerKind,
    em: &ConfigVfxEmitterDefinition,
) -> AnyElement {
    let (vals, is_curve) = read_sampler(em, kind);
    let dims = kind.dims();
    let labels = comp_labels(dims);
    let muted = cx.theme().muted_foreground;

    v_flex()
        .gap_1()
        .p_2()
        .rounded_md()
        .border_1()
        .border_color(cx.theme().border.opacity(0.4))
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(div().text_xs().font_bold().child(kind.label()))
                .child(render_sampler_mode_dropdown(
                    cx,
                    format!("{:08x}-{}-sm-{:?}-mode", hash, idx, kind),
                    idx,
                    kind,
                    is_curve,
                )),
        )
        .child(h_flex().gap_1().children((0..dims).map(|c| {
            let id = format!("{:08x}-{}-sm-{:?}-{}", hash, idx, kind, c);
            v_flex()
                .gap_0p5()
                .flex_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(muted)
                        .child(labels.get(c).copied().unwrap_or("?").to_string()),
                )
                .child(render_number_input(
                    cx,
                    id,
                    vals.get(c).copied().unwrap_or(0.0),
                    move |v| set_sampler_component(idx, kind, c, v),
                ))
                .into_any_element()
        })))
        .into_any_element()
}

fn render_emitter_editor(
    cx: &mut Context<AppSidebar>,
    hash: u32,
    idx: usize,
    em: &ConfigVfxEmitterDefinition,
) -> AnyElement {
    let muted = cx.theme().muted_foreground;
    let title = em
        .emitter_name
        .clone()
        .unwrap_or_else(|| format!("发射器 #{}", idx + 1));

    // 基本参数
    let basic_fields = vec![
        (
            "名称 emitter_name".to_string(),
            render_text_input(
                cx,
                format!("{:08x}-{}-name", hash, idx),
                em.emitter_name.clone().unwrap_or_default(),
                "Fire_Particle",
                move |v| set_name_idx(idx, v),
            ),
        ),
        (
            "寿命 lifetime".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-lifetime", hash, idx),
                get_num_field(em, NumField::Lifetime),
                move |v| set_num_field(idx, NumField::Lifetime, v),
            ),
        ),
        (
            "帧数 num_frames".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-num_frames", hash, idx),
                get_num_field(em, NumField::NumFrames),
                move |v| set_num_field(idx, NumField::NumFrames, v),
            ),
        ),
        (
            "混合模式 blend_mode".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-blend", hash, idx),
                get_num_field(em, NumField::BlendMode),
                move |v| set_num_field(idx, NumField::BlendMode, v),
            ),
        ),
        (
            "Alpha参考 alpha_ref".to_string(),
            render_number_input(
                cx,
                format!("{:08x}-{}-alpha", hash, idx),
                get_num_field(em, NumField::AlphaRef),
                move |v| set_num_field(idx, NumField::AlphaRef, v),
            ),
        ),
    ];

    // 贴图
    let mut texture_children: Vec<AnyElement> = TEX_ITEMS
        .iter()
        .map(|(f, label, placeholder)| {
            let f = *f;
            v_flex()
                .gap_1()
                .flex_1()
                .child(div().text_xs().text_color(muted).child(label.to_string()))
                .child(render_text_input(
                    cx,
                    format!("{:08x}-{}-tex-{:?}", hash, idx, f),
                    get_texture(em, f),
                    placeholder,
                    move |v| set_texture_idx(idx, f, v),
                ))
                .into_any_element()
        })
        .collect();
    // tex_div
    let tv = tex_div_values(em);
    texture_children.push(
        v_flex()
            .gap_1()
            .flex_1()
            .child(
                div()
                    .text_xs()
                    .text_color(muted)
                    .child("贴图分割 tex_div (U/V)".to_string()),
            )
            .child(h_flex().gap_1().children((0..2).map(|c| {
                let id = format!("{:08x}-{}-texdiv-{}", hash, idx, c);
                render_number_input(cx, id, tv[c], move |v| set_tex_div_comp(idx, c, v))
                    .into_any_element()
            })))
            .into_any_element(),
    );

    let flag_children = vec![h_flex()
        .gap_3()
        .flex_wrap()
        .children(FLAGS.iter().map(|(flag, label)| {
            let checked = get_flag(em, *flag);
            render_flag_toggle(
                cx,
                format!("{:08x}-{}-flag-{:?}", hash, idx, flag),
                idx,
                *flag,
                label,
                checked,
            )
        }))
        .into_any_element()];
    let sampler_children: Vec<AnyElement> = SamplerKind::all()
        .iter()
        .map(|k| render_sampler_row(cx, hash, idx, *k, em))
        .collect();

    v_flex()
        .gap_4()
        // 发射器工具条
        .child(
            h_flex()
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(IconName::Settings)
                        .child(div().font_bold().text_sm().child(title)),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new(format!("play-single-{:08x}-{}", hash, idx))
                                .icon(IconName::Play)
                                .label("播放单个")
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    play_single_emitter(cx, idx);
                                })),
                        )
                        .child(
                            Button::new(format!("reset-single-{:08x}-{}", hash, idx))
                                .ghost()
                                .label("重置")
                                .on_click(cx.listener(move |_, _, _, cx| {
                                    reset_single_emitter(cx, idx);
                                })),
                        ),
                ),
        )
        // 基本参数
        .child(render_section(
            cx,
            "基本参数",
            vec![h_flex()
                .gap_3()
                .flex_wrap()
                .children(basic_fields.into_iter().map(|(label, input)| {
                    v_flex()
                        .gap_1()
                        .w_40()
                        .child(div().text_xs().text_color(muted).child(label))
                        .child(input)
                        .into_any_element()
                }))
                .into_any_element()],
        ))
        // 渲染标志
        .child(render_section(cx, "渲染标志", flag_children))
        // 采样器
        .child(render_section(
            cx,
            "采样器（数值输入 + 常量/曲线预设）",
            sampler_children,
        ))
        // 贴图资源
        .child(render_section(cx, "贴图资源", texture_children))
        .into_any_element()
}

// ── 右侧系统详情面板 ──

fn render_system_detail(
    cx: &mut Context<AppSidebar>,
    hero: &str,
    name: &str,
    hash: u32,
    wd: &ConfigVfxSystemDefinition,
) -> AnyElement {
    let emitters: Vec<ConfigVfxEmitterDefinition> =
        primary_list_ref(wd).map(|l| l.clone()).unwrap_or_default();
    let emitter_count = emitters.len();
    let active_tab = with_state(|s| s.active_tab);

    v_flex()
        .size_full()
        .child(
            // 面板顶栏
            h_flex()
                .px_4()
                .py_2()
                .border_b_1()
                .border_color(cx.theme().border)
                .bg(cx.theme().background)
                .items_center()
                .justify_between()
                .child(
                    h_flex()
                        .gap_2()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .font_bold()
                                .child(format!("{} / {}  {} 个发射器", hero, name, emitter_count)),
                        )
                        .child(
                            div()
                                .px_1p5()
                                .py_0p5()
                                .rounded_md()
                                .bg(cx.theme().accent.opacity(0.1))
                                .text_xs()
                                .child(hash_hex(hash)),
                        ),
                )
                .child(
                    h_flex()
                        .gap_2()
                        .child(
                            Button::new("particle-reset-sys-btn")
                                .ghost()
                                .icon(IconName::Redo)
                                .label("重置系统")
                                .on_click(cx.listener(|_, _, _, cx| reset_system(cx))),
                        )
                        .child(
                            Button::new("particle-stop-btn")
                                .icon(IconName::CircleX)
                                .label("停止")
                                .on_click(cx.listener(|_, _, _, cx| stop_playing(cx))),
                        )
                        .child(
                            Button::new("particle-play-btn")
                                .icon(IconName::Play)
                                .label("播放")
                                .on_click(cx.listener(|_, _, _, cx| play_working(cx))),
                        ),
                ),
        )
        .when(emitter_count == 0, |d| {
            d.child(
                div().flex_1().flex().items_center().justify_center().child(
                    v_flex().gap_2().items_center().child(IconName::File).child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("该粒子系统未包含发射器定义数据"),
                    ),
                ),
            )
        })
        .when(emitter_count > 0, |d| {
            d.child(
                v_flex()
                    .size_full()
                    // 发射器 tab 行
                    .child(
                        h_flex()
                            .px_2()
                            .py_1()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .gap_1()
                            .children(emitters.iter().enumerate().map(|(idx, em)| {
                                let is_active = active_tab == idx;
                                let label = em
                                    .emitter_name
                                    .clone()
                                    .unwrap_or_else(|| format!("发射器 #{}", idx + 1));
                                let btn = Button::new(format!("emitter-tab-{}", idx)).label(label);
                                let btn = if is_active { btn } else { btn.ghost() };
                                btn.on_click(cx.listener(move |_, _, _, cx| {
                                    update_state(|s| {
                                        s.active_tab = idx;
                                    });
                                    clear_all_input_buffers();
                                    cx.notify();
                                }))
                                .into_any_element()
                            })),
                    )
                    // 编辑器主体
                    .child(
                        emitters
                            .get(active_tab)
                            .map(|em| {
                                div()
                                    .flex_1()
                                    .overflow_y_scrollbar()
                                    .p_4()
                                    .child(render_emitter_editor(cx, hash, active_tab, em))
                                    .into_any_element()
                            })
                            .unwrap_or_else(|| div().flex_1().into_any_element()),
                    ),
            )
        })
        .into_any_element()
}

// ── 公开入口 ──

/// 粒子系统编辑器：Rayon 树状文件侧边栏 → 选中系统 → 发射器参数编辑 → 自动重播。
pub fn render_particles(_sidebar: &mut AppSidebar, cx: &mut Context<AppSidebar>) -> AnyElement {
    let should_start_scan = STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if !state.is_initialized && !state.is_scanning {
            state.is_initialized = true;
            state.is_scanning = true;
            true
        } else {
            false
        }
    });

    // 首次进入时，在后台 Rayon 多线程异步构建英雄粒子树
    if should_start_scan {
        let weak_entity = cx.entity().downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let res = crate::services::runtime::tokio_runtime()
                    .spawn_blocking(scan_particles_rayon)
                    .await;
                let (roots, systems_map) = res.unwrap_or_default();

                let _ = weak_entity.update(&mut cx, |_, cx| {
                    update_state(|state| {
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

    let (
        flat_visible_nodes,
        total_heroes,
        total_particles,
        connected,
        error,
        ws_url,
        query,
        auto_play,
        is_scanning,
        selected_system,
        hero,
        name,
        hash,
        wd,
    ) = with_state(|s| {
        let mut flat = Vec::new();
        let query_lower = s.search_query.trim().to_lowercase();
        collect_flat_visible_nodes(&s.tree_roots, 0, &query_lower, &mut flat);
        let total_heroes = s.tree_roots.len();
        let total_particles: usize = s.tree_roots.iter().map(|n| n.children.len()).sum();

        (
            flat,
            total_heroes,
            total_particles,
            s.connected,
            s.error.clone(),
            s.ws_url.clone(),
            s.search_query.clone(),
            s.auto_play,
            s.is_scanning,
            s.selected_system.clone(),
            s.selected_hero.clone(),
            s.selected_system.as_ref().map(|x| x.name.clone()),
            s.selected_system.as_ref().map(|x| x.hash),
            s.working_def.clone(),
        )
    });

    let theme = cx.theme();
    let sidebar_bg = theme.sidebar;
    let border_color = theme.border;
    let bg_color = theme.background;
    let accent_color = theme.accent;
    let accent_fg = theme.accent_foreground;
    let muted_fg = theme.muted_foreground;
    let danger_color = theme.danger;

    // 右侧详情面板
    let right_panel = match (hero, name, hash, wd) {
        (Some(h), Some(n), Some(hh), Some(w)) => render_system_detail(cx, &h, &n, hh, &w),
        _ => div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .gap_2()
                    .items_center()
                    .child(IconName::Palette)
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted_fg)
                            .child("请在左侧展开英雄并选择粒子系统"),
                    ),
            )
            .into_any_element(),
    };

    let search_input_elem = render_search_input(cx, query.clone());
    let page_header_elem = render_page_header(cx, &ws_url, connected, auto_play);

    h_flex()
        .w_full()
        .h_full()
        .gap_3()
        .child(
            // ── 左侧英雄粒子树侧边栏（完全对齐 wad_browser 结构与视觉风格） ──
            v_flex()
                .w_80()
                .h_full()
                .p_3()
                .gap_2()
                .bg(sidebar_bg)
                .border_r_1()
                .border_color(border_color)
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().font_bold().text_base().child("英雄粒子树"))
                                .when(is_scanning, |this| {
                                    this.child(div().text_xs().opacity(0.6).child("(扫描中...)"))
                                }),
                        )
                        .child(
                            Button::new("refresh-particles-tree")
                                .icon(IconName::Redo)
                                .ghost()
                                .small()
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    start_async_rescan(cx);
                                })),
                        ),
                )
                // 搜索过滤框
                .child(
                    v_flex()
                        .gap_1()
                        .child(
                            h_flex()
                                .w_full()
                                .px_2()
                                .py_1()
                                .bg(bg_color)
                                .border_1()
                                .border_color(border_color)
                                .rounded_md()
                                .items_center()
                                .gap_2()
                                .child(IconName::Search)
                                .child(div().flex_1().child(search_input_elem))
                                .when(!query.is_empty(), |this| {
                                    this.child(
                                        Button::new("particle-search-clear")
                                            .ghost()
                                            .small()
                                            .icon(IconName::Close)
                                            .on_click(cx.listener(|_, _, _, cx| {
                                                update_state(|s| {
                                                    s.search_query.clear();
                                                });
                                                set_edit_cursor("particle-search", 0);
                                                clear_input_buffer("particle-search");
                                                cx.notify();
                                            })),
                                    )
                                }),
                        )
                        .child(div().px_1().text_xs().text_color(muted_fg).child(
                            if query.is_empty() {
                                format!("共 {} 个英雄 · {} 个粒子", total_heroes, total_particles)
                            } else {
                                let matched_count =
                                    flat_visible_nodes.iter().filter(|n| !n.is_dir).count();
                                format!("匹配 {} 个粒子", matched_count)
                            },
                        )),
                )
                // 树状列表区域（严格只挂载当前展开显示的节点 DOM）
                .child(
                    v_flex()
                        .flex_1()
                        .w_full()
                        .overflow_y_scrollbar()
                        .gap_0p5()
                        .when(is_scanning, |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .py_4()
                                    .flex()
                                    .justify_center()
                                    .text_sm()
                                    .opacity(0.6)
                                    .child("Rayon 多线程扫描英雄粒子中..."),
                            )
                        })
                        .when(!is_scanning && flat_visible_nodes.is_empty(), |this| {
                            this.child(
                                div()
                                    .w_full()
                                    .py_4()
                                    .flex()
                                    .justify_center()
                                    .text_sm()
                                    .opacity(0.6)
                                    .child("无匹配粒子节点"),
                            )
                        })
                        .children(flat_visible_nodes.into_iter().map(|node| {
                            let is_dir = node.is_dir;
                            let is_selected = !is_dir
                                && selected_system
                                    .as_ref()
                                    .map(|s| Some(s.hash) == node.hash)
                                    .unwrap_or(false);
                            let padding_left_px = (node.depth * 14 + 6) as f32;
                            let hero_name = node.hero_name.clone();
                            let node_hash = node.hash;

                            div()
                                .w_full()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .pl(px(padding_left_px))
                                .when(is_selected, |this| {
                                    this.bg(accent_color).text_color(accent_fg)
                                })
                                .when(!is_selected, |this| {
                                    this.hover(|style| style.bg(accent_color.opacity(0.1)))
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_, _, _window, cx| {
                                        if is_dir {
                                            let h = hero_name.clone();
                                            update_state(|s| {
                                                toggle_node_expansion(&mut s.tree_roots, &h);
                                            });
                                            cx.notify();
                                        } else if let Some(target_hash) = node_hash {
                                            let h = hero_name.clone();
                                            let mut found_system = None;
                                            update_state(|s| {
                                                if let Some(systems) = s.hero_systems.get(&h) {
                                                    if let Some(sys) = systems
                                                        .iter()
                                                        .find(|s| s.hash == target_hash)
                                                    {
                                                        s.selected_hero = Some(h.clone());
                                                        s.selected_system = Some(sys.clone());
                                                        s.active_tab = 0;
                                                        s.working_def = Some(sys.def.clone());
                                                        s.initial_def_backup =
                                                            Some(sys.def.clone());
                                                        found_system = Some(sys.clone());
                                                    }
                                                }
                                            });
                                            clear_all_input_buffers();
                                            cx.notify();

                                            if let Some(sys) = found_system {
                                                let auto = with_state(|s| s.auto_play);
                                                if auto {
                                                    let ron = sys.def_ron;
                                                    spawn_play_ron(cx, ron);
                                                }
                                            }
                                        }
                                    }),
                                )
                                .child(
                                    h_flex()
                                        .gap_1p5()
                                        .items_center()
                                        .child(if node.is_dir {
                                            if node.is_expanded {
                                                IconName::FolderOpen
                                            } else {
                                                IconName::Folder
                                            }
                                        } else {
                                            IconName::Play
                                        })
                                        .child(
                                            div()
                                                .font_medium()
                                                .text_sm()
                                                .truncate()
                                                .child(node.name.clone()),
                                        )
                                        .when(node.is_dir, |this| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .opacity(0.5)
                                                    .child(format!("({})", node.children_count)),
                                            )
                                        })
                                        .when_some(node.hash, |this, h| {
                                            this.child(
                                                div()
                                                    .ml_auto()
                                                    .mr_2()
                                                    .px_1()
                                                    .py_0p5()
                                                    .rounded_sm()
                                                    .bg(if is_selected {
                                                        accent_fg.opacity(0.2)
                                                    } else {
                                                        accent_color.opacity(0.15)
                                                    })
                                                    .text_xs()
                                                    .child(hash_hex(h)),
                                            )
                                        }),
                                )
                        })),
                ),
        )
        .child(
            // ── 右侧工作区 ──
            v_flex()
                .flex_1()
                .h_full()
                .p_3()
                .gap_2()
                // 页头状态栏
                .child(page_header_elem)
                // 错误提示
                .when_some(error.as_ref(), |d, err| {
                    d.child(
                        div()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .bg(danger_color.opacity(0.1))
                            .text_color(danger_color)
                            .text_xs()
                            .child(err.clone()),
                    )
                })
                // 主工作区面板
                .child(
                    v_flex()
                        .flex_1()
                        .w_full()
                        .bg(bg_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_md()
                        .overflow_hidden()
                        .child(right_panel),
                ),
        )
        .into_any_element()
}

// ── 页头：标题 + WS 连接控制 ──

fn render_page_header(
    cx: &mut Context<AppSidebar>,
    ws_url: &str,
    connected: bool,
    auto_play: bool,
) -> AnyElement {
    let theme = cx.theme();
    let (hero, system_name, hash) = with_state(|s| {
        (
            s.selected_hero.clone(),
            s.selected_system.as_ref().map(|x| x.name.clone()),
            s.selected_system.as_ref().map(|x| x.hash),
        )
    });

    h_flex()
        .w_full()
        .justify_between()
        .items_center()
        .px_3()
        .py_2()
        .bg(theme.background)
        .border_1()
        .border_color(theme.border)
        .rounded_md()
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child(IconName::Palette)
                .child(
                    div()
                        .font_bold()
                        .text_base()
                        .child(match (&hero, &system_name) {
                            (Some(h), Some(n)) => format!("{} / {}", h, n),
                            _ => "粒子系统编辑器".to_string(),
                        }),
                )
                .when_some(hash, |this, h| {
                    this.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .bg(theme.accent.opacity(0.2))
                            .rounded_sm()
                            .text_xs()
                            .child(hash_hex(h)),
                    )
                }),
        )
        .child(
            h_flex()
                .gap_2()
                .items_center()
                .child({
                    let label = format!("自动播放: {}", if auto_play { "ON" } else { "OFF" });
                    let btn = Button::new("particle-auto-play-toggle").label(label);
                    let btn = if auto_play { btn } else { btn.ghost() };
                    btn.on_click(cx.listener(|_this, _, _, cx| {
                        update_state(|s| s.auto_play = !s.auto_play);
                        cx.notify();
                    }))
                })
                .child(
                    div()
                        .px_2()
                        .py_1()
                        .rounded_md()
                        .bg(theme.background)
                        .text_xs()
                        .text_color(theme.muted_foreground)
                        .child(format!("Server: {}", ws_url)),
                )
                .when(!connected, |d| {
                    d.child({
                        let url = ws_url.to_string();
                        Button::new("particle-connect-btn")
                            .label("连接服务器")
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                let target_url = url.clone();
                                update_state(|s| {
                                    s.error = None;
                                    s.ws_url = target_url.clone();
                                });
                                cx.notify();

                                let (handle, mut ev_rx) =
                                    particle_service::connect_to_particle_server(&target_url);
                                update_state(|s| {
                                    s.ws_handle = Some(handle);
                                });

                                let weak = cx.entity().downgrade();
                                cx.spawn(
                                    move |_: gpui::WeakEntity<AppSidebar>,
                                          cx: &mut gpui::AsyncApp| {
                                        let mut cx2 = cx.clone();
                                        async move {
                                            while let Some(event) = ev_rx.recv().await {
                                                let notify = process_ws_event(event);
                                                if notify {
                                                    if let Some(e) = weak.upgrade() {
                                                        let _ = e.update(
                                                            &mut cx2,
                                                            |_, cx| cx.notify(),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    },
                                )
                                .detach();
                            }))
                    })
                })
                .when(connected, |d| {
                    d.child(
                        Button::new("particle-disconnect-btn")
                            .icon(IconName::CircleX)
                            .label("断开")
                            .on_click(cx.listener(|_this, _, _, cx| {
                                update_state(|s| {
                                    if let Some(h) = &s.ws_handle {
                                        h.disconnect();
                                    }
                                    s.ws_handle = None;
                                    s.connected = false;
                                });
                                cx.notify();
                            })),
                    )
                })
                .when(connected, |d| {
                    d.child(
                        div()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .bg(theme.accent.opacity(0.15))
                            .text_color(theme.accent)
                            .text_xs()
                            .font_bold()
                            .child("已连接"),
                    )
                }),
        )
        .into_any_element()
}

// ── WS 事件处理 ──

fn process_ws_event(event: ParticleWsEvent) -> bool {
    match event {
        ParticleWsEvent::Connected => {
            update_state(|s| s.connected = true);
            true
        }
        ParticleWsEvent::Disconnected { error } => {
            update_state(|s| {
                s.connected = false;
                s.ws_handle = None;
                if let Some(e) = error {
                    s.error = Some(e);
                }
            });
            true
        }
    }
}
