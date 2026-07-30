# 流程

创建材质对象 -> prepare -> specialize -> as_bind_group

# 旋转

英雄联盟是左手坐标系

# 着色器架构与 SPIR-V 转译集成指南

## 着色器变体与结构体匹配规则

- 英雄联盟客户端中不同分支条件下的着色器包含不同的结构体定义与全局变量布局
- 对于变体结构体，统一采用包含字符最长与字段最全的结构体声明进行对接
- 对于 Uniform 与 Input Output 接口，按照物理内存槽位与全局映射规则统一布局

## Vulkan SPIR-V 资源绑定与槽位隔离

### 描述符集冲突背景

- 从英雄联盟客户端提取的 `DXBC` 字节码通过 `dxbc-spirv` 工具转译为 `SPIR-V` 时，触发 `Vulkan` 验证层拦截
- 验证层报错标识为 `VUID-VkGraphicsPipelineCreateInfo-layout-07988`
- 独立转译导致顶点着色器与像素着色器均将资源默认映射在 `DescriptorSet` 0 且从 `Binding` 0 自增分配
- 顶点着色器占用 `Binding` 0 与 1 存放常量缓冲区，像素着色器占用 `Binding` 0 与 1 存放纹理与采样器，合并管线时物理重叠
- `Bevy` 引擎规范将 `Set` 0 预留给通用 `View`，`Set` 1 预留给 `Mesh View`，`Set` 2 预留给多实例，自定义材质资产必须绑定在 `Set` 3

### 编译器源码改造与选项扩充

- 修改 `spirv_mapping.h` 与 `spirv_mapping.cpp` 中的 `BasicResourceMapping` 类
- 增加成员变量 `m_setIndex` 与 `m_bindingOffset`
- 改写 `mapDescriptor` 虚方法，采用指定的 `Set` 索引并将绑定槽位改为 `m_bindingOffset` 加上自增描述符索引
- 修改 `tools/dxbc_compiler.cpp` 中的 `Options` 结构，增加 `--set` 与 `--binding-shift` 命令行解析参数

### 构建与提取器逻辑改造

- 项目新增 `CMakeLists.txt` 替代 `Meson` 构建配置，生成 Release 可执行文件并覆盖至 `assets/tools/dxbc_compiler.exe`
- 修改 `examples/extract_shaders.rs` 中的 `process_toc` 函数
- 根据 `shader_toc.shader_type` 区分顶点着色器与像素着色器
- 顶点着色器转译参数附加 `--set 3`
- 像素着色器转译参数附加 `--set 3 --binding-shift 2`
- 使得顶点常量缓冲区分布在 `Set` 3 的 `Binding` 0 与 1，像素采样器与纹理平移映射在 `Set` 3 的 `Binding` 2 至 7

### Bevy 材质管线对齐

- 在 `ParticleMaterialQuad` 结构体中使用 `AsBindGroup` 宏声明槽位
- 顶点 Uniforms 绑定在 `Binding` 0
- 像素 Uniforms 绑定在 `Binding` 1
- 采样器分别绑定在 `Binding` 2 至 4
- 纹理分别绑定在 `Binding` 5 至 7

## DXBC RDEF 内存布局解析与符号导出

### RDEF 模块解析设计

- 新增 `dxbc_rdef.h` 与 `dxbc_rdef.cpp` 中的 `RdefParser` 模块
- 读取 `RDEF` 或 `RD11` Chunk，跳过 8 字节头部精确定位 Payload 指针
- 提取常量缓冲区的名称、字节大小、变量列表、起始字节偏移量 `startOffset` 与尺寸 `size`
- 提取资源绑定列表中的资源名称、绑定类型与绑定点 `bindPoint`

### 调试故障排查与修复阶段

#### 阶段 1 无名称
- 原版未解析 `RDEF` 块，默认导出匿名符号
- 修复方案为在 `dxbc_resources.cpp` 的 `emitDebugName` 中优先使用 `RDEF` 解析出的原始标识符替换默认编号

#### 阶段 2 顶级成员缺失
- 原版在 `emitDclCbv` 中为常量缓冲区外层套用了 `defStructWrapper` 包装结构体
- `spirv-reflect` 仅反射最外层结构体导致内部成员被隐藏
- 修复方案为剔除外层包装，直接暴露顶层 `Block` 结构体

