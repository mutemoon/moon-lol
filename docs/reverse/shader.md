# Shader 系统逆向分析

> 目标：League of Legends 粒子(VFX)渲染的 shader 输入装配路径。
> 从「配置反序列化 → 运行时描述符 → shader 常量/纹理装配 → DXBC 加载」完整梳理。
> 分析方式：纯静态分析（IDA Pro 8.3，imagebase=0x140000000）。
> 详细内容已拆分到 [shader/](shader/) 子文档，本文只做简介和目录。

## 总体数据流

```
解包 bin 配置 (VfxSystemDefinitionData / VfxEmitterDefinitionData, 带 hash 属性)
        │  反序列化（字符串键属性解析器 sub_1412CE0F0 链）
        ▼
运行时描述符 ParticleEmitterDescriptor (1136B, 由 ParticleEmitterDescriptor_Init 构造)
        │  ShaderEffect 通过 +16 字段引用该描述符
        ▼
ShaderEffect_SetupParticleShader(_Full)  ← 把描述符字段映射为 shader 常量/纹理
        │
        ├─ ShaderManager_LoadShader   → 加载 shader 对象到 ShaderEffect+128
        ├─ 7-pass 渲染状态循环         → 混合/深度/模板/Z 写
        ├─ 上传常量 (TEXTURE_INFO / kColorFactor / SLICE_RANGE / ...)
        ├─ 绑定纹理槽 (DIFFUSE_MAP / FALLOFF / PALETTE / ...)
        └─ ShaderManager_FinalizeShader → 从 WAD 加载预编译 DXBC 并创建 D3D shader
```

因为解包出的 `VfxSystemDefinitionData/VfxEmitterDefinitionData` 是带 hash 属性的 bin 配置对象，而渲染时真正使用的是一个 1136 字节的运行时描述符，所以两者**不是同一个结构体**；配置只是「数据源」，需要反序列化拷贝/转换到描述符后才用于渲染。

## 子文档目录

| 子文档 | 内容 |
|------|------|
| [shader/structs.md](shader/structs.md) | 运行时结构体定义（已写入 IDB）：`ShaderEffect`(512B)、`ParticleEmitterDescriptor`(1136B)、`ShaderPassVariantData`(136B)、`ParticleRenderContext`(952B) 完整字段布局；IDA 复现脚本（`parse_decls`/`SetType`）；qword 标尺寻址准确性说明；Hex-Rays 类型推断经验 |
| [shader/deserialization.md](shader/deserialization.md) | 配置 → 描述符反序列化：解析器入口链（sub_1412CE0F0 等）、6 个配置属性读取器签名；运行时描述符全量默认值表（来自 `ParticleEmitterDescriptor_Init`，含每个偏移 ↔ 配置键的对应）；shader 端关键常量默认值（哨兵/Fresnel/白色/epsilon 等全局地址） |
| [shader/assembly.md](shader/assembly.md) | Shader 输入装配核心概览：`SetupParticleShader(_Full)` 公共流程（5 步）、常量/纹理来源映射表、Full 与精简两版差异；工厂 + 虚表 +8 分发（重量 224B / 轻量 152B 两类）；发射器 `Type`(Simple/Complex) 字段作为 Full/Simple 上游总开关的完整选择链 |
| [shader/assembly-full.md](shader/assembly-full.md) | Full 版完整装配逻辑（Rust 伪代码，反编译逐行核对）：纹理槽 0..11 → 常量名映射表(off_141B26570)；装配主链 (A)-(N) 全步骤；`BuildShaderPathAndDefines` 的 shader 路径选择 + 宏开关触发条件表；`BindTextureSlot` 采样器状态/纹理回退逻辑；逐-pass 渲染态组装表 |
| [shader/constants.md](shader/constants.md) | 深度转换 & 软粒子 & 切片 & 默认纹理常量：`cDepthConversionParams` 计算式；`VfxSoftParticleParams`(20B) 字段语义与 `cSoftParticleParams`/`cSoftParticleControl` 取值；切片技术（`QUAD_PS_Slice`/`SLICE_RANGE`）双重逻辑；`PIXEL_COLOR_REMAP_RAMP` 默认 1×1 纯黑纹理证据链与系统默认纹理槽布局；`Clamp_No_Mip` 采样器真实状态与项目侧对齐 |
| [shader/dxbc.md](shader/dxbc.md) | DXBC 加载：预编译字节码存储于 `ShaderCache.dx11.wad.client`，路径拼接 `.dx11` + defines 哈希定位变体 |

## 相关函数索引

完整函数命名见 [functions.md](functions.md)。核心：

| 函数 | 命名 |
|------|------|
| sub_1412B7330 | ShaderEffect_SetupParticleShader_Full |
| sub_1412B7EA0 | ShaderEffect_SetupParticleShader |
| sub_1412DD450 | ShaderEffect_BuildShaderPathAndDefines |
| sub_1412E39D0 | ShaderEffect_SetSoftParticleParams |
| sub_141323730 | ShaderEffect_SetDepthConversionParams |
| sub_1412A4530 | ParticleEmitterDescriptor_Init |
| sub_1412CE0F0 | 配置反序列化顶层入口 |
| sub_1412FFED0 | ShaderEffect_SetConstantVec4ByName |
| sub_14131BC10 | ShaderEffect_SetConstantVec4ByHandle |
| sub_1413C6E60 | Shader_HashConstantName |
| sub_1412AD400 | ShaderEffect_BindTextureSlot |
| sub_14130B300 | ShaderEffect_GetTextureSlot |
| sub_141306BA0 | ShaderManager_LoadShader |
| sub_14130F940 | ShaderManager_FinalizeShader |
| byte_141E4D7D0 | ShaderPass_RenderPassTable |
