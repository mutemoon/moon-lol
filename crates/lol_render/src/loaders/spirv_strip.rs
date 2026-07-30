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