#### 阶段 3 名字偏移走样
- 常量缓冲区包含 `float3` 与 `uint` 等非 16 字节成员
- 先前计算使用 `startOffset / 16` 作为成员索引导致累计走样
- 修复方案为在 `spirv_builder.cpp` 的 `emitDebugMemberNames` 中遍历 `structType->byteOffset(m)` 匹配真实物理字节偏移

#### 阶段 4 名称乱码前缀
- `Shader Model` 5 的 `RDEF` 变量记录结构体大小为 40 字节，先前按 `Shader Model` 4 的 24 字节步长遍历导致越界
- 先前误将操作数 1 的字节偏移量当作字符拼接入字符串
- 修复方案为在 `dxbc_rdef.cpp` 中检测 Shader Model 版本，版本高于 5.0 时按 40 字节步长解析变量记录

### 复合类型与未引用字段表现

- 矩阵类型如 `float4x4` 连续占用 4 个 16 字节槽位，`RDEF` 仅在起始点记录一次变量名，后续行由转译器自动推导
- 常量缓冲区中未被逻辑读写的字段在内存布局中仍保留物理槽位，并由验证工具标记为未引用

## cbuffer 矩阵行主序写入约定

### 坑：矩阵被降级为 4 个 vec4，反射看不出主序

- `dxbc-spirv` 转译 `float4x4` 时不发 `OpTypeMatrix`，而是降级为 4 个连续 `vec4` 成员（offset `0`/`16`/`32`/`48`）加 `dp4_f32` 手写点积
- 后果：`spirv-dis` 与 `naga` 反射只能看到"4 个 `vec4` @ 0/16/32/48"，读不出行/列主序；主序信息埋在 `main` 的 `dp4` 接线里，必须读 `main` body 才能判定
- 根因是 `DXBC` 原生矩阵指令即 `dp4` 作用于行寄存器，转换器做 1:1 一致 lowering，非 `bug`

### 判定：dp4(pos, member_i) 即行主序

- 顶点着色器典型写法 `SV_Position_i = dp4(pos4, mProj[i])`，`dp4` 为四元点积
- `result_i = pos4 · mProj[i]` 等价于 `(M·pos4)_i` 仅当 `mProj[i]` 为矩阵第 `i` 行，故为**行主序**
- 若出现 `OpVectorTimesScalar` 扇入 `B0·v.x + B1·v.y + ...` 则为列主序（本套 `shader` 未出现）

### 写法：一律转置后按列写出

- `Bevy` `Mat4` 为列主序，`to_cols_array()` 按列写出；直接写入等于存了转置，相机一动即畸变（静止时可能误判为正常）
- 正确写法：`mat.transpose().to_cols_array()`（转置的列 = 原矩阵的行 = 匹配 `shader` 行主序）
- 内容须为 `clip_from_world = projection.get_clip_from_view() * global_transform.to_matrix().inverse()`：`shader` 把 `mProj` 作用于世界坐标 `ATTRIBUTE_WORLD_POSITION`，仅写 `clip_from_view` 会让世界坐标直接进入透视除法，四边形落在近截面被裁掉
- 非 4×4 成员（如 `vCamera` 为 `vec3`）的 `offset` 仍以 `spirv-dis` 反射为准（如 `quad` 的 `PerFrameVertexCB` 中 `vCamera` @ `64`）

### 全量验证：3885/3885 均为行主序

- 全量扫描 `assets/shaders/hlsl` 下 3885 个 `.spv`：`dp4_f32` 调用 8212 处；`OpVectorTimesScalar` / `OpMatrixTimesVector` / `OpVectorTimesMatrix` / `OpMatrixTimesScalar` 均为 `0`
- 结论：本套 `shader` 矩阵访问 100% 为 `dp4` 行约定，写 `mat4` 一律 `transpose().to_cols_array()`，无需逐个判定
- 验证命令：
  ```sh
  find assets/shaders/hlsl -name '*.spv' | while read p; do spirv-dis "$p" 2>/dev/null; done \
    | awk '/dp4_f32/{d++} /OpVectorTimesScalar|OpMatrixTimesVector|OpVectorTimesMatrix/{c++} END{print "dp4="d, "col="c}'
  ```
- 前提：此为"当前 `dxbc_compiler.exe` + 这套 `shader`"的性质；更换转换器或 `shader` 来源后需重扫确认
- 参考实现：`examples/particle_quad_spirv_dynamic.rs` 的 `update_per_frame_uniforms`

