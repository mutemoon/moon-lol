use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
use league_loader::game::LeagueLoader;
use league_loader::prop_bin::LeagueWadLoaderTrait;

use crate::components::sidebar::AppSidebar;
use crate::services::prop_ron::{convert_prop_bytes_to_ron, get_global_game_hashes};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
    pub hash: Option<u64>,
    pub children: Vec<TreeNode>,
}

#[derive(Debug, Clone)]
pub struct FlatNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub has_children: bool,
    pub hash: Option<u64>,
}

pub struct WadBrowserState {
    pub search_query: String,
    pub tree_roots: Vec<TreeNode>,
    pub selected_file: Option<PathBuf>,
    pub status_message: Option<String>,
    pub file_size_bytes: usize,
    pub editor_state: Option<Entity<InputState>>,
    pub pending_ron: Option<(String, usize)>,
    pub is_initialized: bool,
    pub is_scanning: bool,
    pub is_parsing: bool,
    pub loader: Option<Arc<LeagueLoader>>,
}

impl Default for WadBrowserState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            tree_roots: Vec::new(),
            selected_file: None,
            status_message: None,
            file_size_bytes: 0,
            editor_state: None,
            pending_ron: None,
            is_initialized: false,
            is_scanning: false,
            is_parsing: false,
            loader: None,
        }
    }
}

thread_local! {
    static WAD_BROWSER_STATE: RefCell<WadBrowserState> = RefCell::new(WadBrowserState::default());
}

