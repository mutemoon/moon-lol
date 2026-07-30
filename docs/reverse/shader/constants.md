# 深度转换 & 软粒子 & 切片 & 默认纹理常量

> 上级：[shader.md](../shader.md)

## cDepthConversionParams（`ShaderEffect_SetDepthConversionParams` / sub_141323730）

始终计算，无固定默认值。令 near=`fmaxf(*(a1+12),1e-6)`，far=`fmaxf(*(a1+16),1e-6)`：

```
cDepthConversionParams = float4( 1/near, -(far-near)/(far*near), 0, 0 )
```

同时上传 `mProj` / `mWorld`(单位阵 xmmword_141E49AA0)。

## 软粒子（`ShaderEffect_SetSoftParticleParams` / sub_1412E39D0）

源结构体 = 解包配置 `VfxSoftParticleDefinitionData`（指针 = `descriptor->softParticleParams`，+48，**默认 NULL**）。已在 IDA 建模为 `VfxSoftParticleParams`(20B) 并把 `descriptor.softParticleParams` 从 `void*` 改为该类型指针，`SetSoftParticleParams` 反编译现以命名字段渲染（`softParticleParams->beginIn` 等）：

| 偏移 | 类型 | 配置字段(camelCase) | 组装去向 |
|------|------|------|------|
| +0x00 | float | `beginIn` | 原样 → cSoftParticleParams.x |
| +0x04 | float | `beginOut` | >0 ? beginOut : 1e8 → .y |
| +0x08 | float | `deltaIn` | 1/max(deltaIn,1e-8) → .z |
| +0x0C | float | `deltaOut` | 1/max(deltaOut,1e-8) → .w |
| +0x10 | byte | `unk_0x3bf176bc` (mode) | 选择 cSoftParticleControl |

> 语义解读：`begin*` 是淡出起始阈值（着色器直接用），`delta*` 是淡出宽度（取倒数 1/delta 成为着色器里的缩放系数），配合公式 `fade = saturate((depth - begin) * (1/delta))`。`beginOut` ≤0 时退化为 1e8（=禁用远端淡出）。字段名↔偏移的绑定由「布局精确匹配(4×f32+1×u8) + 上述语义自洽」确认。

**默认 NULL → 软粒子关闭，两个常量不上传。** 开启时取值：

```
cSoftParticleParams  = float4( beginIn,
                               beginOut > 0 ? beginOut : 1.0e8,
                               1.0 / max(deltaIn,  1e-8),
                               1.0 / max(deltaOut, 1e-8) )
cSoftParticleControl 先清零，再按 mode = unk_0x3bf176bc:
  mode 0 : (0, 1, 0, 1)
  mode 1 : (0, 1, 1, 0)
  mode 2 : (1, 0, 0, 1)
  mode≥3 : (0, 0, 0, 0)
```

`SOFT_PARTICLES` define 在 `BuildShaderPathAndDefines`(sub_1412DD450) 中 `pass==0`(正常 quad pass) 且 `*(*(a2+0x3B3)) && *(a2+0x254)>=2 && descriptor->softParticleParams!=0` 时添加。

## 切片技术（`QUAD_PS_Slice` / `SLICE_RANGE`）

配置 f32 `sliceTechniqueRange` 反序列化到描述符 `+132`（已在 IDA 重命名为 `sliceTechniqueRange`），同时驱动两处逻辑：

**(A) 着色器变体选择**（`BuildShaderPathAndDefines`，行 404-417）：当 `v53` 为真时，向路径构建器追加一个 **额外的切片 PS 技术**（不替换基础 QUAD_PS）：
- quad → `ParticleSystem/QUAD_PS_Slice.ps`
- mesh → `ParticleSystem/MESH_PS_Slice.ps`
- skinned → `SkinnedMesh/PARTICLE_PS_Slice.ps`

`v53` 默认 0，仅在同时满足下列条件时置为 `v52 = (sliceTechniqueRange != 0.0)`：
- `pass==0`（行 191 排除 distortion pass a3∈{2,3}、行 193 排除 shadow pass a3==1）
- renderMode 未走禁用分支（行 234 `if(v60 && (v60-3)>2) goto LABEL_89` 等会跳过）

**(B) 常量上传**（`SetupParticleShader(_Full)`）：当 `sliceTechniqueRange > 0.0` 时上传 `SLICE_RANGE`（只有 x/y 有意义，即 vec2）：

```
SLICE_RANGE = float4( range, 1.0/(range*range), 0, 0 )   // range = sliceTechniqueRange
```

> 路径追加用 `!= 0.0`(v52)、常量上传用 `> 0.0`：正值时两者一致；负值（异常）会编译 slice 变体但不上传常量。sub_1412B7330 直接 `SetConstantVec4ByName`；sub_1412B7EA0 经 `DeclareConstantUsed` + `SetConstantVec4ByHandle` 上传，值一致。

## 颜色重映射 ramp 默认纹理（`PIXEL_COLOR_REMAP_RAMP`）

