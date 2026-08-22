//! 提取 worker：独立进程运行 bevy-bound 的提取管线，client 通过 stdio 流式接收进度。
//!
//! 协议：stdout 每行一个 JSON `{"step": u8, "kind": "log"|"status", "msg": string}`，
//! step 与 client `ExtractionStep` 对齐（0=Git 同步 / 1=基础+UI / 2=音频 / 3=Shader / 4=完成）。
//! 退出码 0 = 成功，非 0 = 失败（错误详情写 stderr）。

use std::path::Path;
use std::process::Command;

use clap::Parser;
use league_core::extract::SkinCharacterDataProperties;
use league_to_lol::data::Data;
use league_to_lol::extract::audio::export_audio_for_skin;
use league_to_lol::extract::{
    ExtractOptions, extract_phase_1_create_loader, extract_ui_all, extract_with_options,
};
use lol_base::audio::ConfigAudio;
use ron::ser::{PrettyConfig, to_string_pretty};
use serde::Serialize;

const STEP_GIT: u8 = 0;
const STEP_BASE: u8 = 1;
const STEP_AUDIO: u8 = 2;
const STEP_SHADER: u8 = 3;
const STEP_DONE: u8 = 4;

#[derive(Parser)]
#[command(name = "lol_extractor")]
struct Args {
    /// 英雄联盟 Game 根目录
    #[arg(long)]
    game_path: String,

    /// 输出 assets 目录
    #[arg(long)]
    assets_dir: String,

    /// 跳过地图 Mesh 优化（Fast Mode）
    #[arg(long)]
    skip_map_geo: bool,

    /// 是否提取基础模型/贴图/地图/UI
    #[arg(long)]
    extract_base: bool,

    /// 是否提取 Shader（当前为占位）
    #[arg(long)]
    extract_shaders: bool,

    /// 是否提取全英雄音效
    #[arg(long)]
    extract_audio: bool,
}

#[derive(Serialize)]
struct Progress {
    step: u8,
    kind: &'static str,
    msg: String,
}

fn emit(step: u8, kind: &'static str, msg: impl Into<String>) {
    let line = serde_json::to_string(&Progress {
        step,
        kind,
        msg: msg.into(),
    })
    .unwrap_or_else(|_| "{\"step\":255,\"kind\":\"log\",\"msg\":\"进度序列化失败\"}".to_string());
    println!("{line}");
}

fn log(step: u8, msg: impl Into<String>) {
    emit(step, "log", msg);
}

fn status(step: u8, msg: impl Into<String>) {
    emit(step, "status", msg);
}

fn main() {
    let args = Args::parse();
    let assets_dir = Path::new(&args.assets_dir).to_path_buf();

    if let Err(e) = run(&args, &assets_dir) {
        eprintln!("[lol_extractor] 提取失败: {e}");
        std::process::exit(1);
    }

    status(STEP_DONE, "提取任务已全量完成");
}

fn run(args: &Args, assets_dir: &Path) -> Result<(), String> {
    log(
        STEP_GIT,
        "[PREFLIGHT] 启动提取 Worker 运行时前置环境检测...",
    );
    log(
        STEP_GIT,
        format!("[PREFLIGHT] 游戏数据源路径: {}", args.game_path),
    );
    log(
        STEP_GIT,
        format!("[PREFLIGHT] 资源输出目标: {}", assets_dir.display()),
    );

    let game_path_buf = Path::new(&args.game_path);
    if !game_path_buf.exists() {
        return Err(format!("英雄联盟客户端路径不存在: {}", args.game_path));
    }

    status(STEP_GIT, "正在 Git 同步 CommunityDragon Hash 字典...");
    sync_community_dragon_data(assets_dir)?;

    if args.extract_base {
        extract_base_and_ui(args, assets_dir)?;
    }

    if args.extract_audio {
        extract_audio(args, assets_dir)?;
    }

    if args.extract_shaders {
        extract_shaders_step(args, assets_dir)?;
    }

    Ok(())
}