pub fn render_wad_browser(
    _sidebar: &AppSidebar,
    window: &mut Window,
    cx: &mut Context<AppSidebar>,
) -> AnyElement {
    let should_start_scan = WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        if !state.is_initialized && !state.is_scanning {
            state.is_initialized = true;
            state.is_scanning = true;
            true
        } else {
            false
        }
    });

    // 首次进入时，在后台异步加载 WAD Header 与 Game Hashes 并构建虚拟文件树结构
    if should_start_scan {
        start_async_wad_scan(cx);
    }

    let (
        flat_visible_nodes,
        total_files,
        selected,
        query,
        status,
        size_bytes,
        _editor,
        is_scanning,
        is_parsing,
        pending_ron,
    ) = WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        let pending = state.pending_ron.take();
        let mut flat = Vec::new();
        let query_lower = state.search_query.trim().to_lowercase();
        collect_flat_visible_nodes(&state.tree_roots, 0, &query_lower, &mut flat);
        let total_files = count_files(&state.tree_roots);
        (
            flat,
            total_files,
            state.selected_file.clone(),
            state.search_query.clone(),
            state.status_message.clone(),
            state.file_size_bytes,
            state.editor_state.clone(),
            state.is_scanning,
            state.is_parsing,
            pending,
        )
    });

    // 在 render 帧安全使用传进来的 window 句柄更新/新建 InputState
    if let Some((ron_str, size)) = pending_ron {
        WAD_BROWSER_STATE.with(|cell| {
            let mut state = cell.borrow_mut();
            state.file_size_bytes = size;
            if let Some(ed) = &state.editor_state {
                ed.update(cx, |input_state, cx| {
                    input_state.set_value(ron_str, window, cx);
                });
            } else {
                let ed = cx.new(|cx| {
                    InputState::new(window, cx)
                        .code_editor("ron")
                        .line_number(true)
                        .searchable(true)
                        .multi_line(true)
                        .default_value(&ron_str)
                });
                state.editor_state = Some(ed);
            }
        });
    }

    let editor = WAD_BROWSER_STATE.with(|cell| cell.borrow().editor_state.clone());

    let theme = cx.theme();

    h_flex()
        .w_full()
        .h_full()
        .gap_3()
        .child(
            // ── 左侧文件树侧边栏 ──
            v_flex()
                .w_80()
                .h_full()
                .p_3()
                .gap_2()
                .bg(theme.sidebar)
                .border_r_1()
                .border_color(theme.border)
                .child(
                    h_flex()
                        .w_full()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(div().font_bold().text_base().child("WAD 虚拟树"))
                                .when(is_scanning, |this| {
                                    this.child(
                                        div().text_xs().opacity(0.6).child("(加载 WAD 头...)"),
                                    )
                                }),
                        )
                        .child(
                            Button::new("refresh-props-tree")
                                .icon(IconName::Redo)
                                .ghost()
                                .small()
                                .on_click(cx.listener(|_, _, _window, cx| {
                                    start_async_wad_scan(cx);
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
                                .bg(theme.background)
                                .border_1()
                                .border_color(theme.border)
                                .rounded_md()
                                .items_center()
                                .gap_2()
                                .child(IconName::Search)
                                .child(div().flex_1().text_sm().child(if query.is_empty() {
                                    "搜索虚拟树节点...".into()
                                } else {
                                    query.clone()
                                })),
                        )
                        .child(
                            div()
                                .px_1()
                                .text_xs()
                                .text_color(theme.muted_foreground)
                                .child(if query.is_empty() {
                                    format!("共 {} 个虚拟文件", total_files)
                                } else {
                                    let matched_count =
                                        flat_visible_nodes.iter().filter(|n| !n.is_dir).count();
                                    format!("匹配 {} 个虚拟文件", matched_count)
                                }),
                        ),
                )
                // 树状列表区域（只挂载当前展开显示的节点 DOM）
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
                                    .child("读取 WAD 头与 Hash 映射中..."),
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
                                    .child("无匹配文件节点"),
                            )
                        })
                        .children(flat_visible_nodes.into_iter().map(|node| {
                            let is_selected = selected.as_ref() == Some(&node.path);
                            let path_clone = node.path.clone();
                            let is_dir = node.is_dir;
                            let hash = node.hash;

                            // 缩进像素
                            let padding_left_px = (node.depth * 14 + 6) as f32;

                            div()
                                .w_full()
                                .py_1()
                                .rounded_md()
                                .cursor_pointer()
                                .pl(px(padding_left_px))
                                .when(is_selected, |this| {
                                    this.bg(theme.accent).text_color(theme.accent_foreground)
                                })
                                .when(!is_selected, |this| {
                                    this.hover(|style| style.bg(theme.accent.opacity(0.1)))
                                })
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_, _, _window, cx| {
                                        let p = path_clone.clone();
                                        if is_dir {
                                            WAD_BROWSER_STATE.with(|cell| {
                                                let mut state = cell.borrow_mut();
                                                toggle_node_expansion(&mut state.tree_roots, &p);
                                            });
                                            cx.notify();
                                        } else if let Some(h) = hash {
                                            start_async_file_parse_from_wad(&p, h, cx);
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
                                            IconName::File
                                        })
                                        .child(
                                            div().font_medium().text_sm().child(node.name.clone()),
                                        ),
                                )
                        })),
                ),
        )
        .child(
            // ── 右侧 Code Editor 工作区 ──
            v_flex()
                .flex_1()
                .h_full()
                .p_3()
                .gap_2()
                .child(
                    // 顶部状态栏
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
                                .child(IconName::File)
                                .child(
                                    div().font_bold().text_base().child(
                                        selected
                                            .as_ref()
                                            .map(|p| p.to_string_lossy().to_string())
                                            .unwrap_or_else(|| "未选择文件".to_string()),
                                    ),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_0p5()
                                        .bg(theme.accent.opacity(0.2))
                                        .rounded_sm()
                                        .text_xs()
                                        .child("WAD -> PROP -> RON"),
                                ),
                        )
                        .child(
                            h_flex()
                                .gap_3()
                                .items_center()
                                .when(is_parsing, |this| {
                                    this.child(
                                        div()
                                            .text_xs()
                                            .opacity(0.7)
                                            .child("从 WAD 按需解包并转译 RON 中..."),
                                    )
                                })
                                .child(div().text_xs().opacity(0.8).child(format!(
                                    "解包大小: {:.2} KB",
                                    size_bytes as f64 / 1024.0
                                )))
                                .when_some(status.clone(), |this, msg| {
                                    this.child(div().text_xs().text_color(theme.danger).child(msg))
                                }),
                        ),
                )
                // 编辑器区域
                .child(
                    v_flex()
                        .flex_1()
                        .w_full()
                        .bg(theme.background)
                        .border_1()
                        .border_color(theme.border)
                        .rounded_md()
                        .overflow_hidden()
                        .child(if is_parsing {
                            div()
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .opacity(0.6)
                                .child("正在按需从 WAD 读取 payload 并反序列化...")
                                .into_any_element()
                        } else if let Some(ed) = editor {
                            Input::new(&ed).h_full().into_any_element()
                        } else {
                            div()
                                .w_full()
                                .h_full()
                                .flex()
                                .items_center()
                                .justify_center()
                                .opacity(0.6)
                                .child("请在左侧展开树节点并点击要读取的虚拟文件...")
                                .into_any_element()
                        }),
                ),
        )
        .into_any_element()
}

