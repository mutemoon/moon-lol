//! SPIR-V 预处理：剥离 LoL 预编译 SPIR-V 中 wgpu/Vulkan 默认设备特性不支持的
//! capability / memory model / execution mode。
//!
//! 根因：dxbc_compiler 产出的 SPIR-V 声明了 Vulkan 1.2 的能力
//! (PhysicalStorageBufferAddresses / VulkanMemoryModel / RoundingModeRTE /
//! DenormFlushToZero / DemoteToHelperInvocation)，而 DefaultPlugins 请求的设备
//! 没有开启对应特性，导致 `alpha_blend_mesh_pipeline is invalid`。
//!
//! 这些视觉着色器并未真正用到物理存储缓冲寻址或 Vulkan 内存模型，剥离后改写为
//! Logical + GLSL450 即可，无需任何 wgpu 特性。DemoteToHelperInvocation 等价于
//! HLSL clip()/discard，改写为 OpKill（核心 Shader 能力）。

/// 需要剥离的 OpCapability 值。
const STRIP_CAPS: &[u32] = &[
    5347, // PhysicalStorageBufferAddresses
    5345, // VulkanMemoryModel
    4467, // RoundingModeRTE
    4465, // DenormFlushToZero
    5379, // DemoteToHelperInvocation
];

const OP_MEMORY_MODEL: u32 = 14;
const OP_EXECUTION_MODE: u32 = 16;
const OP_CAPABILITY: u32 = 17;
const OP_FUNCTION_END: u32 = 56;
const OP_LABEL: u32 = 248;
const OP_KILL: u32 = 252;
const OP_DEMOTE_TO_HELPER: u32 = 5380;
const OP_EXECUTION_MODE_ID: u32 = 331;

const ADDR_LOGICAL: u32 = 0;
const MEM_GLSL450: u32 = 1;
const ADDR_PHYS_STORAGE_64: u32 = 5348;
const MEM_VULKAN: u32 = 3;

const MODE_ROUNDING_RTE: u32 = 4462;
const MODE_DENORM_FLUSH: u32 = 4460;

/// 剥离并改写 SPIR-V 二进制。输入不合法时原样返回。
pub fn strip_spirv(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return bytes.to_vec();
    }
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();

    // magic 校验
    if words[0] != 0x0723_0203 {
        return bytes.to_vec();
    }

    let mut out: Vec<u32> = Vec::with_capacity(words.len());
    // header：magic / version / generator / bound / schema 原样保留
    out.extend_from_slice(&words[0..5]);

    // OpKill 是块终结指令而 OpDemoteToHelperInvocation 不是：改写后需丢弃
    // 到下一个 OpLabel/OpFunctionEnd 之前的死代码（如紧随的 OpBranch），
    // 否则同块双终结触发 spirv-val 的 Branch must appear in a block
    let mut skip_dead = false;
    let mut i = 5;
    while i < words.len() {
        let word0 = words[i];
        let opcode = word0 & 0xFFFF;
        let wc = (word0 >> 16) as usize;
        if wc == 0 {
            // 非法指令，无法继续安全解析，原样返回剩余
            out.extend_from_slice(&words[i..]);
            break;
        }
        let inst = &words[i..i + wc];

        if skip_dead {
            if matches!(opcode, OP_LABEL | OP_FUNCTION_END) {
                skip_dead = false;
            } else {
                i += wc;
                continue;
            }
        }

        match opcode {
            OP_CAPABILITY if inst.len() >= 2 && STRIP_CAPS.contains(&inst[1]) => {
                // 丢弃该 capability
            }
            OP_MEMORY_MODEL if inst.len() >= 3 => {
                // PhysicalStorageBuffer64 Vulkan -> Logical GLSL450
                let _ = (ADDR_PHYS_STORAGE_64, MEM_VULKAN);
                out.push(word0);
                out.push(ADDR_LOGICAL);
                out.push(MEM_GLSL450);
            }
            OP_DEMOTE_TO_HELPER => {
                // OpDemoteToHelperInvocation -> OpKill（单字指令，word_count 不变），
                // 并丢弃其后到下一个 label 之前的死代码
                out.push((1u32 << 16) | OP_KILL);
                skip_dead = true;
            }
            OP_EXECUTION_MODE
                if inst.len() >= 3 && matches!(inst[2], MODE_ROUNDING_RTE | MODE_DENORM_FLUSH) =>
            {
                // 丢弃 rounding/denorm execution mode
            }
            OP_EXECUTION_MODE_ID
                if inst.len() >= 3 && matches!(inst[2], MODE_ROUNDING_RTE | MODE_DENORM_FLUSH) =>
            {
                // ID 形式的同类 execution mode，一并丢弃
            }
            _ => out.extend_from_slice(inst),
        }

        i += wc;
    }

    out.iter().flat_map(|w| w.to_le_bytes()).collect()
}

