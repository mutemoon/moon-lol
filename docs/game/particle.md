# 粒子与 Shader 系统

## 1. Shader 系统简史

### 阶段 1：静态绑定（AsBindGroup 硬编码）

最初沿用 Bevy 标准模式——每种几何族（quad / mesh / unlit_decal）各写一份
`AsBindGroup` 派生宏，编译期固定 binding 槽位。所有变体共享同一套
`BindGroupLayout`，`uniform` 字段写死偏移。

因为每个变体的布局不同（不同 defs 组合导致 cbuffer 成员数量/排列不同），所以
这种方式只能服务 BASE 变体，切换变体即错位。

### 阶段 2：spirv-reflect 反射 + 规范绑定表

引入 `spirv-reflect.exe` 离线反射，从 SPIR-V 的 `OpName`/`OpDecorate Binding`
读取每个变体的 (binding 号, 槽位类型, cbuffer 成员 offset/size)。

早期 quad 变体的纹理 binding 按 GLSL 声明顺序漂移（同一语义纹理在不同变体
落在不同 binding），因此在提取时做了"规范绑定表"重映射——离线改写 `.spv` 的
`Binding` 装饰字，把所有 quad 变体统一到一套固定槽位。

事后扩展到全部 6 个家族并改为动态布局，规范绑定表方案废弃。

### 阶段 3：动态材质 + 变体专属布局

彻底废弃静态材质，全部迁移到 `ParticleMaterialDynamic`。核心变化：

- **布局从 ShaderMap 读取**：不再写死 binding 号，而是查 `map.ron`
- **并集布局 vs 变体布局双轨**：`vert_layout`/`frag_layout`（家族并集）覆盖
  bind group 全部槽位；`vert_variant_layout`/`frag_variant_layout`（变体自身）
  提供正确的 `set_param` offset——不同 defs 会剔除未使用成员导致 offset 漂移
- **SPIR-V 直通**：通过 `WgpuFeatures::PASSTHROUGH_SHADERS` 绕开 naga，
  `spirv_strip.rs` 运行时剥离 DXC 产出的 Vulkan 1.2 能力声明

### 阶段 4：RDEF 替代 spirv-reflect

dxbc-compiler 新版本不再生成语义名 OpName（cbuffer/纹理名退化为 `cb0`/`t0`），
导致 `spirv-reflect.exe` 的 `--yaml` 输出无法区分 `$Globals` 与普通 cbuffer。
同时 `$Globals` 内部成员名也丢失（退化为 `"m"`），多变量同寄存器时部分成员
甚至整个消失。

解决方案：直接从 DXBC 的 RDEF chunk 提取原始语义名和完整成员表，绕过
SPIR-V 反射。`extract_shaders.rs` 中的 `build_rdef_layout` 函数完全替代了
`spirv-reflect.exe` 的 YAML 解析流水线。

---

## 2. 离线提取管线

```
DXBC (WAD)
    │
    ▼
dxbc-compiler.exe  ──→  SPIR-V (.spv)
    │                       │
    ▼                       ▼
RDEF 解析              spirv_strip
    │                  (capability 剥离)
    ▼                       │
ShaderLayoutDescriptor      ▼
    │                  .spv 写盘
    ▼
unify_family_layouts
    │
    ├─ 构建家族并集（合并同族全部变体 binding）
    ├─ 按 cbuffer 名字母序重排 binding_index → 密集 Vulkan 编号
    ├─ 改写 .spv 的 Binding 装饰字（与并集对齐）
    └─ 裁剪变体布局（仅保留 UniformBuffer，去重后写入布局池）
    │
    ▼
map.ron（ShaderMap 序列化）
```

### 2.1 DXBC → SPIR-V

`process_toc` 从 `ShaderCache.dx11.wad.client` 读取原始 DXBC，
调用 `dxbc-compiler.exe` 编译为 SPIR-V。关键编译参数：

- VS：`--set 3`
- PS：`--set 3 --binding-shift 100`

`binding_index` 公式：`shift + typeBase + regIndex`
- `typeBase`: cbv=0, srv=16, sampler=32, uav=48
- VS shift=0, PS shift=100

### 2.2 RDEF 解析

`parse_rdef` 从 DXBC 的 RDEF chunk 提取：

- **cbuffer 表**：名字、总大小、每个变量的 (name, offset, size, used)
- **资源绑定表**：名字、类型 (cbuffer/texture/sampler)、寄存器号 (bind_point)
- **程序类型**：VS 还是 PS（program_type == 0xFFFF → pixel）

`build_rdef_layout` 用这些信息构建完整的 `ShaderLayoutDescriptor`，
包含语义名 `$Globals`、`mProj` 等，等价于旧版 spirv-reflect 产出。

### 2.3 unify_family_layouts

离线统一 pass，解决两个问题：