/// 从 WAD 列表与 Game Hashes 映射构建层级树（只构建路径与 entry hash 映射，不读取 payload）
fn build_wad_tree(loader: &LeagueLoader) -> Vec<TreeNode> {
    let game_hashes = get_global_game_hashes();
    let mut seen = HashSet::new();

    let mut path_hashes: Vec<(PathBuf, u64)> = Vec::new();
    for wad in &loader.wads {
        for hash in wad.wad.entries.keys() {
            if !seen.insert(*hash) {
                continue;
            }
            if let Some(file_path_str) = game_hashes.get(hash) {
                let path = PathBuf::from(file_path_str);
                path_hashes.push((path, *hash));
            } else {
                let path = PathBuf::from(format!("unknown_hashes/0x{:016x}.bin", hash));
                path_hashes.push((path, *hash));
            }
        }
    }

    // 针对每个 PathBuf 组装多层级 TreeNode
    let mut root_children: Vec<TreeNode> = Vec::new();

    for (full_path, hash) in path_hashes {
        let components: Vec<String> = full_path
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();
        if components.is_empty() {
            continue;
        }

        insert_path_into_tree(&mut root_children, &components, 0, PathBuf::new(), hash);
    }

    sort_nodes(&mut root_children);
    root_children
}

fn insert_path_into_tree(
    nodes: &mut Vec<TreeNode>,
    components: &[String],
    comp_idx: usize,
    parent_path: PathBuf,
    hash: u64,
) {
    if comp_idx >= components.len() {
        return;
    }

    let comp_name = &components[comp_idx];
    let is_last = comp_idx == components.len() - 1;
    let current_path = parent_path.join(comp_name);

    if is_last {
        nodes.push(TreeNode {
            name: comp_name.clone(),
            path: current_path,
            is_dir: false,
            is_expanded: false,
            hash: Some(hash),
            children: Vec::new(),
        });
    } else {
        let existing = nodes
            .iter_mut()
            .find(|n| n.is_dir && n.name.eq_ignore_ascii_case(comp_name));
        if let Some(dir_node) = existing {
            insert_path_into_tree(
                &mut dir_node.children,
                components,
                comp_idx + 1,
                current_path,
                hash,
            );
        } else {
            let mut new_dir = TreeNode {
                name: comp_name.clone(),
                path: current_path.clone(),
                is_dir: true,
                is_expanded: false,
                hash: None,
                children: Vec::new(),
            };
            insert_path_into_tree(
                &mut new_dir.children,
                components,
                comp_idx + 1,
                current_path,
                hash,
            );
            nodes.push(new_dir);
        }
    }
}

fn sort_nodes(nodes: &mut [TreeNode]) {
    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    for node in nodes.iter_mut() {
        if node.is_dir {
            sort_nodes(&mut node.children);
        }
    }
}

/// 切换树节点的折叠/展开状态
fn toggle_node_expansion(nodes: &mut [TreeNode], target_path: &Path) -> bool {
    for node in nodes.iter_mut() {
        if node.path == target_path {
            if node.is_dir {
                node.is_expanded = !node.is_expanded;
            }
            return true;
        }
        if node.is_dir && target_path.starts_with(&node.path) {
            if toggle_node_expansion(&mut node.children, target_path) {
                return true;
            }
        }
    }
    false
}

fn count_files(nodes: &[TreeNode]) -> usize {
    let mut total = 0;
    for node in nodes {
        if node.is_dir {
            total += count_files(&node.children);
        } else {
            total += 1;
        }
    }
    total
}