const OP_DECORATE: u32 = 71;
const DECORATION_BINDING: u32 = 33;
const DECORATION_LOCATION: u32 = 30;
const DECORATION_COMPONENT: u32 = 31;

const OP_NAME: u32 = 5;
const OP_ENTRY_POINT: u32 = 15;
const OP_TYPE_FLOAT: u32 = 22;
const OP_TYPE_VECTOR: u32 = 23;
const OP_TYPE_POINTER: u32 = 32;
const OP_FUNCTION: u32 = 54;
const OP_VARIABLE: u32 = 59;
const OP_ACCESS_CHAIN: u32 = 65;
const OP_IN_BOUNDS_ACCESS_CHAIN: u32 = 66;
const OP_EXT_INST: u32 = 12;
const OP_VECTOR_SHUFFLE: u32 = 79;
const OP_COMPOSITE_EXTRACT: u32 = 81;
const OP_COMPOSITE_INSERT: u32 = 82;

/// SPIR-V 存储类：顶点/片元阶段输入。
pub const STORAGE_INPUT: u32 = 1;
/// SPIR-V 存储类：顶点/片元阶段输出。
pub const STORAGE_OUTPUT: u32 = 3;

fn bytes_to_words(bytes: &[u8]) -> Option<Vec<u32>> {
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return None;
    }
    let words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if words[0] != 0x0723_0203 {
        return None;
    }
    Some(words)
}