## 变体纹理绑定按声明顺序漂移

### 坑：GLSL 无 layout(binding)，转换器按声明顺序分配且拆分 sampler2D

- 同源的逆向 `GLSL` `.frag` **没有 `layout(binding=N)` 限定符**；`dxbc-spirv` 转译时按**声明顺序**自增分配 `Binding`
- 转换器把每个 `GLSL` `sampler2D` 拆成两个独立资源（`wgpu` 的 image/sampler 分离约定），命名后缀规律固定：
  - `NAME__SMP` = 采样器（`OpTypeSampler`）
  - `NAME__TX` = 纹理（`OpTypeImage`）
  - `NAME_SharedSampler` = 共享采样器状态（`D3D` `SamplerState`，无配对纹理）
  - `NAME_SharedTexture` = 共享纹理（被多个采样器引用，无配对采样器）
- 后果：在前面插入一个纹理，会把其后所有资源的 `Binding` 整体后顶；同一语义纹理在不同变体落在不同 `Binding`

### 证据：0000（最简）vs 0195（最复杂）

| 资源语义 | shader_0000 | shader_0195 |
| --- | --- | --- |
| `TEXTURE`（纹理/采样器） | tex=6 / smp=2 | **tex=15** / smp=6 |
| `PARTICLE_COLOR_TEXTURE` | tex=7 / smp=3 | **tex=16** / smp=7 |
| `PIXEL_COLOR_REMAP_RAMP`（共享纹理） | tex=5 | **tex=13** |
| `Clamp_No_Mip`（共享采样器） | smp=4 | smp=9 |
| `sPalettesTexture` | — | tex=14 / smp=5 |
| `sDepthTexture` | — | tex=10 |
| `FOW_MAP` | — | tex=11 |
| `NAVMESH_MASK_TEXTURE` | — | tex=12 / smp=4 |
| `TEXTUREMULT` | — | tex=17 / smp=8 |
| 资源总数 | 6 | 16 |

- 同一 `Binding` 号语义也不同：`Binding 5` 在 0000 是 `PIXEL_COLOR_REMAP_RAMP`，在 0195 是 `sPalettesTexture__SMP`
- 连 `cbuffer` 布局都变：0195 有 `_Globals@2`/`PerFramePixelCB@3`，0000 没有这两个（最简变体未引用，被转换器 `DCE`）
- 验证命令（列出某变体全部 `(binding, name)`）：
  ```sh
  spirv-dis assets/shaders/hlsl/particlesystem/quad/ps/shader_0195.spv \
    | grep -E 'OpDecorate.*Binding' | sed 's/^[[:space:]]*//'
  ```

### 判定与写法：绑定必须按语义名反射驱动

- `Binding` 号**不可写死**：写死 `5/6/7` 只对被手调的那一个变体（如 `BASE`）有效，换纹理更多的变体即错位
- 正确做法：反射 `SPV` 得到 `(语义名 -> (texture_binding, sampler_binding))`，游戏侧按语义名（`TEXTURE`、`PARTICLE_COLOR_TEXTURE`、`sPalettesTexture`…）供纹理句柄，渲染侧把句柄绑到该变体反射出的 `Binding` 号
- 语义名由剥后缀得到（`__TX`/`__SMP`/`_SharedTexture`/`_SharedSampler`），名字来自 `GLSL` 声明，跨变体稳定，是可靠的反射 `key`

### 动态绑定方案草图（与 `AsBindGroup` 静态布局的冲突）

- `Bevy` 的 `AsBindGroup` 宏在编译期写死 `Binding` 号，一个 `Material` 只能有一套 `bind group layout`
- 而本套 `shader` 每个变体布局都不同 -> 单一固定布局（当前 `ParticleMaterialQuad` / `ParticleMaterialQuadDynamic` 的 `5/6/7`）只服务 `BASE` 变体
- 解法方向：按变体生成独立 `RenderPipelineDescriptor` + `bind group layout`，布局由该变体 `SPV` 反射的 `(语义名 -> binding)` 驱动；或手写 `AsBindGroup` 在运行时按反射构建 `BindGroupLayout`/`BindGroupLayoutEntries`
- 当前生产 `ParticleMaterialQuad::ConditionalMaterialKeyQuad.shader_frag_defs` 写死 `vec![]` 只选 `BASE` 变体，正是该问题的体现：变体 `shader` 可换但布局静态
- **实际落地方案**：下一节的规范绑定表方案（离线把所有变体 `patch` 到同一套绑定，静态 `Material` 即可通吃）