/// 3. ShaderCache 提取与转译
fn extract_shaders_step(args: &Args, assets_dir: &Path) -> Result<(), String> {
    status(STEP_SHADER, "正在提取并反编译 ShaderCache...");
    log(STEP_SHADER, "[SHADER] 开始探测 dxbc-compiler 工具...");

    let dxbc_compiler = league_to_lol::extract::find_dxbc_compiler(assets_dir).ok_or_else(|| {
        format!(
            "未找到 dxbc-compiler 工具，请确保在 {}/tools/ 或 assets/tools/ 目录下存在 dxbc-compiler.exe",
            assets_dir.display()
        )
    })?;

    log(
        STEP_SHADER,
        format!("[SHADER] 找到编译器: {}", dxbc_compiler.display()),
    );

    let shaders_out_dir = assets_dir.join("shaders");
    let options = league_to_lol::extract::ExtractShaderOptions {
        game_path: Path::new(&args.game_path).to_path_buf(),
        out_dir: shaders_out_dir,
        dxbc_compiler_path: dxbc_compiler,
        toc_paths: Vec::new(),
        skip_existing: false,
        save_dxbc: false,
    };

    league_to_lol::extract::extract_shaders_pipeline(
        &options,
        Some(&|msg: &str| {
            log(STEP_SHADER, msg);
        }),
    )?;

    log(STEP_SHADER, "[SHADER] ShaderCache 提取与转译已全部完成！");
    Ok(())
}

/// 0. 自动同步/克隆 CommunityDragon 哈希解包数据仓库
fn sync_community_dragon_data(assets_dir: &Path) -> Result<(), String> {
    let cd_dir = assets_dir.join("CommunityDragon-Data");
    let repo_url = "https://github.com/CommunityDragon/Data.git";

    if !cd_dir.exists() {
        log(
            STEP_GIT,
            format!("[GIT] 检测到 CommunityDragon-Data 不存在，开始 git clone: {repo_url}"),
        );
        if let Some(parent) = cd_dir.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建 CommunityDragon-Data 父目录失败: {e}"))?;
        }
        let status = Command::new("git")
            .args(["clone", "--depth", "1", repo_url])
            .arg(&cd_dir)
            .status()
            .map_err(|e| format!("git clone 命令启动失败: {e}"))?;
        if status.success() {
            log(STEP_GIT, "[GIT] 成功克隆 CommunityDragon-Data 数据库！");
        } else {
            return Err("git clone CommunityDragon-Data 失败，请检查 Git 环境或网络状况。".into());
        }
    } else {
        log(
            STEP_GIT,
            "[GIT] CommunityDragon-Data 已存在，正在 git pull 最新哈希映射...",
        );
        let status = Command::new("git")
            .arg("pull")
            .current_dir(&cd_dir)
            .status()
            .map_err(|e| format!("git pull 命令启动失败: {e}"))?;
        if status.success() {
            log(
                STEP_GIT,
                "[GIT] CommunityDragon-Data 已成功更新到最新状态。",
            );
        } else {
            log(
                STEP_GIT,
                "[GIT] [WARNING] git pull 未成功，继续使用本地现有 Hash 数据库。",
            );
        }
    }
    Ok(())
}

/// 1. 基础模型/贴图/地图/UI 提取
fn extract_base_and_ui(args: &Args, assets_dir: &Path) -> Result<(), String> {
    status(STEP_BASE, "正在提取基础模型/贴图/地图与 UI 资源...");
    log(
        STEP_BASE,
        "[EXTRACT] 开始提取基础游戏数据 (模型/贴图/地图) 与 UI 资源...",
    );

    let hashes_dir = assets_dir
        .join("CommunityDragon-Data")
        .join("hashes")
        .join("lol");
    let hashes_dir_str = hashes_dir.to_string_lossy().to_string();

    log(STEP_BASE, "[EXTRACT] 提取游戏基本数据...");
    extract_with_options(
        &args.game_path,
        &hashes_dir_str,
        ExtractOptions {
            skip_map_geo: args.skip_map_geo,
        },
    );

    log(STEP_BASE, "[EXTRACT] 提取全套 UI 矢量与纹理资源...");
    extract_ui_all(&args.game_path);

    if !args.skip_map_geo {
        log(STEP_BASE, "[POST] 执行地图几何优化与后处理...");
        post_process_mapgeo(assets_dir);
    }
    Ok(())
}

