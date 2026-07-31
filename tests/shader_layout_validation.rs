//! # shader_layout_validation
//!
//! map.ron 内部一致性校验：并集布局 vs 变体布局的 binding 编号对齐、
//! set_param 调用点成员名在各家族 BASE 变体布局中的解析情况、
//! 丢失成员名的诊断枚举。
//!
//! 注：与 .spv 直接对标的校验（map_ron_matches_spirv_ground_truth、
//! set_param_data_lands_at_spirv_offsets）已移除——新编译器不再生成 OpName
//! 标签，cbuffer 名字桥接不再可行。

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use league_utils::LeagueShader;
use lol_base_render::shader_layout::{BindingTypeDesc, ShaderLayoutDescriptor};
use serde::Deserialize;

// ---------------------------------------------------------------------------
// map.ron 反序列化目标（Bevy DynamicScene RON 文本）
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct ShaderMapRon {
    entries: HashMap<LeagueShader, BTreeMap<u64, EntryRon>>,
    layouts: Vec<ShaderLayoutDescriptor>,
    unified: HashMap<LeagueShader, ShaderLayoutDescriptor>,
}

#[derive(Deserialize)]
struct EntryRon {
    shader_handle: HandleRon,
    layout_index: u32,
}

/// map.ron 里 `Handle<Shader>` 经 Bevy 反射序列化为 `Path("...")`。
#[derive(Deserialize)]
enum HandleRon {
    Path(String),
}

// ---------------------------------------------------------------------------
// 校验
// ---------------------------------------------------------------------------