/// 解析模块中带 Location 装饰、指定存储类的 float 接口变量，
/// 返回 (location, component) → 向量分量数（标量记 1，Component 装饰缺省 0）。
/// 同一 location 可被多个变量按 Component 打包占用，必须分开统计，
/// 否则跨 component 合并宽度会把窄变量加宽到越过 4 分量边界
/// （VUID-StandaloneSpirv-Component-04921）。输入不合法返回 None。
pub fn interface_vector_widths(
    bytes: &[u8],
    storage_class: u32,
) -> Option<std::collections::BTreeMap<(u32, u32), u32>> {
    let words = bytes_to_words(bytes)?;

    let mut floats: std::collections::BTreeSet<u32> = Default::default();
    // vector id → (分量类型 id, 分量数)
    let mut vectors: std::collections::BTreeMap<u32, (u32, u32)> = Default::default();
    // pointer id → (存储类, 指向类型 id)
    let mut pointers: std::collections::BTreeMap<u32, (u32, u32)> = Default::default();
    // 变量 id → (指针类型 id, 存储类)
    let mut vars: std::collections::BTreeMap<u32, (u32, u32)> = Default::default();
    // 变量 id → location
    let mut locations: std::collections::BTreeMap<u32, u32> = Default::default();
    // 变量 id → component（无装饰即 0）
    let mut components: std::collections::BTreeMap<u32, u32> = Default::default();

    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        if wc == 0 || i + wc > words.len() {
            return None;
        }
        let inst = &words[i..i + wc];
        match opcode {
            OP_TYPE_FLOAT if wc >= 3 && inst[2] == 32 => {
                floats.insert(inst[1]);
            }
            OP_TYPE_VECTOR if wc >= 4 => {
                vectors.insert(inst[1], (inst[2], inst[3]));
            }
            OP_TYPE_POINTER if wc >= 4 => {
                pointers.insert(inst[1], (inst[2], inst[3]));
            }
            OP_VARIABLE if wc >= 4 => {
                vars.insert(inst[2], (inst[1], inst[3]));
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_LOCATION => {
                locations.insert(inst[1], inst[3]);
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_COMPONENT => {
                components.insert(inst[1], inst[3]);
            }
            _ => {}
        }
        i += wc;
    }

    let mut out: std::collections::BTreeMap<(u32, u32), u32> = Default::default();
    for (var_id, (ptr_type, storage)) in &vars {
        if *storage != storage_class {
            continue;
        }
        let Some(&location) = locations.get(var_id) else {
            continue;
        };
        let Some(&(ptr_storage, pointee)) = pointers.get(ptr_type) else {
            continue;
        };
        if ptr_storage != storage_class {
            continue;
        }
        let width = if floats.contains(&pointee) {
            1
        } else if let Some(&(comp, count)) = vectors.get(&pointee) {
            if !floats.contains(&comp) {
                continue;
            }
            count
        } else {
            continue;
        };
        let key = (location, components.get(var_id).copied().unwrap_or(0));
        out.insert(key, out.get(&key).copied().unwrap_or(0).max(width));
    }
    Some(out)
}

/// 把 PS 输入接口变量按 (location, component) 加宽到目标分量数（vecN → vecM，M > N）。
///
/// 根因：D3D11 允许 VS 输出比 PS 输入宽，dxbc_compiler 原样保留了这种
/// 不对称（如 MeshVs 输出 TEXCOORD1 为 vec3 而 MeshPs 读 vec2），而 Vulkan
/// 在未启用 maintenance4 时要求输出分量数 ≤ 输入分量数（触发
/// VUID-RuntimeSpirv-maintenance4-06817）。加宽输入是安全方向：新增分量
/// 无人读取，逐分量 OpAccessChain 的结果类型也不变。
///
/// 变量存在整向量 OpLoad 等无法安全改写的用法时返回 None（调用方告警
/// 跳过）；无需改写（宽度已达标）时也返回 None，改写天然幂等。
/// 目标键为 (location, component)；加宽上限受两重约束：location 的 4
/// 分量边界（VUID-StandaloneSpirv-Component-04921），以及同 location 上按
/// Component 打包的下一个变量（如 TEXCOORD2@(3,0) + TEXCOORD6@(3,2)，越界
/// 重叠触发 VUID-StandaloneSpirv-OpEntryPoint-08721），超限部分截断。
pub fn widen_ps_inputs(
    bytes: &[u8],
    targets: &std::collections::BTreeMap<(u32, u32), u32>,
) -> Option<Vec<u8>> {
    let words = bytes_to_words(bytes)?;

    let mut floats: std::collections::BTreeSet<u32> = Default::default();
    let mut vectors: std::collections::BTreeMap<u32, (u32, u32)> = Default::default();
    let mut pointers: std::collections::BTreeMap<u32, (u32, u32)> = Default::default();
    // 变量 id → (指令起始 word 下标, 指针类型 id, 存储类)
    let mut vars: std::collections::BTreeMap<u32, (usize, u32, u32)> = Default::default();
    let mut locations: std::collections::BTreeMap<u32, u32> = Default::default();
    let mut components: std::collections::BTreeMap<u32, u32> = Default::default();

    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        if wc == 0 || i + wc > words.len() {
            return None;
        }
        let inst = &words[i..i + wc];
        match opcode {
            OP_TYPE_FLOAT if wc >= 3 && inst[2] == 32 => {
                floats.insert(inst[1]);
            }
            OP_TYPE_VECTOR if wc >= 4 => {
                vectors.insert(inst[1], (inst[2], inst[3]));
            }
            OP_TYPE_POINTER if wc >= 4 => {
                pointers.insert(inst[1], (inst[2], inst[3]));
            }
            OP_VARIABLE if wc >= 4 => {
                vars.insert(inst[2], (i, inst[1], inst[3]));
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_LOCATION => {
                locations.insert(inst[1], inst[3]);
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_COMPONENT => {
                components.insert(inst[1], inst[3]);
            }
            _ => {}
        }
        i += wc;
    }

    // 同一 location 可能被多个 Input 变量按 Component 打包，加宽不得
    // 越过同 location 下一个变量的起始 component
    let mut loc_components: std::collections::BTreeMap<u32, Vec<u32>> = Default::default();
    for (var_id, (_, _, storage)) in &vars {
        if *storage != STORAGE_INPUT {
            continue;
        }
        if let Some(&loc) = locations.get(var_id) {
            loc_components
                .entry(loc)
                .or_default()
                .push(components.get(var_id).copied().unwrap_or(0));
        }
    }

    // 筛出需加宽的变量：(变量指令下标, 变量 id, float 分量类型 id, 目标分量数)
    let mut rewrites: Vec<(usize, u32, u32, u32)> = Vec::new();
    for (var_id, (inst_idx, ptr_type, storage)) in &vars {
        if *storage != STORAGE_INPUT {
            continue;
        }
        let Some(&location) = locations.get(var_id) else {
            continue;
        };
        let component = components.get(var_id).copied().unwrap_or(0);
        let Some(&target_width) = targets.get(&(location, component)) else {
            continue;
        };
        // 上限：同 location 比本变量 component 大的最小起始位，否则 4 分量边界
        let cap = loc_components
            .get(&location)
            .and_then(|cs| cs.iter().filter(|c| **c > component).min().copied())
            .unwrap_or(4)
            .min(4);
        let target_width = target_width.min(cap.saturating_sub(component));
        let Some(&(_, pointee)) = pointers.get(ptr_type) else {
            continue;
        };
        let Some(&(comp, count)) = vectors.get(&pointee) else {
            continue;
        };
        if !floats.contains(&comp) || count >= target_width {
            continue;
        }
        rewrites.push((*inst_idx, *var_id, comp, target_width));
    }
    if rewrites.is_empty() {
        return None;
    }

    // 安全检查：目标变量只允许出现在逐分量访问链与声明/装饰性指令里；
    // 整向量 OpLoad 的结果类型会随加宽失配，遇到则整体放弃改写。
    // 扫描必须区分 ID 操作数与数字字面量：全局段的 OpMemberName /
    // OpMemberDecorate / OpConstant 等携带成员索引、常量值字面量，裸 word
    // 匹配会撞上变量 id 误报（如 $Globals 成员索引 33 撞 TEXCOORD1 id）。
    // 全局段只有 OpVariable 初始化器能真正引用变量 id；函数体内对
    // 已知带字面量的指令只检查 ID 操作数位，未知指令保守全扫。
    let rewrite_ids: std::collections::BTreeSet<u32> =
        rewrites.iter().map(|(_, id, _, _)| *id).collect();
    let hits = |ws: &[u32]| ws.iter().any(|w| rewrite_ids.contains(w));
    let mut in_function = false;
    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        let inst = &words[i..i + wc];
        if opcode == OP_FUNCTION {
            in_function = true;
        }
        let unsafe_use = if !in_function {
            // 全局段：类型/常量/调试名/装饰指令均无法引用变量 id，
            // 只有 OpVariable 初始化器（第 5 个 word）可能引用
            opcode == OP_VARIABLE && wc >= 5 && hits(&inst[4..])
        } else {
            match opcode {
                OP_ACCESS_CHAIN | OP_IN_BOUNDS_ACCESS_CHAIN => false,
                // OpExtInst：inst[4] 是指令号字面量；参数 [5..] 是 ID
                // （InterpolateAt* 系列会直接拿变量指针，必须检查）
                OP_EXT_INST if wc >= 5 => hits(&inst[5..]),
                // OpVectorShuffle：[3][4] 是向量 ID，[5..] 是分量字面量
                OP_VECTOR_SHUFFLE if wc >= 5 => hits(&inst[3..5]),
                // OpCompositeExtract：[3] 是复合 ID，[4..] 是索引字面量
                OP_COMPOSITE_EXTRACT if wc >= 4 => hits(&inst[3..4]),
                // OpCompositeInsert：[3][4] 是 ID，[5..] 是索引字面量
                OP_COMPOSITE_INSERT if wc >= 5 => hits(&inst[3..5]),
                _ => hits(&inst[1..]),
            }
        };
        if unsafe_use {
            return None;
        }
        i += wc;
    }

    // 查找/新建目标向量类型与 Input 指针类型，新 id 从 bound 起分配。
    // 因为复用的既有类型可能声明在被改写变量之后（def-before-use 会被破坏），
    // 所以把被改写的 OpVariable 整体挪到全局段末尾（第一个 OpFunction 之前），
    // 新类型也插在那里；装饰/EntryPoint 对变量的前向引用合法，函数体均在其后。
    let mut next_id = words[3];
    let mut new_insts: Vec<u32> = Vec::new();

    let mut vec_type_of: std::collections::BTreeMap<(u32, u32), u32> = Default::default();
    let mut ptr_type_of: std::collections::BTreeMap<u32, u32> = Default::default();
    // 变量指令下标 → 新指针类型 id
    let mut var_new_ptr: std::collections::BTreeMap<usize, u32> = Default::default();
    for &(inst_idx, _, comp, width) in &rewrites {
        let vec_id = *vec_type_of.entry((comp, width)).or_insert_with(|| {
            match vectors.iter().find(|(_, v)| **v == (comp, width)) {
                Some((&id, _)) => id,
                None => {
                    let id = next_id;
                    next_id += 1;
                    new_insts.extend_from_slice(&[(4u32 << 16) | OP_TYPE_VECTOR, id, comp, width]);
                    id
                }
            }
        });
        let ptr_id = *ptr_type_of.entry(vec_id).or_insert_with(|| {
            match pointers
                .iter()
                .find(|(_, p)| **p == (STORAGE_INPUT, vec_id))
            {
                Some((&id, _)) => id,
                None => {
                    let id = next_id;
                    next_id += 1;
                    new_insts.extend_from_slice(&[
                        (4u32 << 16) | OP_TYPE_POINTER,
                        id,
                        STORAGE_INPUT,
                        vec_id,
                    ]);
                    id
                }
            }
        });
        var_new_ptr.insert(inst_idx, ptr_id);
    }

    // 重组模块：跳过原位置的待改写变量，在第一个 OpFunction 前插入
    // 新类型 + 改写后的变量声明
    let mut out_words: Vec<u32> = Vec::with_capacity(words.len() + new_insts.len());
    out_words.extend_from_slice(&words[0..5]);
    out_words[3] = next_id;

    let mut moved_vars: Vec<u32> = Vec::new();
    let mut emitted_tail = false;
    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        if !emitted_tail && opcode == OP_FUNCTION {
            out_words.extend_from_slice(&new_insts);
            out_words.extend_from_slice(&moved_vars);
            emitted_tail = true;
        }
        if let Some(&ptr_id) = var_new_ptr.get(&i) {
            let mut inst = words[i..i + wc].to_vec();
            inst[1] = ptr_id;
            moved_vars.extend_from_slice(&inst);
        } else {
            out_words.extend_from_slice(&words[i..i + wc]);
        }
        i += wc;
    }
    if !emitted_tail {
        out_words.extend_from_slice(&new_insts);
        out_words.extend_from_slice(&moved_vars);
    }
    Some(out_words.iter().flat_map(|w| w.to_le_bytes()).collect())
}

