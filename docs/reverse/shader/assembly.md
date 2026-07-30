# Shader 输入装配（核心概览）

> 上级：[shader.md](../shader.md)。Full 版逐行装配伪代码见 [assembly-full.md](assembly-full.md)。

`sub_1412B7330` / `sub_1412B7EA0` 都是 **ShaderEffect 的虚表方法**（仅被 vtable 引用，无直接调用），契约相同：`a1`=ShaderEffect，`a1->descriptor`=运行时描述符，`a1->shader`=加载好的 shader 对象。因为它们把描述符字段逐一映射为 shader 常量/纹理绑定，所以它们就是「根据运行时结构体装配 shader 输入」的地方。

| 函数 | 命名 | 角色 |
|------|------|------|
| sub_1412B7330 | `ShaderEffect_SetupParticleShader_Full` | 完整版（VS/distortion+palette 变体） |
| sub_1412B7EA0 | `ShaderEffect_SetupParticleShader` | 精简版（PS/simple-color 变体） |

## 公共流程

1. `renderType = *(*(descriptor->primitive) + 8)`（Full 版中 ==12 直接返回）；
2. 拼 shader 路径 → `ShaderManager_LoadShader` 加载到 `a1->shader`；
3. 遍历 `ShaderPass_RenderPassTable`（byte_141E4D7D0，7 项×48B，每项 [+0]=passId [+8]=define 名），对每个有效 pass 调 `BuildShaderPathAndDefines` + `ShaderPass_UpdateRenderState`：
   - `descriptor->blendMode`(+117) 置混合态；
   - `descriptor->alphaRef`(+119) 置 ALPHA_TEST；
   - `descriptor->depthBiasFactor0/1`(+124/128) 与哨兵比较置深度偏移；
   - `descriptor->forceAnimMeshZWrite`(+115)&1 / `descriptor->backFaceOn`(+17) 影响 Z 写；
   - `descriptor->miscFlags`(+720)&1 影响 pass 分支；
4. 上传常量/绑纹理（均来自 descriptor）；
5. `ShaderManager_FinalizeShader(a1->shader, 2)` 创建 D3D shader 对象。

## 常量 / 纹理来源映射

| shader 输入 | 来源（descriptor 字段） |
|------|------|
| TEXTURE_INFO | texDivX/Y(+140/+144) → float4(u, 1/u, 1/v, 0) |
| TEXTURE_INFO_2 | Full: 第二纹理尺寸；PS: 常量(1,1,1,0) |
| kColorFactor | colorModulate(+224，默认白) |
| APPLY_TEAM_COLOR_CORRECTION | applyTeamColor(a1+25) 开关 + teamColorCorrection(+116)&1 取色 |
| vFresnel / FRESNEL | 常量(0,0,0,1) |
| PARTICLE_DEPTH_PUSH_PULL | depthPushPull(+828) |
| SLICE_RANGE | alphaSliceRange(+132)（>0 才上传，=float4(r, 1/r², 0,0)） |
| DistortionPower | distortionParams(+80)→[0] |
| SoftParticleParams | softParticleParams(+48)（默认 NULL→关，见 [constants.md](constants.md)） |
| DIFFUSE_MAP / FALLOFF_TEXTURE / PARTICLE_COLOR_TEXTURE | renderType==7 时绑定(+152/+184/+168) |
| cPaletteSelectMain / cAlphaErosion / vReflection | 仅 Full 版：paletteObj(+64) / alphaErosionObj(+56) / reflectionObj(+72) |

## 两版差异

- **Full (sub_1412B7330)**：多了 `overrideShader`(+104) 处理、TEXTURE_INFO_2 取第二纹理、以及调色板/alpha 侵蚀/反射三套纹理绑定；
- **PS (sub_1412B7EA0)**：TEXTURE_INFO_2 用常量、kColorFactor 直接取 +224 颜色。

## 谁调用 / 谁选择（工厂 + 虚表分发）

两个 setup 都是 ShaderEffect 的**虚方法**，位于各自类 vtable 的 **+8**（同一虚槽的两种实现），无任何直接 `call`：