`PIXEL_COLOR_REMAP_RAMP` 是粒子 PS 的一个**纹理槽名**（经 `D3D_GetDevice()` 虚表偏移 512 按名解析成句柄并缓存），本身不含像素；其“默认值”取决于未配置 remap 时绑进该槽的纹理。

**绑定逻辑**（`ParticleShader_BindColorRemapRampOrDefault` / sub_1412D8920）：当粒子无自定义 color-remap（flag `descriptor+168 & 0x10 == 0`）时，把全局纹理注册表 `g_RenderTextureRegistry`(qword_141ED9EC8) 的 **`+192`** 槽纹理绑到 `PIXEL_COLOR_REMAP_RAMP`。

**默认纹理来源**（`RenderTextureRegistry_CreateSystemDefaults` / sub_141304770，在管理器构造时调用）：用 `D3D_GetDevice()` 虚表 +200(2D)/+224(cube) **程序化创建 8 张 1×1 系统默认纹理**（无 WAD/文件加载）：

```c
v93 = 0;                 // 1 个 texel，4 字节全 0
v15[0] = &v93;           // 像素数据指针（单像素）
tex = D3D_GetDevice()->vtable[200](device, 0/*RGBA8*/, 1/*1x1*/);
sub_141307F10(mgr, ..., "texture_registry_system_default_black", 0, tex, a2);
mgr[24] = tex;           // mgr[24] == +192
```

| 槽偏移 | 名称字符串 | 1×1 像素值 |
|------|------|------|
| +184 | `texture_registry_system_default` | 0xFFFFFFFF 白 |
| **+192** | **`texture_registry_system_default_black`** | **0x00000000 纯黑（= PIXEL_COLOR_REMAP_RAMP 默认）** |
| +200 | `texture_registry_system_default_black_opaque` | 0xFF000000 不透明黑 |
| +216 | `texture_registry_system_default_2d_shadow` | fmt17，值 1.0f |
| +232 | `texture_registry_system_streaming_2d_placeholder` | 0x00000000 |

> 证据链闭环：创建（texel=0、名 `..._default_black`）→ 存入 `mgr[24]`(=+192) → 从 `*(g_RenderTextureRegistry+192)` 绑定到 `PIXEL_COLOR_REMAP_RAMP`。因 `mgr[24]=mgr+24×8=+192` 与绑定处读取偏移精确对齐，故 **默认值 = 程序生成的 1×1 纯黑图（RGBA 全 0，即“透明纯黑”，采样得 (0,0,0,0)）**。注意区别于 +200 的 `black_opaque`(0xFF000000)。

## 该 ramp 的采样器：`Clamp_No_Mip` 的默认状态

`PIXEL_COLOR_REMAP_RAMP` 官方就用共享采样器 `Clamp_No_Mip` 采样，其真实采样态由绑定调用决定，而非同名字符串。

**名字来源**：二进制里不存在 `Clamp_No_Mip` 字符串（IDA 字符串检索 0 命中），因为这个名字只活在 shader 源码——它是 HLSL 的共享 `SamplerState` 声明，GLSL 转译后并入 combined 形式 `CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip`（`assets/reverse/shaders/particlesystem/quad_slice/ps/BASE.frag` 核实，粒子系全族的共享采样器只有这一个）。

**采样态来源**：引擎侧真正决定采样状态的是绑定调用。`ParticleShader_BindColorRemapRampOrDefault`(sub_1412D8920) 对该槽的调用是：

```c
ShaderResourceSlot_BindTexture(slot, tex, 1, 1, 0, 0, 1, 1)
```

因为 `ShaderResourceSlot_BindTexture`(sub_1413CCC40) 把尾部 6 参写入槽位 `+16..+20`，而 [assembly-full.md](assembly-full.md) 的 BindTextureSlot 一节已确证 `+16=0x01, +17=0x01`（即 `0x0101`）= 线性过滤 + clamp 寻址、`+18=0` = 关 mip，所以逆向侧 `Clamp_No_Mip` 的默认状态为：

| 属性 | 游戏值 |
|------|------|
| 过滤 | Linear |
| 寻址 | Clamp |
| Mip | 关闭（与名字 `No_Mip` 一致） |

> 与 `addressMode` 未显式设置即默认 Clamp、`mipFilter: 0` 显式关 mip 的 shareddata 定义（见 X3DSharedSamplerDef `0xa6bded4f`）完全吻合。

**项目侧对齐**：早期 `as_bind_group` 里只有 `XXX__SMP` 后缀的采样器才取配对贴图自带 sampler，而 `Clamp_No_Mip_SharedSampler` 无该后缀，所以恒回退到 Bevy 的 `fallback_image.d2.sampler`（Nearest + ClampToEdge），过滤模式与游戏（Linear）不一致——只是当时绑的是 1×1 透明黑贴图，Nearest 与 Linear 采样结果相同，故无可见差异。现已修复：具名共享采样器从 shareddata 建为 Linear 采样器，未命中时回退到预建的“游戏默认采样器”（Linear + Clamp + Linear mip），与游戏默认行为一致。
