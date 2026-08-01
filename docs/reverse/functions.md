# 函数命名记录

> 仅记录逆向过程中碰到并命名的函数 / 数据地址（速查表）。
> Shader 系统的详细分析（数据流、结构体布局、默认值、装配逻辑、DXBC 加载）见 [shader.md](shader.md)。

## Shader 系统相关函数

| 函数 | 命名 | 说明 |
|------|------|------|
| sub_1412DD450 | `ShaderEffect_BuildShaderPathAndDefines` | 构建 shader 文件路径和预处理器定义列表（按 renderType/pass 选 VS/PS 变体、追加宏，含 SOFT_PARTICLES 条件）。详见 [shader/assembly-full.md](shader/assembly-full.md) |
| sub_1412B7330 | `ShaderEffect_SetupParticleShader_Full` | 完整版粒子 shader 装配（VS/distortion+palette 变体）。详见 [shader/assembly.md](shader/assembly.md)、[shader/assembly-full.md](shader/assembly-full.md) |
| sub_1412B7EA0 | `ShaderEffect_SetupParticleShader` | 精简版粒子 shader 装配（PS/simple-color 变体），与 Full 版同一虚表契约。详见 [shader/assembly.md](shader/assembly.md) |
| sub_1412E39D0 | `ShaderEffect_SetSoftParticleParams` | 上传软粒子常量 `cSoftParticleParams`/`cSoftParticleControl`；`descriptor.softParticleParams` 为 NULL（默认）时直接返回（软粒子关闭）。详见 [shader/constants.md](shader/constants.md) |
| sub_141323730 | `ShaderEffect_SetDepthConversionParams` | 上传 `mProj`/`mWorld`(单位阵) 及 `cDepthConversionParams`（由相机 near/far 计算，非固定默认值）。详见 [shader/constants.md](shader/constants.md) |
| sub_1412B25F0 | `ParticleRenderList_CreateShaderEffects` | 工厂：为重量(224B, vtable=off_141B266B0)/轻量(152B, vtable=off_141B27368) 两类图元创建 ShaderEffect。详见 [shader/assembly.md](shader/assembly.md) |
| sub_1412C3FE0 | `ParticleShaderEffect_Prepare_Heavy` | 重量粒子效果预处理（算深度边界/排序键），末尾虚表 +8 分发 → `ShaderEffect_SetupParticleShader_Full` |
| sub_1412C48B0 | `ParticleShaderEffect_Prepare_Light` | 轻量粒子效果预处理（152B 类），虚表 +8 分发 → `ShaderEffect_SetupParticleShader` |
| sub_1412A9EC0 / sub_1412AA6B0 | `ParticleShaderEffect_dtor_Heavy` / `_Light` | 两类析构，均链到基类析构 `sub_1412A7000`（重置 `*obj=&off_141B266A0` 并释放 11 个纹理/资源槽），证明重/轻共享同一基类 |
| sub_1412CE0F0 | `ParticleSystem_LoadEmitters` | .troy 粒子系统顶层解析器：逐发射器建 `ParticleEmitterDescriptor`(1136B)，按 `Type`(Simple/Complex，默认 Complex) 分类到轻/重量向量——即 Simple/Full shader 的总开关。详见 [shader/assembly.md](shader/assembly.md) |
| sub_1412CE810 | `ParticleEmitter_ParseCommon` | 解析发射器公共字段到 descriptor（Simple/Complex 都走） |
| sub_1412D0730 | `ParticleEmitter_ParseSimple` | 解析 `Type=="Simple"` 发射器专有字段（轻量路径） |
| sub_1412CEB10 | `ParticleEmitter_ParseComplex` | 解析 `Type=="Complex"`(默认)发射器专有字段（重量路径），含 Override-Offset/Rotation/Scale |
| sub_1411477B0 / sub_141147EC0 / sub_141147470 / sub_141147670 / sub_141147A20 / sub_141147CA0 | 配置属性读取器（string/bool/int/float/vec2/vec3） | 签名 `(ctx,out,section,key,default)`，最后一个参数为配置缺失时的默认值。详见 [shader/deserialization.md](shader/deserialization.md) |
| sub_1412FFED0 | `ShaderEffect_SetConstantVec4ByName` | 按常量名上传一个 float4 |
| sub_1412FFF90 | `ShaderEffect_DeclareConstantUsed` | 按名声明/标记 shader 常量被使用（含数量） |
| sub_14131BC10 | `ShaderEffect_SetConstantVec4ByHandle` | 按句柄上传 float4 |
| sub_1413C6E60 | `Shader_HashConstantName` | 常量名 → 句柄哈希 |
| sub_1412AD400 | `ShaderEffect_BindTextureSlot` | 把描述符里的纹理源绑定到指定 shader 采样槽（索引 1..11）。详见 [shader/assembly-full.md](shader/assembly-full.md) |
| sub_14130B300 | `ShaderEffect_GetTextureSlot` | 按句柄取纹理绑定槽 |
| sub_1412C73E0 | `ParticleEmitter_GetSecondaryTextureInfo` | 取第二纹理(TEXTURE_INFO_2)的尺寸信息 |
| sub_141304770 | `RenderTextureRegistry_CreateSystemDefaults` | `g_RenderTextureRegistry` 构造时程序化创建 8 张 1×1 系统默认纹理（无 WAD/文件加载）。详见 [shader/constants.md](shader/constants.md) |
| sub_1412D8920 | `ParticleShader_BindColorRemapRampOrDefault` | 粒子无自定义 color-remap 时把系统默认纯黑纹理绑到 `PIXEL_COLOR_REMAP_RAMP` 槽。详见 [shader/constants.md](shader/constants.md) |
| sub_1413CCC40 | `ShaderResourceSlot_BindTexture` | 纹理槽绑定器：`if(!tex) tex=slot[6];`，尾部 6 个参为采样器状态（非尺寸） |
| sub_141306BA0 | `ShaderManager_LoadShader` | 加载/缓存 shader 对象，通过路径哈希查找 |
| sub_14130F940 | `ShaderManager_FinalizeShader` | 最终确定 shader，创建 D3D shader 对象 |
| sub_141312C30 | `ShaderPass_CompileOrGetCached` | 从预编译缓存中查找或创建 D3D shader 对象 |
| sub_1413018C0 | `Shader_MergeDefinesAndCompile` | 合并所有 defines，从 WAD 加载对应 DXBC 变体 |
| sub_141312FE0 | `ShaderCompileCache_Lookup` | 按 VS路径+PS路径+defines哈希 查找编译缓存 |
| sub_14130D150 | `ShaderBlobCache_Lookup` | 查找/创建 shader blob 缓存 |
| sub_14130ED40 | `ShaderVS_LoadFromWad` | 从 WAD 加载顶点着色器 DXBC |
| sub_14130DAC0 | `ShaderPS_LoadFromWad` | 从 WAD 加载像素着色器 DXBC |
| sub_14130B700 | `Shader_ResetShader` | 重置 shader 状态 |
| sub_141302D40 | `Shader_CopyFromTemplate` | 从模板 shader 复制数据 |
| sub_1413BA0E0 | `D3DShaderCompiler_Init` | 初始化 d3dcompiler_47.dll，加载 D3DReflect |
| sub_1413C0D50 | `D3DShaderCompiler_InitReflectionBuffers` | 初始化 shader 反射缓冲区 |
| sub_1413CA640 | `D3D_GetDevice` | 获取 D3D 设备对象 |
| sub_1413CA860 | `ShaderWad_LoadDx11File` | 拼接 .dx11 后缀并从 WAD 加载 shader 文件 |
| sub_1413C8030 | `ShaderWad_LoadByHash` | 按哈希从 WAD 中查找并加载 shader 数据 |
| sub_1413C2D50 | `ShaderWad_LoadAndCreateD3DShader` | 从 WAD 加载 DXBC 并创建 D3D shader 对象 |
| sub_1405E3D10 | `WadArchive_Init` | 初始化 WAD 归档列表（含 ShaderCache.dx11.wad） |
| sub_141145C10 | `StringPool_Insert` | 字符串池化/去重（用于 shader 路径） |
| sub_1413BBC50 | `ShaderDefinesList_AddDefine` | 向 defines 列表添加一个 define |
| sub_1413BDF50 | `ShaderDefinesList_AddDefinePair` | 向 defines 列表添加 key-value 对 |
| sub_1411A8DB0 | `StringBuilder_Append` | 字符串构建器追加 |
| sub_1411A76E0 | `StringBuilder_Concat` | 字符串构建器拼接 |
| sub_1411A77D0 | `StringBuilder_AppendChar` | 字符串构建器追加字符 |
| sub_1411A6E20 | `StringBuilder_Free` | 释放字符串构建器 |
| sub_1411A8C60 | `StringBuilder_FromObject` | 从对象构建字符串 |
| sub_1411AE960 | `StringBuilder_Format` | 格式化字符串构建器 |
| sub_14130F350 | `ShaderEffect_IsSkinningRequired` | 检查是否需要蒙皮处理 |
| sub_1412E3AD0 | `ShaderPass_IsValid` | 检查 shader pass 是否有效 |
| sub_1412E1250 | `ShaderPass_GetRenderState` | 获取 shader pass 的渲染状态 |
| sub_14131E100 | `ShaderPass_UpdateRenderState` | 更新 shader pass 的渲染状态 |
| sub_141300270 | `ShaderPass_GetDefines` | 获取 shader pass 的 defines |
| sub_1412FFD00 | `ShaderDefines_GetData` | 获取 defines 数据 |
| sub_141306DE0 | `ShaderManager_LoadShaderFromFile` | 从文件加载 shader |
| sub_1412F9840 | `ShaderMaterial_Apply` | 应用 shader 材质 |
| sub_1412F7060 | `ShaderMaterial_Update` | 更新 shader 材质参数 |