1. **跨变体 binding 对齐**：同一家族的 256+ 变体可能有不同的资源绑定表。
   取所有变体 binding 的并集，按 cbuffer 名字（`$Globals` → `PerFrameVertexCB`
   → `PerFramePixelCB` → 纹理 → 采样器）字母序排序后重新分配密集的
   Vulkan binding 编号，让一族内所有变体的 `(binding, 资源)` 映射一致。

2. **VS/PS cbuffer 碰撞**：VS 和 PS 可能都有名为 `$Globals` 的 cbuffer，
   落在同一 `(set, binding)`。并集后拆分为不同的 binding 编号。

同时重写 .spv 文件中的 `OpDecorate Binding` 装饰字，保证文件与 map.ron 一致。

### 2.4 map.ron 结构

```rust
struct ShaderMap {
    /// 每个家族/变体的 lookup：shader_hash → ShaderMapEntry
    entries: HashMap<LeagueShader, HashMap<u64, ShaderMapEntry>>,
    /// 去重后的 cbuffer 布局池（layout_index 指向这里）
    layouts: Vec<ShaderLayoutDescriptor>,
    /// 共享渲染数据（采样器定义、纹理定义）
    shared_render_data: SharedRenderData,
}

struct ShaderMapEntry {
    /// 统一后的 shader handle
    shader_handle: Handle<Shader>,
    variant_key: String,
    /// layouts 池索引
    layout_index: u32,
}
```

- `entries` 按 `(LeagueShader, defs_hash)` 定位变体
- `layouts` 只保留 `UniformBuffer` 类型 binding（纹理/采样器槽位已收敛到并集表）
- 布局按值去重：4397 变体去重为 323 套 cbuffer 布局

---

## 3. 运行时渲染管线

### 3.1 发射器装配（assembly.rs）

```
ConfigVfxEmitterDefinition（!PROP 提取的 VFX 配置）
    │
    ▼
assembly::derive_defs(emitter_def)
    │  按 kind + emitter_def 宏开关推导：
    │  ├─ vert/frag 家族（LeagueShader 枚举）
    │  ├─ defs 字符串数组
    │  ├─ blend_mode
    │  └─ ParticleTextureInputs
    │
    ▼
ParticleMaterialDynamic::create(kind, emitter_def, shader_map)
    │
    ▼
spawn emitter entity（emitter + particles + 材质）
```

`derive_defs` 按 `ParticleRenderKind` 选择 shader 家族：
- Quad → `QuadVs` / `QuadPs`（含 slice 时 → `QuadPsSlice`）
- Mesh → `MeshVs` / `MeshPs`
- Distortion → `DistortionVs` / `DistortionPs`
- SkinnedMesh → `SkinnedMeshParticleVs` / `SkinnedMeshParticlePs`
- UnlitDecal → `UnlitDecalVs` / `UnlitDecalPs`

defs 默认全关（与逆向默认描述符一致），由 `emitter_def` 中的标志位控制开启。

### 3.2 动态材质生命周期

```
Main World                          Render World
─────────                           ────────────
ParticleMaterialDynamic             ExtractedMaterialKeys
  (创建时查 ShaderMap                 (extract 收集 key +
   取布局描述符)                       布局描述符)
    │                                   │
    ▼                                   ▼
  Bevy Extract ─────────────────→   Prepare: 按需拼装
                                      BindGroupLayout
                                        │
                                        ▼
                                    as_bind_group:
                                      ├─ uniform buffer 创建
                                      ├─ 纹理绑定解析
                                      └─ 采样器配对/回退
                                        │
                                        ▼
                                    specialize:
                                      ├─ 选顶点布局 (Mesh/Quad)
                                      ├─ 填 BindGroupLayoutDescriptor
                                      └─ 选 SPIR-V shader
```

**布局缓存键** `PipelineLayoutKey`：
```
(vert_shader: LeagueShader, vert_defs: Vec<String>,
 frag_shader: LeagueShader, frag_defs: Vec<String>)
```
defs 在构造时排序归一化，相同内容命中同一份 `BindGroupLayout`。

### 3.3 set_param：按名写入 uniform

```rust
material.set_param("mProj", projection_matrix);
material.set_param("TEXTURE_INFO", num_cols);
```

实现逻辑：
1. 遍历 `vert_variant_layout` 和 `frag_variant_layout`
2. 在 `UniformBuffer.members` 中按名查找目标成员
3. 按 `member.offset` 写入材质内部的 CPU 字节 blob（`BTreeMap<u32, Vec<u8>>`）
4. Bevy 的 `as_bind_group` change detection 检测到 blob 变更后，
   重建 bind group 并上传新 buffer 内容到 GPU

**为什么需要变体专属布局**：`$Globals` 由松散全局变量拼成，不同 defs
组合会剔除未使用成员并重新紧排 offset。用并集布局的 offset 写到 `set_param`
会落错位置，所以必须用变体自身的反射布局。

### 3.4 SPIR-V 直通与 stripping