| 类（对象大小） | vtable | +0（析构） | +8（本方法） |
|------|------|------|------|
| 重量（224B） | `off_141B266B0` | `ParticleShaderEffect_dtor_Heavy` | `ShaderEffect_SetupParticleShader_Full` |
| 轻量（152B） | `off_141B27368` | `ParticleShaderEffect_dtor_Light` | `ShaderEffect_SetupParticleShader` |

- **工厂 `ParticleRenderList_CreateShaderEffects`（sub_1412B25F0）**：从渲染上下文 `*(mgr+48)` 的两条预排序图元向量各建一类效果——`vector@+32/+40` → `sub_141196540(224)` + `*obj=&off_141B266B0`（重量）；`vector@+48/+56` → `sub_141196540(152)` + `*obj=&off_141B27368`（轻量）。因为重量类多出调色板/侵蚀/反射/第二纹理等字段（结构体更大→224B），所以“用 Full 还是 Simple”由图元被上游归入哪条向量决定。
- **分发点**：`ParticleShaderEffect_Prepare_Heavy`(sub_1412C3FE0) / `ParticleShaderEffect_Prepare_Light`(sub_1412C48B0) 中同一句 `(*(*a1 + 8))(a1, a2, a3, a4)` 即虚表 +8 调用，按对象实际类型解析为 `_Full` 或精简版。
- **继承证据**：`ParticleShaderEffect_dtor_Light`/`_Heavy` 末尾都链到基类析构 `sub_1412A7000`（其重置 `*obj=&off_141B266A0` 并释放 11 个纹理/资源槽），证明两类共享同一基类。

## 上游分类条件：发射器 `Type` 字段（Simple / Complex）

“图元被归入哪条向量”在**配置加载阶段**就已由 .troy 文件的发射器 `Type` 字段决定（`ParticleSystem_LoadEmitters` / sub_1412CE0F0）：

```c
// 循环 GroupPart{n}，逐个发射器：
v10 = ParticleEmitterDescriptor_Init(sub_141196540(1136));   // 1136B 描述符
ParticleEmitter_ParseCommon(a1, group, n, v10);             // 公共字段
// 读配置项 Type，默认 "Complex"
read "System"."Type" -> v40 (default "Complex")
if (v40 == "Simple") {
    ParticleEmitter_ParseSimple(a1, group, v10);           // 轻量路径
    push v10 -> C+48 向量（轻量/152B）
} else {  // "Complex" 或其他（默认）
    ParticleEmitter_ParseComplex(a1, group, v10);          // 重量路径
    // 额外读 Override-Offset/Rotation/Scale -> v10[169..177]
    push v10 -> C+32 向量（重量/224B）
}
```

因为 `Type="Simple"` 的发射器被推入 `C+48`、而 `Type="Complex"`（默认）推入 `C+32`，而工厂 `ParticleRenderList_CreateShaderEffects` 又分别为 `C+48`→轻量类(`SetupParticleShader`)、`C+32`→重量类(`SetupParticleShader_Full`)，所以**发射器配置的 `Type` 字段就是决定粒子最终走 Simple 还是 Full 粒子 shader 的上游总开关**（默认 Complex → Full）。

**完整选择链**：.troy 配置 `Type` → `ParticleSystem_LoadEmitters` 分类 → `C+32`(重)/`C+48`(轻) 向量 → `ParticleRenderList_CreateShaderEffects` 建重/轻 ShaderEffect 对象 → `Prepare_Heavy`/`_Light` 的 `(*(*a1+8))(...)` 虚表 +8 装配 `Full`/精简版。

shader 常量名（TEXTURE_INFO/kColorFactor/SLICE_RANGE/PARTICLE_DEPTH_PUSH_PULL/vFresnel/DIFFUSE_MAP/FALLOFF_TEXTURE 等）与 `shaders/hlsl/particlesystem` 下解包的 quad/mesh/distortion 变体 cbuffer/资源绑定名一致。