## 粒子系统 Quad 网格相关函数

| 函数 | 命名 | 说明 |
|------|------|------|
| sub_1412C1340 | `ParticleEmitter_FillQuadVertices` | 粒子发射器批量填充动态顶点/索引缓冲：每个粒子生成一个 billboard 四边形（4顶点+6索引），对应 QUAD_VS.vs |
| sub_1412D70B0 | `ParticleEmitter_FillMeshQuadVertices` | 网格粒子(MESH_VS)路径的四边形顶点填充，40字节/顶点（比 quad 路径多12字节法线/UV数据） |
| sub_1412B5370 | `Particle_ComputeBillboardAxes` | 计算粒子 billboard 的 right/up 轴向量（相机朝向或按旋转角查表） |
| sub_14129BB00 | `Particle_FillQuadVertexUVs` | 填充 quad 4个顶点的 UV/附加数据（28字节顶点格式的后12字节，6个WORD） |
| sub_14129BE00 | `Particle_FillMeshVertexUVs` | 填充 mesh 粒子4个顶点的 UV/附加数据 |
| sub_1411847B0 | `Vec3_Normalize` | 归一化 vec3 |
| sub_14116D130 | `Color_PackToDWORD` | 将颜色打包为 DWORD（用作顶点色） |
| sub_1412B6FD0 | `ParticleEmitterFactory_Create` | 按类型(0-12)创建粒子发射器对象（分配内存+设置vtable） |
| sub_1412A4530 | `ParticleEmitterDescriptor_Init` | 粒子发射器描述符构造函数：分配24字节子对象，主vtable=off_141B28B98(FillQuadVertices所在)，子vtable=off_141B28CC8；初始化默认贴图(DefaultColorOverlifetime/Falloff)及各曲线参数。默认值表见 [shader/deserialization.md](shader/deserialization.md) |