## 规范绑定表方案与跨阶段碰撞

### 方案：规范绑定表 + SPIR-V 原始字改写

- 实现 `crates/lol_render/src/loaders/spirv_remap.rs`：`remap_bindings` 线性扫 `SPV` 指令流，按 `OpName`（指令 5）建 id 到名字的映射（uniform 变量无直接名时经 `OpVariable`（指令 59）回退到其类型名），改写 `OpDecorate`（指令 71）+ `Binding`（枚举 33）的值字；`DescriptorSet`（34）保持不动
- 改写是纯原始字操作，不依赖 `naga`：272 个 quad 变体中 136 个 `naga` 不可解析（这些变体在 `Bevy` 本就不可用），patch 工具对其 skip 计数不 panic
- 规范表（set 3 内 20 槽全 distinct）：vert cbuffer `0/1`，frag cbuffer `18/19`，纹理 `5-13`，采样器 `2/3/4/14-17`；`wgpu` 允许 bind group layout 是 shader 使用 binding 的超集，故一套固定 Material 通吃所有只用子集的变体
- 应用点：提取时 patch（`extract_shaders.rs` 在 `compile_dxbc_to_spirv` 后、写盘前，仅 `toc_name.contains("/quad/")`）；历史产物用 `patch_spirv_bindings.rs` 补打
- 待办坑：仅 quad 变体做了归一；mesh、unlit_decal、distortion、skinnedmesh 等其余 TOC 未归一，仍按声明顺序漂移

### 坑：vert 与 frag 同名 cbuffer 碰撞，粒子全透明不可见

- 症状：`particle_quad_spirv_dynamic` 跑满 250 帧无崩溃、绑定验证全过，但屏幕上看不到粒子
- 根因：frag 0195 的 `$Globals`（64B）与 vert 0000 的 `$Globals`（32B）都落在 set 3 binding 0，两者成员布局完全不同；`wgpu` 把同一 `(set, binding)` 跨阶段合并成一个绑定位，一个 buffer 无法同时满足两种布局
- 细节：Material 最初只喂一份 vert 布局的数据，`frag` 读 `cSoftParticleControl`（0195 反汇编确认其 `[2]` 分量是 alpha 主乘子，offset 16）时读到 vert 的 `PARTICLE_DEPTH_PUSH_PULL` 零字节 -> alpha 恒 0 -> 全透明
- 教训：`naga` 反射验证只查资源 binding 是否规范，**不查喂给 Material 的数据布局是否匹配 shader 期望**；帧数达标 + 无管线报错 ≠ 渲染正确

### 修复：frag cbuffer 拆到独立 binding

- `canonical_quad_binding` 改为阶段感知（`QuadStage::Vertex`/`Fragment`）：vert `$Globals` -> `0`、`PerFrameVertexCB` -> `1`；frag `$Globals` -> `18`、`PerFramePixelCB` -> `19`；纹理与采样器阶段无关不变
- Material 相应拆成 4 个字段：`#[uniform(0, visibility(vertex))]`、`#[uniform(1, visibility(vertex))]`、`#[uniform(18, visibility(fragment))]`、`#[uniform(19, visibility(fragment))]`
- 0195 可见所需 frag 值（由 `diag_quad_wgsl.rs` 的 `naga` HLSL 反汇编确认）：`$Globals.cSoftParticleControl` = `(1,0,1,0)` 使 RGB 走 base 色、alpha 走 base alpha；`PerFramePixelCB.cDepthConversionParams`（offset 80）`= (1,0,..)` 使软粒子深度差项为 0，避免 `NaN×0` 传播
- vert/frag 的 cbuffer 大小与成员布局由 `reflect_spirv` 反射 + HLSL 反汇编双重确认，不要用 vert 布局套 frag

### 验证：截图看像素，不看帧数

- 假阳性教训：250 帧自动退出 + validate PASS 时粒子其实不可见；此后验证一律加截图
- 方式：`bevy::render::view::screenshot::Screenshot::primary_window()` + `save_to_disk` observer，约第 100 帧抓主窗口落盘，再人工看图
- 结果：修复后截图中央出现白色方块粒子（纹理为白色兜底 + RGB 乘子 1），确认 0195 真实渲染
- 残留无害告警：vert 0000 未输出 frag 期望的 Location 3 Component 2 输入（Vulkan 接口校验提示），按规范该输入读未定义值，当前变体未实际消费它，可忽略；换变体若消费到需回补该输出

