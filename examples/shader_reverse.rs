use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use autoagents::core::agent::prebuilt::executor::{ReActAgent, ReActAgentOutput};
use autoagents::core::agent::task::Task;
use autoagents::core::agent::{self, AgentDeriveT, BaseAgent, DirectAgent};
use autoagents::llm::backends::ollama::Ollama;
use autoagents::llm::backends::openai::OpenAI;
use autoagents::llm::backends::openrouter::OpenRouter;
use autoagents::llm::builder::LLMBuilder;
use autoagents::llm::LLMProvider;
use autoagents::prelude::{AgentBuilder, AgentOutputT, SlidingWindowMemory};
use autoagents_derive::{agent, AgentHooks, AgentOutput};
use glsl_lang_pp::processor::event::{DirectiveKind, Event};
use glsl_lang_pp::processor::fs::StdProcessor;
use glsl_lang_pp::processor::nodes::{Define, DefineObject};
use glsl_lang_pp::processor::ProcessorState;
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{registry, EnvFilter};
use walkdir::WalkDir;

// ==========================================
// 1. 数据结构定义
// ==========================================

/// 合并 Agent 的输出结果 (已移除 defines 字段)
#[derive(Debug, Serialize, Deserialize, AgentOutput, Clone)]
pub struct MergeShaderOutput {
    #[output(description = "包含 #ifdefs 逻辑的新合并恢复中的 shader 源代码。")]
    pub source_code: String,

    #[output(description = "解释做了哪些更改。")]
    pub explanation: String,
}

impl From<ReActAgentOutput> for MergeShaderOutput {
    fn from(output: ReActAgentOutput) -> Self {
        let resp = output.response;
        match serde_json::from_str::<MergeShaderOutput>(&resp) {
            Ok(value) => value,
            Err(e) => {
                info!("解析 JSON 失败: {} - 错误: {}", resp, e);
                // Fallback for parsing failure
                MergeShaderOutput {
                    source_code: String::new(),
                    explanation: format!("解析 JSON 失败: {} (错误: {})", resp, e),
                }
            }
        }
    }
}

/// 对比 Agent 的输出结果
#[derive(Debug, Serialize, Deserialize, AgentOutput, Clone)]
pub struct CompareShaderOutput {
    #[output(description = "指示两个 shader 是否在计算上等效的布尔值。")]
    pub is_equivalent: bool,

    #[output(description = "如果有差异，详细分析差异，或者确认等效。")]
    pub analysis: String,
}

impl From<ReActAgentOutput> for CompareShaderOutput {
    fn from(output: ReActAgentOutput) -> Self {
        match serde_json::from_str::<CompareShaderOutput>(&output.response) {
            Ok(value) => value,
            Err(_) => CompareShaderOutput {
                is_equivalent: false,
                analysis: output.response,
            },
        }
    }
}

// ==========================================
// 2. Agent 定义
// ==========================================

#[agent(
    name = "shader_merger",
    description = "你是一位 GLSL 逆向工程专家。你的工作是使用预处理指令（#ifdef, #if）将 <恢复中的 shader> 和 <条件编译后的目标 shader> 合并为一个文件。",
    output = MergeShaderOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct ShaderMergerAgent {}

#[agent(
    name = "shader_verifier",
    description = "你是一位 GLSL 静态分析专家。比较两个 shader 。忽略变量重命名（例如 _10 vs _55）和空白。专注于 AST 结构和计算逻辑。",
    output = CompareShaderOutput,
)]
#[derive(Default, Clone, AgentHooks)]
pub struct ShaderVerifierAgent {}

// ==========================================
// 3. 核心逻辑与工具函数
// ==========================================