```
DXC 编译的 Vulkan 1.2 SPIR-V
    │
    ▼
spirv_strip::strip_spirv(bytes)
    │  ├─ 剥离 OpCapability: PhysicalStorageBufferAddresses,
    │  │   VulkanMemoryModel, RoundingModeRTE, DenormFlushToZero,
    │  │   DemoteToHelperInvocation
    │  ├─ 剥离 OpMemoryModel
    │  ├─ 剥离 OpExecutionMode: RoundingModeRTE, DenormFlushToZero
    │  ├─ DemoteToHelperInvocation → OpKill
    │  └─ 改写 AddressingModel→Logical, MemoryModel→GLSL450
    │
    ▼
wgpu SPIR-V passthrough (PASSTHROUGH_SHADERS)
    │  不经过 naga 翻译，原样发给 GPU 驱动
    ▼
GPU 执行
```

`PASSTHROUGH_SHADERS` 是 wgpu 的特性，允许直接提交 SPIR-V 字节码。
本项目的 shader 不含多重入口点，绕过 naga 避免了 naga 对 SPIR-V
部分指令的解析失败。

### 3.5 纹理与采样器绑定

动态材质在 `as_bind_group` 中按 binding 名解析纹理/采样器：

| 后缀 | 含义 | 处理方式 |
|------|------|---------|
| `__TX` | 纹理（OpTypeImage） | 从 `textures` map 取 Handle，缺失则 fallback |
| `__SMP` | 采样器（OpTypeSampler） | 与对应 `__TX` 配对，相同过滤/寻址参数 |
| `_SharedSampler` | 共享采样器状态 | 从 `SharedSamplerCache` 按名取 wgpu Sampler |
| `_SharedTexture` | 共享纹理 | 从 `SharedRenderData` 按名取 Handle |
| 无后缀 | cbuffer | 不做纹理解析 |

```rust
// 族 × unified 纹理绑定名表
const QUAD_TEXTURE_BINDS: VfxTextureBinds = VfxTextureBinds {
    diffuse: "DIFFUSE_MAP__TX",
    falloff: "FALLOFF__TX",
    palette: "PALETTE_TEXTURE__TX",
    color_remap: "PIXEL_COLOR_REMAP_RAMP_SharedTexture",
};
```

`ParticleTextureInputs` 提供按语义命名的贴图输入（`texture`、
`particle_color_texture`、`texture_mult` 等），`create` 按 kind 查表
映射到各族 unified 布局里的实际 binding 名。

### 3.6 粒子生命周期

```
英雄实体
  │  On<CommandSkinParticleSpawn>
  ▼
skin/particle.rs: 解析 trigger_key → vfx_hash
  │
  ▼
CommandParticleSpawn { entity, vfx_handle }
  │
  ▼
emitters/state.rs: 加载 ConfigVfxEmitterDefinition
  │
  ▼
emitters/state.rs: derive_defs + create 材质
  │
  ▼
spawn emitter entity
  ├── PluginLifetime（发射器生命周期）
  ├── ParticleRenderKind
  ├── ParticleMaterialDynamic（材质句柄）
  ├── Mesh3d + MeshMaterial3d
  └── particle child entities
        ├── PluginLifetime（粒子生命周期）
        └── Transform（以 emitter 为父节点）
```

- **emitter 生命结束**：粒子可能仍需存活（如淡出动画）→ emitter 用
  "无子实体时才销毁" 模式
- **英雄生命结束**：所有 emitter 和粒子随英雄销毁
- **`is_local_orientation`**：emitter 手动更新自己的 GlobalTransform，
  particle 用父节点的 GlobalTransform 计算 world matrix 传给 shader

---

## 4. 关键设计决策

| 决策 | 原因 |
|------|------|
| SPIR-V 直通而非 naga | 386/4397 个变体 naga 不可解析；DXC 产出标准合规 |
| 并集布局 + 变体布局双轨 | 并集保证 bind group 槽位完整，变体保证 set_param offset 正确 |
| RDEF 替代 spirv-reflect | 新 dxbc-compiler 不产 OpName，语义名只能从 DXBC源头获取 |
| cbuffer 布局去重 | 4397 变体仅 323 套不同 cbuffer 布局，map.ron 大幅缩小 |
| 按名查 offset | 不依赖 binding_index 做 offset 定位，因同一 cbuffer 在不同变体 binding 号不同 |

---

## 5. 参考索引

- `examples/extract_shaders.rs` — 离线提取与 map.ron 生成
- `crates/lol_particle/src/particle/dynamic.rs` — 动态材质核心实现
- `crates/lol_particle/src/particle/assembly.rs` — 发射器参数推导
- `crates/lol_particle/src/emitters/` — 粒子发射器创建/更新/回收
- `crates/lol_render/src/loaders/spirv_strip.rs` — SPIR-V 预处理
- `crates/lol_base_render/src/shader_layout.rs` — 布局类型定义
- `tests/shader_layout_validation.rs` — 端到端 cbuffer 布局验证
- `docs/reverse/shader.md` + 子文档 — 原始游戏 shader 逆向分析