### 反射实现与验证

- 反射能力：`crates/lol_render/src/particle/reflection.rs` 的 `reflect_spirv` 新增 `AddressSpace::Handle`（`Image`/`Sampler`）反射，输出 `ShaderResourceReflection`；`group_texture_slots` 按语义名配对 texture/sampler
- 小范围验证：`examples/particle_quad_spirv_dynamic.rs` 的 `validate_texture_bindings` 反射 0000 `assert` 硬编码 `5/6/7` 与反射一致，并反射 0195 打印漂移，证明需动态绑定

## 验证与调试结论

- 使用 `spirv-reflect.exe` 验证 `shader_0015.spv`，确认所有字段与绝对物理字节偏移精准对齐且无乱码
- 运行 `extract_shaders` 重新转换全量 256 个着色器
- 绑定红黄蓝绿测试图与纯白纯绿控制变量验证粒子平面渲染，材质调色盘逻辑与颜色混合在 `GPU` 上运行正常，验证层无报错

## 调试案例：$Globals UV 参数全零导致粒子不可见

### 现象

`particle_quad_spirv_dynamic` 示例运行，emitter 正常创建、材质正常挂载，但屏幕上看不到粒子。

### 调试步骤与技巧

#### 第一步：把材质的所有字段 dump 到文件

在 `impl From<&ParticleMaterialQuad> for ConditionalMaterialKeyQuad` 的 `from` 函数里（每次材质用于 pipeline specialization 时都会调用），写入覆盖模式日志（每次只保留最新）：

```rust
// quad.rs — From 实现内
use std::io::Write;
fn fmt_uniform(buf: &RawUniformBuffer256) -> String {
    // 只打印非零 Vec4，避免 256 字节全量输出
    buf.data.iter().enumerate()
        .filter(|(_, v)| **v != Vec4::ZERO)
        .map(|(i, v)| format!("[{}]={:.4?}", i, v))
        .collect::<Vec<_>>().join(", ")
        .or_else(|| Some("(all zero)".into())).unwrap()
}
if let Ok(mut f) = std::fs::File::create("material_debug.log") {
    let _ = f.write_all(log_content.as_bytes());
}
```

**结论**：`uniforms_vert_globals` 和 `uniforms_frag_perframe` 全零。

#### 第二步：用 OnceLock 一次性 dump ShaderMap 布局

在 `update_particle` 系统的 `if let Some(shader_map)` 块里，用 `OnceLock` 保证只在第一帧写一次，避免每帧刷文件：

```rust
use std::sync::OnceLock;
static DUMPED: OnceLock<()> = OnceLock::new();
DUMPED.get_or_init(|| {
    // 遍历 shader_map.0.get(&QuadPs/QuadVs)，
    // 按 binding 排序打印所有成员 [offset] size name
    // 写入 shader_layout_debug.log
});
```

**结论**：`QuadPs` 对应当前 `shader_hash` 的 `blocks` 是空的（`{}`），说明 frag shader 没有任何 uniform block，`uniforms_frag_*` 全零是正常的，不是问题。

#### 第三步：直接看 map.ron 而不是运行时 dump

`assets/shaders/map.ron` 是离线生成的 ShaderMap 序列化文件，可以直接用行号定位：

```powershell
# 找到 QuadPs 块起始行
Select-String -Path 'assets\shaders\map.ron' -Pattern 'QuadPs' | Select-Object -First 3

# 再在该行之后找当前 hash 的条目
Select-String -Path 'assets\shaders\map.ron' -Pattern '17241709254077376921' `
    | Where-Object { $_.LineNumber -gt 196021 } `
    | Select-Object LineNumber, Line | Select-Object -First 5
```

然后用编辑器直接跳到该行查看 `blocks` 内容，**比运行时 dump 更快更直观**。

**结论**：`shader_0001.spv`（当前用的 frag）的 `blocks: {}`——frag 端没有 uniform，不是问题根源。vert shader `$Globals (binding=0)` 有两个成员：

```
TEXTURE_INFO         offset= 0, size= 4   (float)
PARTICLE_DEPTH_PUSH_PULL  offset=16, size=16 (v4float)
```