/// 用户提供的预处理函数
pub fn preprocess_glsl(source: &str, conditions: &[String]) -> String {
    // 适配类型: 将 String slice 转换为 &str
    let cond_str_refs: Vec<&str> = conditions.iter().map(|s| s.as_str()).collect();

    let mut processor = StdProcessor::default();
    let parsed = processor.parse_source(source, "input.glsl".as_ref());
    let mut state_builder = ProcessorState::builder();

    for cond in cond_str_refs {
        let parts: Vec<&str> = cond.splitn(2, ' ').collect();
        let name = parts[0];
        let value = if parts.len() > 1 { parts[1] } else { "1" };

        if let Ok(obj) = DefineObject::from_str(value) {
            state_builder = state_builder.definition(Define::object(name.into(), obj, false));
        }
    }

    let state = state_builder.finish();
    let mut output = String::new();

    for event in parsed.process(state) {
        if let Ok(event) = event {
            match event {
                Event::Token { token, masked, .. } => {
                    if !masked {
                        output.push_str(token.text());
                    }
                }
                Event::Directive { directive, masked } => {
                    if !masked {
                        match directive.kind() {
                            DirectiveKind::Version(_)
                            | DirectiveKind::Extension(_)
                            | DirectiveKind::Pragma(_)
                            | DirectiveKind::Line(_) => {
                                output.push_str(&directive.to_string());
                            }
                            _ => output.push('\n'),
                        }
                    }
                }
                _ => {}
            }
        }
    }
    output
}

async fn create_agent<A: AgentDeriveT + agent::AgentHooks>(
    llm: Arc<dyn LLMProvider>,
    agent_impl: A,
) -> Result<BaseAgent<ReActAgent<A>, DirectAgent>> {
    let sliding_window_memory = Box::new(SlidingWindowMemory::new(10));

    AgentBuilder::<_, DirectAgent>::new(ReActAgent::new(agent_impl))
        .llm(llm)
        .memory(sliding_window_memory)
        .build()
        .await
        .map_err(|e| anyhow!("构建 Agent 失败: {:?}", e))
        .map(|agent| agent.agent)
}

fn get_sorted_files(dir: &str, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == extension))
        .map(|e| e.path().to_path_buf())
        .collect();

    files.retain(|p| {
        !p.file_stem()
            .map_or(false, |s| s.to_string_lossy().contains("BASE"))
    });

    files.sort_by(|a, b| {
        let len_ord = a
            .file_name()
            .map_or(0, |s| s.len())
            .cmp(&b.file_name().map_or(0, |s| s.len()));
        if len_ord == Ordering::Equal {
            a.cmp(b)
        } else {
            len_ord
        }
    });

    Ok(files)
}

fn save_checkpoint(step: usize, filename: &str, content: &str) -> Result<()> {
    let dir = Path::new("assets/shaders_reverse_history");
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let file_path = dir.join(format!("{}_{}_step_{}.glsl", timestamp, filename, step));
    fs::write(file_path, content)?;
    Ok(())
}

fn extract_defines_from_filename(file_path: &Path) -> Result<Vec<String>> {
    let filename_str = file_path
        .file_stem()
        .context("File stem not found")?
        .to_string_lossy();

    // 假设 defines 之间用 '__' 分隔
    let defines: Vec<String> = filename_str.split("__").map(|s| s.to_uppercase()).collect();

    if defines.is_empty() {
        return Err(anyhow!("无法从文件名中提取 defines: {}", filename_str));
    }
    Ok(defines)
}

