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
                // OpDemoteToHelperInvocation -> OpKill（单字指令，word_count 不变）
                out.push((1u32 << 16) | OP_KILL);
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
        ]);

        let out = strip_spirv(&input);
        let w = parse_words(&out);
        assert!(w.contains(&((1u32 << 16) | OP_KILL)));
        assert!(!w.contains(&((1u32 << 16) | OP_DEMOTE_TO_HELPER)));
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

    /// 用真实 LoL spv 文件验证剥离结果，并写出供 spirv-val 外部校验。
    #[test]
    #[ignore]
    fn strip_real_shader_0063() {
        let path = "../../assets/shaders/hlsl/particlesystem/quad/ps/shader_0063.spv";
        let Ok(input) = std::fs::read(path) else {
            eprintln!("找不到真实 spv 文件 {path}，跳过");
            return;
        };
        let out = strip_spirv(&input);
        let out_path = std::env::temp_dir().join("shader_0063_stripped.spv");
        std::fs::write(&out_path, &out).unwrap();

        let w = parse_words(&out);
        assert_eq!(w[0], 0x0723_0203, "magic 保留");
        // 不再含任何被剥离的 capability
        for cap in STRIP_CAPS {
            assert!(
                !w.windows(2)
                    .any(|x| x[0] & 0xFFFF == OP_CAPABILITY && x[1] == *cap),
                "capability {cap} 仍残留"
            );
        }
        // OpMemoryModel 改为 Logical + GLSL450
        let mm = w
            .windows(3)
            .find(|x| x[0] & 0xFFFF == OP_MEMORY_MODEL)
            .unwrap();
        assert_eq!(mm[1], ADDR_LOGICAL);
        assert_eq!(mm[2], MEM_GLSL450);
        // 不含 rounding/denorm execution mode
        assert!(
            !w.windows(4)
                .any(|x| x[0] & 0xFFFF == OP_EXECUTION_MODE && x[2] == MODE_ROUNDING_RTE)
        );
        assert!(
            !w.windows(4)
                .any(|x| x[0] & 0xFFFF == OP_EXECUTION_MODE && x[2] == MODE_DENORM_FLUSH)
        );
        println!("剥离后 {} 字节，写出至 {}", out.len(), out_path.display());
    }
}