/// 2. 全英雄音效提取（bnk/wpk -> ww2ogg -> ron）
fn extract_audio(args: &Args, assets_dir: &Path) -> Result<(), String> {
    status(STEP_AUDIO, "正在提取全英雄音效配置...");
    log(
        STEP_AUDIO,
        "[AUDIO] 开始全英雄音效提取 (bnk/wpk -> ww2ogg -> ron)...",
    );

    let loader = extract_phase_1_create_loader(&args.game_path);
    let char_dir = assets_dir.join("characters");
    let mut character_dirs = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&char_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !name.starts_with("SRU")
                    && !name.starts_with("sru")
                    && name != "Locke"
                    && name != "inhibitor"
                    && name != "nexus"
                    && name != "turret"
                    && name != "tftchampion"
                {
                    character_dirs.push(name);
                }
            }
        }
    }
    character_dirs.sort();

    log(
        STEP_AUDIO,
        format!(
            "[AUDIO] 搜索到 {} 个英雄目录，开始提取音效...",
            character_dirs.len()
        ),
    );

    let mut success_count = 0;
    for (idx, champ_dir) in character_dirs.iter().enumerate() {
        let lower_name = champ_dir.to_lowercase();
        let display_name = format!("{}{}", lower_name[..1].to_uppercase(), &lower_name[1..]);

        log(
            STEP_AUDIO,
            format!(
                "[{}/{}] 正在处理音效: {}",
                idx + 1,
                character_dirs.len(),
                display_name
            ),
        );

        let skin_bin_path = format!("data/characters/{}/skins/skin0.bin", lower_name);

        let Ok(skin_prop_group) = loader.get_prop_group_by_paths(vec![&skin_bin_path]) else {
            log(
                STEP_AUDIO,
                format!("  [SKIP] 无法加载 skin bin: {}", skin_bin_path),
            );
            continue;
        };
        let Some(skin_data) = skin_prop_group.get_by_class::<SkinCharacterDataProperties>() else {
            log(
                STEP_AUDIO,
                format!(
                    "  [SKIP] 无法获取 SkinCharacterDataProperties: {}",
                    display_name
                ),
            );
            continue;
        };

        let audio_config: ConfigAudio =
            export_audio_for_skin(&loader, &display_name, "skin0", &skin_data);
        let output_audio_path = assets_dir
            .join("characters")
            .join(champ_dir)
            .join("skins")
            .join("skin0_audio.ron");

        if let Some(parent) = output_audio_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if let Ok(serialized) = to_string_pretty(&audio_config, PrettyConfig::default()) {
            let _ = std::fs::write(&output_audio_path, serialized);
        }

        ensure_audio_bank_in_skin0(assets_dir, champ_dir);
        success_count += 1;
    }

    log(
        STEP_AUDIO,
        format!(
            "[AUDIO] 全英雄音效提取完成: 成功 {}/{}",
            success_count,
            character_dirs.len()
        ),
    );
    Ok(())
}

fn post_process_mapgeo(assets_dir: &Path) {
    let map_name = "sr_seasonal_map";
    let input_glb = assets_dir.join("maps").join(map_name).join("mapgeo.glb");
    let output_gltf = assets_dir.join("maps").join("output.gltf");

    if !input_glb.exists() {
        log(STEP_BASE, "[POST] 未找到 mapgeo.glb，跳过地图几何优化");
        return;
    }

    log(STEP_BASE, "[POST] 运行 gltf-transform webp 优化纹理...");
    let cmd = format!(
        "npx gltf-transform webp \"{}\" \"{}\"",
        input_glb.display(),
        output_gltf.display()
    );

    #[cfg(target_os = "windows")]
    let status = Command::new("cmd").args(["/C", &cmd]).status();
    #[cfg(not(target_os = "windows"))]
    let status = Command::new("sh").args(["-c", &cmd]).status();

    if let Ok(st) = status {
        if st.success() {
            log(STEP_BASE, "[POST] 地图 GLTF 优化完成");
        }
    }
}

fn ensure_audio_bank_in_skin0(assets_dir: &Path, champ_dir: &str) {
    let skin0_path = assets_dir
        .join("characters")
        .join(champ_dir)
        .join("skins")
        .join("skin0.ron");

    let Ok(content) = std::fs::read_to_string(&skin0_path) else {
        return;
    };

    if !content.contains("AudioBank") {
        let replacement = format!(
            "components: [\n        (type: \"AudioBank\", data: (path: \"assets/characters/{}/skins/skin0_audio.ron\")),\n",
            champ_dir
        );
        let new_content = content.replace("components: [", &replacement);
        let _ = std::fs::write(&skin0_path, new_content);
    }
}
