# 运行时结构体定义（已写入 IDB）

> 上级：[shader.md](../shader.md)

因为 MCP 的 `declare_type`/`search_structs` 工具与 IDA 8.3 的新类型系统(til)不兼容（报 `PT_EMPTY` / `get_ordinal_qty` 错误），所以改用 `py_eval` + `idc.parse_decls` 直接在本地类型库中定义了以下结构体，并用 `idc.SetType` 把 `sub_1412B7330` / `sub_1412B7EA0` 的 `a1` 参数类型设为 `ShaderEffect *`，使反编译以命名字段渲染。

## 2.1 ShaderEffect（512 字节）

```c
struct ShaderEffect
{
  void *field0;
  void *field8;
  struct ParticleEmitterDescriptor *descriptor; // +16  运行时描述符指针
  char gap24;
  unsigned __int8 applyTeamColor;      // +25  队伍色开关
  char gap26[2];
  int passIndex0;                      // +28  (==-1 检查)
  char gap32[4];
  int passIndex1;                      // +36
  int passIndex2;                      // +40
  char gap44[4];
  void *texSlotDiffuse;                // +48
  void *texSlotColorOverLife;          // +56
  void *texSlotDistortion;             // +64
  void *texSlotFalloff;                // +72
  void *texSlotSecondary;              // +80
  void *texSlotReflection;             // +88
  void *texSlotPalette;                // +96
  void *texSlotErosion;                // +104
  char gap112[16];
  void *shader;                        // +128 已加载的 shader 对象
  char gapTail[376];
};                                     // total 512 bytes
```

## 2.2 ParticleEmitterDescriptor（1136 字节）

```c
struct ParticleEmitterDescriptor
{
  void *field0;
  void *field8;
  char gap16;
  unsigned __int8 backFaceOn;          // +17  p-backfaceon
  unsigned __int8 texturePixelate;     // +18
  char gap19;
  unsigned __int8 renderMode;          // +20
  char gap21[2];
  unsigned __int8 importance;          // +23  (Medium→3, 默认 2)
  char gap24[24];
  void *softParticleParams;            // +48  默认 NULL(软粒子关)
  void *alphaErosionObj;               // +56
  void *paletteObj;                    // +64
  void *reflectionObj;                 // +72
  void *distortionParams;              // +80
  void *field88;
  void *primitive;                     // +96  *(+96)+8 = renderType 字节
  void *overrideShader;                // +104
  unsigned __int16 pass;               // +112
  unsigned __int8 flagsDisableZ;       // +114
  unsigned __int8 forceAnimMeshZWrite; // +115 默认 1
  unsigned __int8 teamColorCorrection; // +116
  unsigned __int8 blendMode;           // +117
  unsigned __int8 stencilRef;          // +118
  unsigned __int8 alphaRef;            // +119 默认 5 (ALPHA_TEST=val/255)
  unsigned int stencilWriteMask;       // +120
  float depthBiasFactor0;              // +124 默认 0
  float depthBiasFactor1;              // +128 默认 0
  float alphaSliceRange;               // +132 默认 0 (SLICE_RANGE)
  float field136;                      // +136 1.0
  float texDivX;                       // +140 1.0
  float texDivY;                       // +144 1.0
  char gap148[4];
  char primaryTexture[16];             // +152 p-texture(空)
  char colorTexture[16];               // +168 DefaultColorOverlifetime.png
  char falloffTexture[16];             // +184 DefaultFalloff.png
  char gap200[24];
  float colorModulate[4];              // +224 kColorFactor(白色{1,1,1,1})
  char gap240[8];
  float particleLifetime;              // +248 p-life = 3.0
  char gap252[12];
  float emitterLifetime;               // +264 e-life = FLT_MAX
  float emitterLifetime2;              // +268 FLT_MAX
  char gap272[448];
  unsigned int miscFlags;              // +720 默认 0x202000 (0x200000=e-local-orient)
  char gap724[104];
  float depthPushPull;                 // +828 PARTICLE_DEPTH_PUSH_PULL = 0
  char gapTail[304];
};                                     // total 1136 bytes
```

## 2.3 在 IDA 中复现定义（供后续重建 IDB 用）

```python
import idc
decls = r'''
struct ParticleEmitterDescriptor { /* 见 2.2 完整字段 */ };
struct ShaderEffect { /* 见 2.1 完整字段 */ };
struct ShaderPassVariantData { /* 见 2.4 */ };
struct ParticleRenderContext { /* 见 2.5 */ };
'''
idc.parse_decls(decls, 0)                 # errors 应为 0
# a1 = ShaderEffect* → descriptor 字段经 a1->descriptor 自动传播命名
idc.SetType(0x1412B7330, "char __fastcall f(ShaderEffect *a1, __int64 a2, __int64 a3, unsigned __int8 a4)")
idc.SetType(0x1412B7EA0, "__int64 __fastcall f(ShaderEffect *a1, __int64 a2, __int64 a3, unsigned __int8 a4)")
idc.SetType(0x1412E39D0, "__int64 __fastcall f(ShaderEffect *a1)")
# BuildShaderPathAndDefines：返回值未被调用方使用 → 改 void，令 result 不再退化为 __int64
idc.SetType(0x1412DD450, "void __fastcall f(ShaderEffect *a1, ParticleRenderContext *a2, unsigned __int8 a3, ShaderPassVariantData *a4)")
```