但这两个参数**从来没有被写入**。

#### 第四步：用 spirv-dis 反汇编确认参数语义

```powershell
spirv-dis assets/shaders/hlsl/particlesystem/quad/vs/shader_0001.spv `
    | Select-String "TEXTURE_INFO|PARTICLE_DEPTH|OpMemberName|OpTypeStruct|OpAccessChain"
```

从 SPIR-V 反汇编还原出 $Globals 的完整结构：

```
$Globals_buf = OpTypeStruct %float %v2float %uint %v4float
  member 0  offset= 0   TEXTURE_INFO        (float)   = num_cols
  member 1  offset= 4   (匿名 v2float)                = [1/num_cols, 1/num_rows]
  member 2  offset=12   (匿名 uint, padding)
  member 3  offset=16   PARTICLE_DEPTH_PUSH_PULL (v4float) = 深度偏移，全零即可
```

shader main 函数逻辑（伪代码）：

```glsl
// 用 member1.x（= 1/num_cols）和 TEXTURE_INFO（= num_cols）计算帧的行列号
row_idx = floor(frame * member1.x)          // = floor(frame / num_cols)
col_idx = frame - row_idx * TEXTURE_INFO    // = frame mod num_cols

// 映射到 UV [0,1] 坐标
out_uv.x = (col_idx + uv_base.x) * member1.x   // = col_idx / num_cols
out_uv.y = (row_idx + uv_base.y) * member1.y   // = row_idx / num_rows
```

**根因：全零时 `member1 = [0, 0]`，UV 被乘以 0，整张纹理只采样 `(0,0)` 一个像素点。**

### 根因

[`emitters/quad.rs`](../../crates/lol_render/src/particle/emitters/quad.rs) 第 37-39 行早就算好了正确的值：

```rust
let _texture_info_vec4 = vfx_emitter_definition_data
    .tex_div
    .map(|tex_div| vec4(tex_div.x, 1.0 / tex_div.x, 1.0 / tex_div.y, 0.));
```

但变量名加了 `_` 前缀，**完全没有使用**。材质创建时 `uniforms_vert_globals` 保持全零。

### 修复

在 `update_particle` 的每帧更新循环里，写入 `$Globals (binding=0)` 的两个参数：

```rust
let tex_div = vfx_emitter_definition_data.tex_div.unwrap_or(Vec2::ONE);
let num_cols = tex_div.x.max(1.0);
let num_rows = tex_div.y.max(1.0);

// TEXTURE_INFO（有 layout 表项，用 set_vert_param 查表写入）
material.set_vert_param("TEXTURE_INFO", num_cols, shader_map);

// 匿名 v2float offset=4（无名字，直接按物理偏移写）
let uv_scale: [f32; 2] = [1.0 / num_cols, 1.0 / num_rows];
material.uniforms_vert_globals.write_bytes_at(4, unsafe {
    std::slice::from_raw_parts(uv_scale.as_ptr() as *const u8, 8)
});
```

单帧纹理（`tex_div: None`）时 `num_cols = num_rows = 1`，写入 `TEXTURE_INFO=1.0`，`member1=[1.0, 1.0]`，UV 坐标正确覆盖整张纹理。

### 调试工具速查

| 工具 | 用途 |
|---|---|
| 覆盖写文件日志 + `fmt_uniform`（只打非零 Vec4）| 快速查看材质当前状态 |
| `OnceLock` 一次性 dump ShaderMap | 避免每帧刷文件，只看第一帧 layout |
| `Select-String` + 行号跳转直读 `map.ron` | 比运行时 dump 更快，适合查 blocks 是否为空 |
| `spirv-dis` + `Select-String` | 反汇编确认参数语义，找出哪些成员影响 UV/alpha/可见性 |
| `spirv-reflect.exe` | 验证 binding layout 是否与期望一致 |

### 教训

- **uniform 全零是隐性错误**：管线不崩溃、验证层不报错，只有渲染结果不对。帧数达标 ≠ 渲染正确。
- **匿名 uniform 成员**：SPIR-V layout 表里没有名字的成员，需要用 `spirv-dis` 看 `OpTypeStruct` 和 `OpAccessChain` 确认物理偏移，再用 `write_bytes_at` 直接写。
- **已算好但未使用的变量**：Rust 编译器对 `_` 前缀变量不警告未使用——检查 emitter 创建路径时注意这类"dead 计算"。