// 抽取单个文件的处理逻辑以减少主循环缩进
async fn process_single_file(
    file_path: &PathBuf,
    target_defines: &[String], // 新增参数
    current_master: &mut String,
    merger: &mut BaseAgent<ReActAgent<ShaderMergerAgent>, DirectAgent>,
    verifier: &mut BaseAgent<ReActAgent<ShaderVerifierAgent>, DirectAgent>,
) -> Result<()> {
    let target_name = file_path.file_name().unwrap().to_string_lossy();
    info!("正在处理目标: {}", target_name);

    let target_content = fs::read_to_string(file_path)?;
    let mut retry_count = 0;
    let max_retries = 3;
    let mut feedback = String::new();

    loop {
        if retry_count >= max_retries {
            return Err(anyhow!("{} 达到最大重试次数", target_name));
        }

        let defines_str = target_defines.join(", ");

        let feedback_str = if feedback.is_empty() {
            "".to_string()
        } else {
            format!("--- 上次失败反馈 (如果有) ---\n{}", feedback)
        };

        // 1. 合并请求
        let merge_prompt = format!(
            "你是一个 GLSL shader 合并专家。你的任务是将一个未见过的 <条件编译后的目标 shader> 的功能合并到一个现有的 <恢复中的 shader> 中，通过使用预处理指令（#ifdef, #else, #endif）\n\
            \n\
            **目标**: \n\
            创建一个新的恢复中的 shader，经过条件编译 (Defines: {defines_str}) 后，它与 <条件编译后的目标 shader> 之间，除了变量名或 AST 结构可以不同，它的计算逻辑必须与 <条件编译后的目标 shader> 完全相同\n\
            \n\
            **关键指令**: \n\
            - **识别差异**: 仔细比较两个 shader 的计算逻辑\n\
            - **使用预处理指令**: 在逻辑发生变化的地方使用预处理指令\n\
            - **忽略重命名**: <条件编译后的目标 shader> 中的变量名（如 `_10`, `_55`）是自动生成的，可能与恢复中的 shader 不同。如果计算逻辑相同，请使用恢复中的 shader 的变量名，不要因为变量名不同而创建分支\n\
            - **最小化更改**: 尽可能保留恢复中的 shader 的结构。只在必要时插入分支\n\
            - **结构体字段**: 如果 <条件编译后的目标 shader> 的结构体（如 `UniformsVertex`）有不同字段，使用 `#ifdef` 处理结构体定义\n\
            - **输出变量**: 注意 `out` 变量（如 `TEXCOORD0`, `TEXCOORD1`）的布局和赋值差异\n\
            \n\
            **输入数据**: \n\
            --- 恢复中的 shader ---\n\
            {current_master}\n\n\
            --- 条件编译后的目标 shader (Defines: {defines_str}) ---\n\
            {target_content}\n\n\
            {feedback_str}",
        );

        let start_time = std::time::Instant::now();

        if let Some(mem) = merger.memory() {
            let mut mem = mem.lock().await;
            mem.clear().await?;
        }

        let merge_output: MergeShaderOutput = merger
            .run(Task::new(&merge_prompt))
            .await
            .context("合并 Agent 执行失败")?;

        info!("合并耗时: {:?}", start_time.elapsed());

        if merge_output.source_code.trim().is_empty() {
            retry_count += 1;
            feedback = "合并 Agent 返回了空的 source_code。请提供有效的合并源代码。".to_string();
            continue;
        }

        // 2. 预处理验证
        // 使用从文件名中获取的 defines 进行预处理
        let preprocessed = preprocess_glsl(&merge_output.source_code, target_defines);

        let start_time = std::time::Instant::now();

        // 3. 对比请求
        let verify_prompt = format!(
            "你是一个 GLSL 静态分析专家。你的任务是验证两个 shader 代码片段在数学和计算上是否**完全等效**\n\
            \n\
            **比较对象**: \n\
            1. **预处理候选者**: 这是合并后的恢复中的 shader 在应用了 `{defines_str}` 宏之后的结果\n\
            2. **原始目标**: 这是我们要复制的<条件编译后的目标 shader> \n\
            \n\
            **等效性标准**: \n\
            - **语义一致性**: 对于相同的输入（Uniforms, Attributes），两个代码必须产生完全相同的输出（`gl_Position` 和所有 `out` 变量）\n\
            - **忽略变量名**: 变量名（如 `_a` vs `_b`）无关紧要\n\
            - **忽略格式**: 空格、换行、注释无关紧要\n\
            - **忽略死代码**: 不影响输出的计算无关紧要\n\
            - **关注数学**: 重点比较数学公式、控制流（if/else）和数据流\n\
            \n\
            **分析步骤**: \n\
            1. 识别两个代码中的所有输出变量\n\
            2. 对于每个输出，回溯其计算公式\n\
            3. 比较公式是否数学上等价（例如 `a * b` 等于 `b * a`）\n\
            4. 如果发现不匹配，请具体指出是哪个输出变量、在哪一行、有什么样的逻辑差异\n\
            \n\
            **输入数据**: \n\
            --- 预处理候选者 (已应用 Defines: {defines_str}) ---\n\
            {preprocessed}\n\n\
            --- 原始目标 ---\n\
            {target_content}",
        );

        if let Some(mem) = verifier.memory() {
            let mut mem = mem.lock().await;
            mem.clear().await?;
        }

        let verify_output: CompareShaderOutput = verifier
            .run(Task::new(&verify_prompt))
            .await
            .context("验证 Agent 执行失败")?;

        info!("验证耗时: {:?}", start_time.elapsed());

        if verify_output.is_equivalent {
            info!("✅ 成功合并 {}", target_name);
            *current_master = merge_output.source_code;
            break;
        } else {
            info!(
                "❌ 第 {} 次尝试不匹配: {}",
                retry_count + 1,
                verify_output.analysis
            );
            feedback = format!("上次尝试未通过等效性检查: {}", verify_output.analysis);
            retry_count += 1;
        }
    }
    Ok(())
}