/// 从 map.ron 文本里抽出 `"lol_base_render::shader::ShaderMap": ( ... )` 的 `( ... )` 子块。
///
/// 不走 `ron::Value` / untagged 中转：因为 `entries` 的键是 `LeagueShader` 枚举（如
/// `DistortionPs`），ron 0.8 经中间 Value 缓冲时会把裸标识符当作 Unit 而丢失枚举
/// 变体名，直接文本抽块后一次性反序列化则能正确读出。按括号配对提取（字
/// 符串字面量内的括号不计数）。
fn extract_shader_map_block(text: &str) -> &str {
    let key = "\"lol_base_render::shader::ShaderMap\":";
    let start = text.find(key).expect("map.ron 中找不到 ShaderMap 资源");
    let after = &text[start + key.len()..];
    let open = after.find('(').expect("ShaderMap 值缺少 (");
    let bytes = after.as_bytes();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escaped = false;
    let mut end = open;
    for (i, &b) in bytes.iter().enumerate().skip(open) {
        let c = b as char;
        if in_str {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_str = false;
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    &after[open..end]
}

/// 工作区根目录（`cargo test` 运行时 CWD 即根 package 目录）。
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 读取并反序列化 map.ron 里的 ShaderMap 资源块。
fn load_shader_map(root: &Path) -> ShaderMapRon {
    let map_path = root.join("assets/shaders/map.ron");
    let map_text = std::fs::read_to_string(&map_path)
        .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", map_path.display()));
    let block = extract_shader_map_block(&map_text);
    ron::from_str(block).unwrap_or_else(|e| panic!("ShaderMapRon 反序列化失败: {e}"))
}

// ===========================================================================
// 运行时两张表一致性诊断：
// create/as_bind_group 按【并集布局】的 binding_index 分配 blob / 建 GPU buffer，
// 而 write_after_member 按【变体布局】的 binding_index 写 blob。
// 若同名 cbuffer 在两张表里编号不同，数据就写进别的 buffer——无报错的静默失效。
// 同时把代码中全部 set_param 调用的成员名在各家族 BASE 变体布局里逐一解析。
// ===========================================================================

#[test]
fn unified_and_variant_binding_tables_are_consistent() {
    let root = workspace_root();
    let shader_map = load_shader_map(&root);
    let base_hash = league_utils::hash_shader_spec(&Vec::<String>::new());

    let mut report = String::new();
    let mut critical: Vec<String> = Vec::new();

    let mut families: Vec<&LeagueShader> = shader_map.entries.keys().collect();
    families.sort_by_key(|f| format!("{f:?}"));

    // ── 检查①：全部变体（不止 BASE）的每个 UniformBuffer binding，
    // 并集布局必须有同名、同 binding_index 的槽位，且并集 total_size
    // 装得下变体所有成员（blob 按并集尺寸分配）
    for family in &families {
        let Some(unified) = shader_map.unified.get(family) else {
            critical.push(format!("{family:?}: 无并集布局"));
            continue;
        };
        for (hash, entry) in &shader_map.entries[family] {
            let Some(variant) = shader_map.layouts.get(entry.layout_index as usize) else {
                continue;
            };
            for (name, vb) in &variant.bindings {
                let BindingTypeDesc::UniformBuffer { members, .. } = &vb.type_desc else {
                    continue;
                };
                let Some(ub) = unified.bindings.get(name) else {
                    critical.push(format!(
                        "{family:?} #{hash}: 变体有 \"{name}\"（binding {}），但并集布局无此槽位 → blob 不会被分配，set_param 写入落空",
                        vb.binding_index
                    ));
                    continue;
                };
                if ub.binding_index != vb.binding_index {
                    critical.push(format!(
                        "{family:?} #{hash}: \"{name}\" binding 错配 — 并集={} vs 变体={} → 写进别的 buffer",
                        ub.binding_index, vb.binding_index
                    ));
                }
                let BindingTypeDesc::UniformBuffer { total_size: u_total, .. } = &ub.type_desc
                else {
                    critical.push(format!(
                        "{family:?} #{hash}: \"{name}\" 在并集布局里不是 UniformBuffer"
                    ));
                    continue;
                };
                let blob_len = (*u_total).max(16);
                for m in members.values() {
                    if m.offset + m.size > blob_len {
                        critical.push(format!(
                            "{family:?} #{hash}: \"{name}\".\"{}\" 变体 offset {}+size {} 超出并集 blob 尺寸 {blob_len} → 写入被截断",
                            m.name, m.offset, m.size
                        ));
                    }
                }
            }
        }
    }

    // ── 检查②：代码里全部 set_param 成员名在各家族 BASE 变体布局的解析情况
    //（particle.rs / emitters/update.rs / dynamic.rs 的调用点汇总）
    let caller_params: &[(&str, &[LeagueShader], &[&str])] = &[
        (
            "Quad",
            &[LeagueShader::QuadVs, LeagueShader::QuadPs],
            &["mProj", "vCamera", "TEXTURE_INFO"],
        ),
        (
            "QuadSlice",
            &[LeagueShader::QuadVs, LeagueShader::QuadPsSlice],
            &["mProj", "vCamera", "TEXTURE_INFO", "SLICE_RANGE"],
        ),
        (
            "Mesh",
            &[LeagueShader::MeshVs, LeagueShader::MeshPs],
            &[
                "mProj",
                "vCamera",
                "mWorld",
                "vParticleUVTransform",
                "kColorFactor",
                "COLOR_LOOKUP_UV",
            ],
        ),
        (
            "Distortion",
            &[LeagueShader::DistortionVs, LeagueShader::DistortionPs],
            &[
                "mProj",
                "vCamera",
                "TEXTURE_INFO",
                "PARTICLE_DEPTH_PUSH_PULL",
                "AlphaTestReferenceValue",
                "DistortionPower",
            ],
        ),
        (
            "SkinnedMesh",
            &[
                LeagueShader::SkinnedMeshParticleVs,
                LeagueShader::SkinnedMeshParticlePs,
            ],
            &[
                "mProj",
                "vCamera",
                "vParticleUVTransform",
                "kColorFactor",
                "COLOR_LOOKUP_UV",
            ],
        ),
        (
            "UnlitDecal",
            &[LeagueShader::UnlitDecalVs, LeagueShader::UnlitDecalPs],
            &[
                "mProj",
                "vCamera",
                "DECAL_WORLD_TO_UV_MATRIX",
                "DECAL_PROJECTION_Y_RANGE",
                "DECAL_WORLD_MATRIX",
                "MODULATE_COLOR",
                "COLOR_UV",
            ],
        ),
    ];

    for (kind, shaders, params) in caller_params {
        report.push_str(&format!("\n=== {kind} （BASE 变体）===\n"));
        for param in *params {
            let mut found: Vec<String> = Vec::new();
            for shader in *shaders {
                let Some(entry) = shader_map
                    .entries
                    .get(shader)
                    .and_then(|v| v.get(&base_hash))
                else {
                    report.push_str(&format!("  [!] {shader:?} 无 BASE 变体条目\n"));
                    continue;
                };
                let Some(layout) = shader_map.layouts.get(entry.layout_index as usize) else {
                    continue;
                };
                for (cb_name, b) in &layout.bindings {
                    if let BindingTypeDesc::UniformBuffer { members, .. } = &b.type_desc {
                        if let Some(m) = members.get(*param) {
                            found.push(format!(
                                "{shader:?} \"{cb_name}\"@binding{} offset {} size {}",
                                b.binding_index, m.offset, m.size
                            ));
                        }
                    }
                }
            }
            if found.is_empty() {
                report.push_str(&format!(
                    "  [MISS] set_param(\"{param}\") 在 BASE 变体两侧均未命中 → 静默 no-op\n"
                ));
            } else {
                report.push_str(&format!("  [ok] {param} → {}\n", found.join(" | ")));
            }
        }
    }

    report.push_str(&format!("\n两张表错配（致命）：{} 项\n", critical.len()));
    for line in &critical {
        report.push_str("  [CRITICAL] ");
        report.push_str(line);
        report.push('\n');
    }

    let report_path = root.join("target/runtime_binding_consistency_report.txt");
    let _ = std::fs::create_dir_all(root.join("target"));
    let _ = std::fs::write(&report_path, &report);
    println!("{report}");

    assert!(
        critical.is_empty(),
        "发现 {} 项并集/变体两张表错配；详见 {}\n前若干项:\n{}",
        critical.len(),
        report_path.display(),
        critical
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}

// ===========================================================================
// 诊断：枚举变体布局里名字丢失的成员（dxbc-compiler 把部分 $Globals 变量
// 名改成了 "m"，导致 set_param 按原名查表落空），列出受影响的家族×变体。
// ===========================================================================

#[test]
fn report_lost_member_names() {
    let root = workspace_root();
    let shader_map = load_shader_map(&root);

    let mut report = String::new();
    let mut affected = 0usize;

    let mut families: Vec<&LeagueShader> = shader_map.entries.keys().collect();
    families.sort_by_key(|f| format!("{f:?}"));

    for family in families {
        let variants = &shader_map.entries[family];
        let mut hashes: Vec<&u64> = variants.keys().collect();
        hashes.sort();
        for hash in hashes {
            let entry = &variants[hash];
            let HandleRon::Path(rel) = &entry.shader_handle;
            let Some(layout) = shader_map.layouts.get(entry.layout_index as usize) else {
                continue;
            };
            for (cb_name, b) in &layout.bindings {
                if let BindingTypeDesc::UniformBuffer { members, .. } = &b.type_desc {
                    for (name, m) in members {
                        // 可疑丢名：单字符成员名（正常命名如 SLICE_RANGE/mProj 至少 2 字符）
                        if name.len() <= 1 {
                            affected += 1;
                            report.push_str(&format!(
                                "{family:?} layout#{} ({rel}): \"{cb_name}\".\"{name}\" @offset {} size {}\n",
                                entry.layout_index, m.offset, m.size
                            ));
                        }
                    }
                }
            }
        }
    }

    let report_path = root.join("target/lost_member_names_report.txt");
    let _ = std::fs::create_dir_all(root.join("target"));
    let _ = std::fs::write(&report_path, &report);
    println!("丢名成员引用共 {affected} 处，详见 {}", report_path.display());
    println!("{report}");
}