/// 按显式映射表（旧 binding 值 → 新 binding 值）改写 SPIR-V 中所有
/// `OpDecorate Binding` 装饰，不在表中的 binding 保持不变。
///
/// 根因：wgpu-hal Vulkan 后端创建 DescriptorSetLayout 时会把 entries（wgpu-core
/// 已按 binding 升序排序）压缩重编号为连续 0..n，naga 编译路径会用 binding_map
/// 同步重写 shader，但 spirv_shader_passthrough 的 SPIR-V 原样透传不经 naga，
/// 导致 shader 里的 binding（如 PS 的 100+）与 DSL 里的压缩编号（0..n）不匹配，
/// 触发 VUID-VkGraphicsPipelineCreateInfo-layout-07988。
///
/// 因此 extract_shaders 的离线统一 pass 会按家族并集布局计算每个 binding 的
/// Vulkan 压缩编号（VS = 并集排名，PS = VS 并集条目数 + 并集排名），并通过
/// 本函数把编号直接固化进 .spv 文件。
///
/// 返回 None 表示无需改写（已是目标编号或输入不合法）。
pub fn remap_bindings(bytes: &[u8], map: &std::collections::BTreeMap<u32, u32>) -> Option<Vec<u8>> {
    if bytes.len() < 20 || bytes.len() % 4 != 0 {
        return None;
    }
    let mut words: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    if words[0] != 0x0723_0203 {
        return None;
    }

    let mut changed = false;
    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        if wc == 0 || i + wc > words.len() {
            return None;
        }
        if opcode == OP_DECORATE && wc >= 4 && words[i + 2] == DECORATION_BINDING {
            if let Some(&new) = map.get(&words[i + 3]) {
                if new != words[i + 3] {
                    words[i + 3] = new;
                    changed = true;
                }
            }
        }
        i += wc;
    }
    if !changed {
        return None;
    }
    Some(words.iter().flat_map(|w| w.to_le_bytes()).collect())
}