// ==========================================
// 4. 主程序
// ==========================================

#[tokio::main]
async fn main() -> Result<()> {
    registry()
        .with(layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // 初始化 LLM
    let llm: Arc<OpenAI> = LLMBuilder::<OpenAI>::new()
        .api_key("sk-ezbetzmomyheddrvoezurkxqxmdwngftofksekuyrboyebkp")
        .model("Qwen/Qwen2.5-Coder-7B-Instruct")
        .base_url("https://api.siliconflow.cn/v1")
        .temperature(0.1) // 保持确定性
        .build()
        .expect("构建 LLM 失败");
    // let llm: Arc<Ollama> = LLMBuilder::<Ollama>::new()
    //     .api_key(api_key)
    //     .model("qwen3:4b")
    //     .temperature(0.1) // 保持确定性
    //     .build()
    //     .expect("构建 LLM 失败");
    // let llm: Arc<OpenRouter> = LLMBuilder::<OpenRouter>::new()
    //     .api_key("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")
    //     .model("qwen/qwen3-coder:free")
    //     .max_tokens(8192) // Shader 可能很长
    //     .temperature(0.1) // 保持确定性
    //     .build()
    //     .expect("构建 LLM 失败");

    // 初始化 Agents
    let mut merger_agent = create_agent(llm.clone(), ShaderMergerAgent::default()).await?;
    let mut verifier_agent = create_agent(llm.clone(), ShaderVerifierAgent::default()).await?;

    // 目录设置
    let input_dir = "assets/shaders_extract/environment/unlit_decal/vs"; // 根据实际修改
    let extension = "vert"; // 或 frag

    // 读取初始 Base 文件
    let base_path = Path::new(input_dir).join(format!("BASE.{}", extension));
    if !base_path.exists() {
        return Err(anyhow!("在 {:?} 未找到 BASE shader", base_path));
    }

    let mut current_master_source = fs::read_to_string(&base_path)?;
    info!("已加载 BASE shader。");

    // 获取排序后的文件列表
    let sorted_files = get_sorted_files(input_dir, extension)?;
    info!("找到 {} 个变体待处理。", sorted_files.len());

    // 迭代处理
    for (index, file_path) in sorted_files.iter().enumerate() {
        let target_defines = match extract_defines_from_filename(file_path) {
            Ok(d) => d,
            Err(e) => {
                info!("由于 define 提取错误跳过文件 {:?}: {:?}", file_path, e);
                continue;
            }
        };

        if let Err(e) = process_single_file(
            file_path,
            &target_defines, // 传递 defines
            &mut current_master_source,
            &mut merger_agent,
            &mut verifier_agent,
        )
        .await
        {
            info!("合并文件失败 {:?}: {:?}", file_path, e);
            // 失败后保存并退出
            save_checkpoint(index, "CRASH_SAVE", &current_master_source)?;
            return Err(e);
        }

        // 成功后保存快照
        if (index + 1) % 5 == 0 || index == sorted_files.len() - 1 {
            save_checkpoint(index, "merged_master", &current_master_source)?;
        }
    }

    info!("🎉 所有 shader 合并成功！");
    fs::write(
        "assets/shaders_reverse_history/FINAL_REVERSED.glsl",
        current_master_source,
    )?;

    Ok(())
}