> 验证：`tinfo_t().get_named_type(None, name).get_size()` → ParticleEmitterDescriptor=1136、ShaderEffect=512、ShaderPassVariantData=136、ParticleRenderContext=952。
> 新类型系统(til)类型不能用旧 `get_struc_id`（返回 0），须用 `tinfo_t.get_named_type` 校验。

## 2.4 ShaderPassVariantData（136 字节，`BuildShaderPathAndDefines` 的 `a4`）

每个 pass 构建出的「shader 变体记录」：VS/PS 路径句柄 + defines 动态数组 + 脏标记。

```c
struct ShaderPassVariantData
{
  char gap0[48];
  unsigned __int64 vsPathHandle;   // +48 = Shader_HashConstantName(VS 路径)
  unsigned __int64 psPathHandle;   // +56 = Shader_HashConstantName(PS 路径)
  void *defines;                   // +64 24B/项：16B name + 8B value
  unsigned int defineCount;        // +72
  unsigned int defineCapacity;     // +76
  unsigned __int8 definesDirty;    // +80
  char gapTail[55];
};
```

## 2.5 ParticleRenderContext（952 字节，`BuildShaderPathAndDefines` 的 `a2`）

渲染上下文（在两个 setup 函数里只透传、不解引用），仅建模已确认的 5 个偏移。三个表的条目布局由步长算术严格推出（100% 可证），单独建为 `ShaderPassStateEntry`(40B) / `ShaderResourceEntry`(16B)：

```c
struct ShaderPassStateEntry   // 条目 40B
{
  char gap0[32];
  void *state;   // +32 -> pass 渲染状态对象（obj+16 -> desc；desc+64 int = 类别 1/2/3）
};
struct ShaderResourceEntry    // 条目 16B
{
  char gap0[8];
  void *object;  // +8 -> shader/材质反射对象（参数表 @ +160 count @ +176；+125 字节标志）
};

struct ParticleRenderContext
{
  char gap0[208];
  struct ShaderPassStateEntry *passStateTable; // +208 idx = ShaderEffect.passIndex0
  char gap216[24];
  struct ShaderResourceEntry *resourceTableA;  // +240 idx = passIndex1
  char gap248[8];
  struct ShaderResourceEntry *resourceTableB;  // +256 idx = passIndex2
  char gap264[332];
  unsigned __int8 detailLevel;                 // +596 画质等级(>=2 才允许 SOFT_PARTICLES)
  char gap597[350];
  unsigned __int8 softParticleAllowed;         // +947 软粒子开关
  char gapTail[4];
};
```

> **准确性说明（qword 标尺寻址）**：因为编译器把 40B/16B 步长优化成「按 qword 计数」的寻址，所以 Hex-Rays 不会把它折叠成 `table[idx].field`，而是保留等价形式：`*((_QWORD*)passStateTable + 5*idx + 4)` ≡ `passStateTable[idx].state`（`40*idx+32`）；`*((_QWORD*)resourceTableA + 2*idx + 1)` ≡ `resourceTableA[idx].object`（`16*idx+8`）。两者数值完全相等，条目结构体仅作类型层面的准确布局记录。
> **未完全考证**：`state`/`object` 指向的引擎对象（渲染状态描述符 / shader 反射对象）只观测到零星字段，身份未完全确认，故未建完整子结构，只在注释里记已观测偏移。
> **误报符号**：`v17` 上的 `AK::WriteBytesMem::Bytes(v17)` 是误报 —— 其实体仅 5 字节 `return *(this+16)`，是多类共用的通用取指针器，与 Wwise 无关。

## 2.6 Hex-Rays 类型推断经验

因为 `descriptor` 局部变量自始至终只承载 `a1->descriptor` 这一种值（角色单一），所以类型能自动传播；因为 `result` 是映射到 RAX 的返回值变量、被复用来存哈希值/指针/整数等多种值，所以它被统一退化为 `__int64`，只能把 `a1->descriptor` 强制转型。修复手段两条：**① 若返回值未被调用方使用，把返回类型改 `void`**（本例 `BuildShaderPathAndDefines` 改 void 后 `result + 720` 直接渲染为 `a1->descriptor->miscFlags`）；**② 给不透明指针参数（如 a2/a4）定义结构体并 `SetType`**，裸偏移 `*(a4+72)` 即变 `a4->defineCount`。