// ---------------------------------------------------------------------------
// cbuffer 内存布局反射：直接从 SPIR-V 二进制读取权威布局
// ---------------------------------------------------------------------------

const OP_MEMBER_NAME: u32 = 6;
const OP_TYPE_STRUCT: u32 = 30;
const OP_MEMBER_DECORATE: u32 = 72;
const DECORATION_OFFSET: u32 = 35;
const DECORATION_DESCRIPTOR_SET: u32 = 34;
const STORAGE_UNIFORM: u32 = 2;
const STORAGE_PUSH_CONSTANT: u32 = 9;

/// 从 SPIR-V 字流的 `start` 处读取一条 literal string（UTF-8，4 字节对齐、
/// 以 null 结尾并零填充），返回 (字符串, 消耗的 word 数)。
fn read_spirv_string(words: &[u32], start: usize) -> (String, usize) {
    let mut bytes: Vec<u8> = Vec::new();
    let mut consumed = 0usize;
    for &w in &words[start..] {
        consumed += 1;
        let b = w.to_le_bytes();
        for &c in &b {
            if c == 0 {
                return (String::from_utf8_lossy(&bytes).into_owned(), consumed);
            }
            bytes.push(c);
        }
    }
    (String::from_utf8_lossy(&bytes).into_owned(), consumed)
}