## VFX 触发 / ResourceResolver 相关函数

> 详细分析（源粒子 vs 目标粒子 `*_tar` 的触发机制、Fiora_BA1_tar 调查）见 [vfx-trigger.md](vfx-trigger.md)。

| 函数 | 命名 | 说明 |
|------|------|------|
| sub_1401A3980 | `ParticleEventData_registerType` | ParticleEventData 反射注册。type hash=88265757 size=88；字段 mEffectKey@24(0xF6386280)、mEnemyEffectKey@28(0x98DF0FB4)、mEffectName@32、mParticleEventDataPairList@48 |
| sub_1401A85B0 | `ResourceResolver_registerType` | ResourceResolver 反射注册。type hash=1923179998；resourceMap@8(0xD2F58721)=map<触发名Hash,粒子系统>，`Fiora_BA1_tar`(0x4E7E421B) 即在此解析 |
| sub_1411C4280 | `ParticleEventData_ctor` | ParticleEventData 构造，vtable=off_141B1D1A8；"fire" 为虚函数，由动画 clip 播放引擎间接调用（无数据 xref） |

## 关键数据结构

| 地址 | 命名 | 说明 |
|------|------|------|
| off_141B28B98 | `ParticleEmitter_VTable` | 粒子发射器主虚表。+0x20 槽 = `ParticleEmitter_FillQuadVertices`(sub_1412C1340)，按 `*(BYTE*)(batchCtx+40)` 分 quad(28B/顶点)/mesh(40B/顶点) 路径填充动态 VB/IB |
| off_141B266B0 / off_141B27368 | 重量(224B) / 轻量(152B) ShaderEffect 虚表 | +0=析构、+8=SetupParticleShader(_Full)。详见 [shader/assembly.md](shader/assembly.md) |
| off_141B266A0 | ShaderEffect 基类虚表 | 基类析构 `sub_1412A7000` 重置到该虚表 |
| off_141B26570 | 纹理槽 → 常量名映射表 | slot 0..11 → TEXTURE/PARTICLE_COLOR_TEXTURE/... 详见 [shader/assembly-full.md](shader/assembly-full.md) |
| byte_141E4D7D0 | `ShaderPass_RenderPassTable` | 7 项×48B，每项 [+0]=passId [+8]=define 名 |
| qword_141ED9EC8 | `g_RenderTextureRegistry` | 全局纹理注册表/管理器（含 1×1 系统默认纹理槽）。详见 [shader/constants.md](shader/constants.md) |

