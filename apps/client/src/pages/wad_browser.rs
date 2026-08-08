use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::{h_flex, v_flex, ActiveTheme, IconName, Sizable, StyledExt};
use rayon::prelude::*;

use crate::components::sidebar::AppSidebar;
use crate::services::prop_ron::convert_prop_file_async;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_expanded: bool,
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

    // 首次进入时，在后台 Rayon 多线程异步构建文件树
    if should_start_scan {
        let weak_entity = cx.entity().downgrade();
        cx.spawn(|_this, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                let (tx, rx) = tokio::sync::oneshot::channel();
                std::thread::spawn(move || {
                    let roots = scan_dir_rayon(Path::new("assets/props"));
                    let _ = tx.send(roots);
                });
                let roots = rx.await.unwrap_or_default();

                let _ = weak_entity.update(&mut cx, |_, cx| {
                    WAD_BROWSER_STATE.with(|cell| {
                        let mut state = cell.borrow_mut();
                        state.tree_roots = roots;
                        state.is_scanning = false;
                    });
                    cx.notify();
                });
            }
        })
        .detach();
    }

    let (
        tree_roots,
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
        (
            state.tree_roots.clone(),
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

    // 收集平铺的可见树节点，折叠部分的子节点绝对不生成 DOM，严格控制 DOM 数量
    let mut flat_visible_nodes = Vec::new();
    collect_flat_visible_nodes(&tree_roots, 0, &query, &mut flat_visible_nodes);

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
                                .child(div().font_bold().text_base().child("PROP / WAD 树"))
                                .when(is_scanning, |this| {
                                    this.child(div().text_xs().opacity(0.6).child("(扫描中...)"))
                                }),
                        )
                        .child(
                            Button::new("refresh-props-tree")
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
                            "搜索树节点...".into()
                        } else {
                            query.clone()
                        })),
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
                                    .child("Rayon 多线程扫描目录中..."),
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
                                        } else {
                                            start_async_file_parse(&p, cx);
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
                                        .child("PROP -> RON"),
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
                                            .child("Rayon 多线程转译 RON 中..."),
                                    )
                                })
                                .child(
                                    div().text_xs().opacity(0.8).child(format!(
                                        "大小: {:.2} KB",
                                        size_bytes as f64 / 1024.0
                                    )),
                                )
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
                                .child("正在多线程解包并反序列化 PROP -> RON...")
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
                                .child("请在左侧展开树节点并选择 PROP 文件...")
                                .into_any_element()
                        }),
                ),
        )
        .into_any_element()
}

/// Rayon 多线程并行扫描文件树
fn scan_dir_rayon(dir: &Path) -> Vec<TreeNode> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();

    let mut nodes: Vec<TreeNode> = paths
        .into_par_iter()
        .map(|path| {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if path.is_dir() {
                let children = scan_dir_rayon(&path);
                TreeNode {
                    name,
                    path,
                    is_dir: true,
                    is_expanded: false,
                    children,
                }
            } else {
                TreeNode {
                    name,
                    path,
                    is_dir: false,
                    is_expanded: false,
                    children: Vec::new(),
                }
            }
        })
        .collect();

    nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    nodes
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

/// 平铺处于展开显示状态的树节点（折叠的节点绝对不占用 DOM 资源）
fn collect_flat_visible_nodes(
    nodes: &[TreeNode],
    depth: usize,
    query: &str,
    acc: &mut Vec<FlatNode>,
) {
    let query_lower = query.trim().to_lowercase();
    for node in nodes {
        let name_lower = node.name.to_lowercase();
        let name_matches = query_lower.is_empty() || name_lower.contains(&query_lower);

        if node.is_dir {
            let child_matches = !query_lower.is_empty() && has_matching_child(node, &query_lower);
            let should_show = name_matches || child_matches;

            if should_show {
                acc.push(FlatNode {
                    name: node.name.clone(),
                    path: node.path.clone(),
                    is_dir: true,
                    depth,
                    is_expanded: node.is_expanded || child_matches,
                    has_children: !node.children.is_empty(),
                });

                if node.is_expanded || child_matches {
                    collect_flat_visible_nodes(&node.children, depth + 1, query, acc);
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

fn start_async_rescan(cx: &mut Context<AppSidebar>) {
    WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.is_scanning = true;
    });
    cx.notify();

    let weak_entity = cx.entity().downgrade();
    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let (tx, rx) = tokio::sync::oneshot::channel();
            std::thread::spawn(move || {
                let roots = scan_dir_rayon(Path::new("assets/props"));
                let _ = tx.send(roots);
            });
            let roots = rx.await.unwrap_or_default();

            let _ = weak_entity.update(&mut cx, |_, cx| {
                WAD_BROWSER_STATE.with(|cell| {
                    let mut state = cell.borrow_mut();
                    state.tree_roots = roots;
                    state.is_scanning = false;
                });
                cx.notify();
            });
        }
    })
    .detach();
}

fn start_async_file_parse(file_path: &Path, cx: &mut Context<AppSidebar>) {
    let p = file_path.to_path_buf();
    WAD_BROWSER_STATE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.selected_file = Some(p.clone());
        state.is_parsing = true;
        state.status_message = None;
    });
    cx.notify();

    let weak_entity = cx.entity().downgrade();

    cx.spawn(|_this, cx: &mut AsyncApp| {
        let mut cx = cx.clone();
        async move {
            let res = convert_prop_file_async(p).await;

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