/// 单个 cbuffer（uniform block）从 SPIR-V 直接反射出的权威布局。
#[derive(Debug, Clone, PartialEq)]
pub struct SpirvCbufferLayout {
    /// `OpDecorate <var> Binding` 的值（离线统一 pass 已改写为 Vulkan 压缩编号）
    pub binding: u32,
    /// `OpDecorate <var> DescriptorSet` 的值
    pub set: u32,
    /// 成员名 → `OpMemberDecorate <struct> <idx> Offset` 的字节偏移。
    /// 被 DXC 剥离名字的未使用成员（有 offset 无 name）不计入。
    pub member_offsets: std::collections::BTreeMap<String, u32>,
}

/// 直接解析 SPIR-V 二进制，反射出全部 Uniform / PushConstant 存储类 cbuffer 的
/// 权威内存布局（binding + 每个具名成员的 offset）。
///
/// 这是 DXBC→SPIR-V 编译产物里 shader 实际读取的地址布局，用作校验 map.ron
/// 变体布局与 `set_param` 写入偏移正确性的地面真值。返回 key 为 cbuffer 名
/// （取自 `OpName <var>`，如 `$Globals`/`PerFrameVertexCB`）。输入不合法返回 None。
pub fn reflect_cbuffer_layouts(
    bytes: &[u8],
) -> Option<std::collections::BTreeMap<String, SpirvCbufferLayout>> {
    use std::collections::BTreeMap;
    let words = bytes_to_words(bytes)?;

    // id → 名字（OpName）
    let mut names: BTreeMap<u32, String> = BTreeMap::new();
    // struct id → (成员下标 → 成员名)（OpMemberName）
    let mut member_names: BTreeMap<u32, BTreeMap<u32, String>> = BTreeMap::new();
    // struct id → (成员下标 → offset)（OpMemberDecorate Offset）
    let mut member_offsets: BTreeMap<u32, BTreeMap<u32, u32>> = BTreeMap::new();
    // 变量 id → binding / set（OpDecorate）
    let mut bindings: BTreeMap<u32, u32> = BTreeMap::new();
    let mut sets: BTreeMap<u32, u32> = BTreeMap::new();
    // pointer id → (存储类, 指向类型 id)
    let mut pointers: BTreeMap<u32, (u32, u32)> = BTreeMap::new();
    // struct 类型 id 集合
    let mut structs: std::collections::BTreeSet<u32> = Default::default();
    // 变量：(变量 id, 指针类型 id, 存储类)
    let mut vars: Vec<(u32, u32, u32)> = Vec::new();

    let mut i = 5;
    while i < words.len() {
        let opcode = words[i] & 0xFFFF;
        let wc = (words[i] >> 16) as usize;
        if wc == 0 || i + wc > words.len() {
            return None;
        }
        let inst = &words[i..i + wc];
        match opcode {
            OP_NAME if wc >= 3 => {
                let (s, _) = read_spirv_string(words.as_slice(), i + 2);
                names.insert(inst[1], s);
            }
            OP_MEMBER_NAME if wc >= 4 => {
                let (s, _) = read_spirv_string(words.as_slice(), i + 3);
                member_names.entry(inst[1]).or_default().insert(inst[2], s);
            }
            OP_MEMBER_DECORATE if wc >= 5 && inst[3] == DECORATION_OFFSET => {
                member_offsets
                    .entry(inst[1])
                    .or_default()
                    .insert(inst[2], inst[4]);
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_BINDING => {
                bindings.insert(inst[1], inst[3]);
            }
            OP_DECORATE if wc >= 4 && inst[2] == DECORATION_DESCRIPTOR_SET => {
                sets.insert(inst[1], inst[3]);
            }
            OP_TYPE_STRUCT if wc >= 2 => {
                structs.insert(inst[1]);
            }
            OP_TYPE_POINTER if wc >= 4 => {
                pointers.insert(inst[1], (inst[2], inst[3]));
            }
            OP_VARIABLE if wc >= 4 => {
                vars.push((inst[2], inst[1], inst[3]));
            }
            _ => {}
        }
        i += wc;
    }

    let mut out: BTreeMap<String, SpirvCbufferLayout> = BTreeMap::new();
    for (var_id, ptr_type, storage) in vars {
        if storage != STORAGE_UNIFORM && storage != STORAGE_PUSH_CONSTANT {
            continue;
        }
        let Some(&(ptr_storage, pointee)) = pointers.get(&ptr_type) else {
            continue;
        };
        if ptr_storage != storage {
            continue;
        }
        if !structs.contains(&pointee) {
            continue;
        }
        // cbuffer 名优先取变量名（spirv-reflect 亦以此作 binding 名，如 $Globals）
        let name = names
            .get(&var_id)
            .cloned()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                names
                    .get(&pointee)
                    .map(|s| s.trim_start_matches("type.").to_string())
            })?;
        let mut offsets_by_idx = member_offsets.get(&pointee).cloned().unwrap_or_default();
        let names_by_idx = member_names.get(&pointee).cloned().unwrap_or_default();
        let mut member_offsets_out: BTreeMap<String, u32> = BTreeMap::new();
        for (idx, off) in offsets_by_idx.iter_mut() {
            if let Some(mname) = names_by_idx.get(idx) {
                if !mname.is_empty() {
                    member_offsets_out.insert(mname.clone(), *off);
                }
            }
        }
        out.insert(
            name,
            SpirvCbufferLayout {
                binding: bindings.get(&var_id).copied().unwrap_or(u32::MAX),
                set: sets.get(&var_id).copied().unwrap_or(0),
                member_offsets: member_offsets_out,
            },
        );
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words_to_bytes(ws: &[u32]) -> Vec<u8> {
        ws.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    fn parse_words(bytes: &[u8]) -> Vec<u32> {
        bytes
            .chunks_exact(4)
            .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
            .collect()
    }

    #[test]
    fn strips_caps_and_rewrites_memory_model() {
        // header + OpCapability Shader + OpCapability PhysicalStorageBufferAddresses
        // + OpMemoryModel PhysicalStorageBuffer64 Vulkan + OpExtInstImport(占位)
        let header = [0x0723_0203u32, 0x0001_0600, 0, 100, 0];
        let cap_shader = (2u32 << 16) | OP_CAPABILITY;
        let cap_phys = (2u32 << 16) | OP_CAPABILITY;
        let mem_model = (3u32 << 16) | OP_MEMORY_MODEL;
        let input = words_to_bytes(&[
            header[0],
            header[1],
            header[2],
            header[3],
            header[4],
            cap_shader,
            1, // Shader
            cap_phys,
            5347, // PhysicalStorageBufferAddresses (应剥离)
            mem_model,
            ADDR_PHYS_STORAGE_64,
            MEM_VULKAN,
        ]);

        let out = strip_spirv(&input);
        let w = parse_words(&out);

        // magic 保留
        assert_eq!(w[0], 0x0723_0203);
        // OpCapability Shader 保留
        assert!(
            w.windows(2)
                .any(|x| x[0] & 0xFFFF == OP_CAPABILITY && x[1] == 1)
        );
        // PhysicalStorageBufferAddresses 已移除
        assert!(
            !w.windows(2)
                .any(|x| x[0] & 0xFFFF == OP_CAPABILITY && x[1] == 5347)
        );
        // OpMemoryModel 改写为 Logical + GLSL450
        let mm = w
            .windows(3)
            .find(|x| x[0] & 0xFFFF == OP_MEMORY_MODEL)
            .unwrap();
        assert_eq!(mm[1], ADDR_LOGICAL);
        assert_eq!(mm[2], MEM_GLSL450);
    }

    #[test]
    fn demote_becomes_kill() {
        let header = [0x0723_0203u32, 0x0001_0600, 0, 10, 0];
        let cap_shader = (2u32 << 16) | OP_CAPABILITY;
        let mem_model = (3u32 << 16) | OP_MEMORY_MODEL;
        let demote = (1u32 << 16) | OP_DEMOTE_TO_HELPER;
        // demote 后紧跟的 OpBranch 是死代码，必须随改写一并丢弃；
        // 下一个 OpLabel 及其后指令需保留
        let branch_dead = (2u32 << 16) | 249; // OpBranch %7
        let label = (2u32 << 16) | OP_LABEL; // %7 = OpLabel
        let func_end = (1u32 << 16) | OP_FUNCTION_END;
        let input = words_to_bytes(&[
            header[0],
            header[1],
            header[2],
            header[3],
            header[4],
            cap_shader,
            1,
            mem_model,
            ADDR_PHYS_STORAGE_64,
            MEM_VULKAN,
            demote,
            branch_dead,
            7,
            label,
            7,
            func_end,
        ]);

        let out = strip_spirv(&input);
        let w = parse_words(&out);
        assert!(w.contains(&((1u32 << 16) | OP_KILL)));
        assert!(!w.contains(&((1u32 << 16) | OP_DEMOTE_TO_HELPER)));
        // 死 OpBranch 已丢弃，OpLabel/OpFunctionEnd 保留
        assert!(!w.windows(2).any(|x| x[0] == branch_dead && x[1] == 7));
        assert!(w.windows(2).any(|x| x[0] == label && x[1] == 7));
        assert!(w.contains(&func_end));
    }

    #[test]
    fn drops_rounding_execution_mode() {
        let header = [0x0723_0203u32, 0x0001_0600, 0, 10, 0];
        let cap_shader = (2u32 << 16) | OP_CAPABILITY;
        let mem_model = (3u32 << 16) | OP_MEMORY_MODEL;
        // OpExecutionMode %main RoundingModeRTE 32  (4 words)
        let em_rte = (4u32 << 16) | OP_EXECUTION_MODE;
        // OpExecutionMode %main OriginUpperLeft  (3 words)
        let em_origin = (3u32 << 16) | OP_EXECUTION_MODE;
        let input = words_to_bytes(&[
            header[0],
            header[1],
            header[2],
            header[3],
            header[4],
            cap_shader,
            1,
            mem_model,
            ADDR_PHYS_STORAGE_64,
            MEM_VULKAN,
            em_rte,
            1,
            MODE_ROUNDING_RTE,
            32,
            em_origin,
            1,
            7, // OriginUpperLeft
        ]);

        let out = strip_spirv(&input);
        let w = parse_words(&out);
        // RoundingModeRTE 已移除
        assert!(
            !w.windows(4)
                .any(|x| x[0] & 0xFFFF == OP_EXECUTION_MODE && x[2] == MODE_ROUNDING_RTE)
        );
        // OriginUpperLeft 保留
        assert!(
            w.windows(3)
                .any(|x| x[0] & 0xFFFF == OP_EXECUTION_MODE && x[2] == 7)
        );
    }
}