## IDA 结构体定义（已写入 IDB 本地类型库）

通过 `idc.parse_decls` 定义、`idc.SetType`/lvar 应用到相关函数，使反编译显示命名字段。完整字段布局与复现脚本见 [shader/structs.md](shader/structs.md)。

| 结构体 | 大小 | 用途 | 应用到 |
|------|------|------|------|
| `ShaderEffect` | 512B | shader 效果对象；`+16`=descriptor 指针、`+128`=shader、`+48..104`=纹理槽、`+28/36/40`=passIndex0/1/2 | 4 个函数的 `a1` |
| `ParticleEmitterDescriptor` | 1136B | 运行时粒子描述符（由 `ParticleEmitterDescriptor_Init` 构造），含 texDiv/colorModulate/alphaRef/depthBias/miscFlags 等 | 经 `ShaderEffect.descriptor` 传播 |
| `ShaderPassVariantData` | 136B | 每 pass 的 shader 变体记录：`+48/+56`=VS/PS 路径句柄、`+64`=defines 动态数组(24B/项)、`+72`=defineCount、`+80`=definesDirty | `BuildShaderPathAndDefines` 的 `a4` |
| `ParticleRenderContext` | 952B | 渲染上下文：`+208`=passStateTable(40B/项,+32=state)、`+240`=resourceTableA(16B/项,+8=obj)、`+256`=resourceTableB(16B/项)、`+596`=detailLevel、`+947`=softParticleAllowed | `BuildShaderPathAndDefines` 的 `a2` |
| `VfxSoftParticleParams` | 20B | 软粒子参数（beginIn/beginOut/deltaIn/deltaOut/mode），`descriptor.softParticleParams` 指向该类型 | `SetSoftParticleParams` |