/// 平铺处于展开显示状态的树节点（折叠的节点绝对不占用 DOM 资源）
fn collect_flat_visible_nodes(
    nodes: &[TreeNode],
    depth: usize,
    query_lower: &str,
    acc: &mut Vec<FlatNode>,
) {
    let is_empty_query = query_lower.is_empty();
    for node in nodes {
        let name_matches = is_empty_query || node.name.to_lowercase().contains(query_lower);

        if node.is_dir {
            let child_matches = !is_empty_query && has_matching_child(node, query_lower);
            let should_show = name_matches || child_matches;

            if should_show {
                acc.push(FlatNode {
                    name: node.name.clone(),
                    path: node.path.clone(),
                    is_dir: true,
                    depth,
                    is_expanded: node.is_expanded || child_matches,
                    has_children: !node.children.is_empty(),
                    hash: None,
                });

                if node.is_expanded || child_matches {
                    collect_flat_visible_nodes(&node.children, depth + 1, query_lower, acc);
                }
            }
        } else if name_matches {
            acc.push(FlatNode {
                name: node.name.clone(),
                path: node.path.clone(),
                is_dir: false,
                depth,
                is_expanded: false,
                has_children: false,
                hash: node.hash,
            });
        }
    }
}

fn has_matching_child(node: &TreeNode, query_lower: &str) -> bool {
    for child in &node.children {
        if child.name.to_lowercase().contains(query_lower) {
            return true;
        }
        if child.is_dir && has_matching_child(child, query_lower) {
            return true;
        }
    }
    false
}

fn start_async_wad_scan(cx: &mut Context<AppSidebar>) {
    WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.is_scanning = true;
    });
    cx.notify();

    let weak_entity = cx.entity().downgrade();
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let res = crate::services::runtime::tokio_runtime()
                .spawn_blocking(move || {
                    let game_dir = r"D:\WeGameApps\英雄联盟\Game";
                    let loader = LeagueLoader::full(game_dir).ok();
                    let roots = if let Some(ref loader) = loader {
                        build_wad_tree(loader)
                    } else {
                        Vec::new()
                    };
                    (loader, roots)
                })
                .await;
            let (loader, roots) = res.unwrap_or((None, Vec::new()));

            let _ = weak_entity.update(&mut cx, |_, cx| {
                WAD_BROWSER_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    state.loader = loader.map(Arc::new);
                    state.tree_roots = roots;
                    state.is_scanning = false;
                });
                cx.notify();
            });
        }
    })
    .detach();
}

fn start_async_file_parse_from_wad(file_path: &Path, hash: u64, cx: &mut Context<AppSidebar>) {
    let p = file_path.to_path_buf();
    let loader_opt = WAD_BROWSER_STATE.with(|cell| cell.borrow().loader.clone());

    WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.selected_file = Some(p.clone());
        state.is_parsing = true;
        state.status_message = None;
    });
    cx.notify();

    let weak_entity = cx.entity().downgrade();

    cx.spawn(move |_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let res = crate::services::runtime::tokio_runtime()
                .spawn_blocking(move || {
                    let loader = loader_opt.ok_or_else(|| "LeagueLoader 未就绪".to_string())?;
                    let mut reader = loader
                        .get_wad_entry_reader_by_hash(hash)
                        .map_err(|e| format!("WAD entry 读取失败: {:?}", e))?;
                    let mut bytes = Vec::new();
                    std::io::Read::read_to_end(&mut reader, &mut bytes)
                        .map_err(|e| format!("读取 payload 失败: {}", e))?;
                    let len = bytes.len();
                    let ron_str = convert_prop_bytes_to_ron(&bytes)?;
                    Ok((ron_str, len))
                })
                .await
                .unwrap_or_else(|_| Err("后台读取线程中断".to_string()));

            let _ = weak_entity.update(&mut cx, |_, cx| {
                WAD_BROWSER_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    state.is_parsing = false;
                    match res {
                        Ok((ron_str, size_bytes)) => {
                            state.pending_ron = Some((ron_str, size_bytes));
                            state.status_message = None;
                        }
                        Err(err) => {
                            state.status_message = Some(err);
                        }
                    }
                });
                cx.notify();
            });
        }
    })
    .detach();
}
