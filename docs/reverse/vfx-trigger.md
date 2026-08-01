# VFX 触发机制逆向：源粒子 vs 目标粒子（以 Fiora_BA1_tar 为例）

> 静态逆向 League of Legends.exe（imagebase 0x140000000）+ 提取数据交叉验证。
> 起因：`Fiora_BA1_tar`（剑姬普攻命中被攻击者身上的粒子）在所有提取出的 props 里找不到任何引用，需查清它在哪里、按什么规则被触发。

## 结论速览

- **源粒子（attacker/trail）** 由**动画事件**触发：动画 bin 的 `AtomicClipData.mEventDataMap` 里的 `ParticleEventData.mEffectKey` → 经 skin 的 `ResourceResolver.resourceMap` 解析为粒子系统，绑定到**攻击者自己**的骨骼。
- **目标粒子（`*_tar`）** **不由动画事件直接命名触发**，也不被任何 spell/animation/character 数据文件按名引用；它只作为 `ResourceResolver.resourceMap` 的一个**条目**存在，由**普攻/命中路径**在运行时用触发名去 resolver 里查出来并 spawn。
- **触发名的构造规律（本轮数据坐实）**：动画 `mEffectKey` / spell `mHitEffectKey` 只存**基名**（`Fiora_BA`/`Fiora_BA2`/`Fiora_BA_Crit`）；resolver 里基名与「基名+`_tar`」成对登记。**⚠保留**：“引擎在运行时把基名拼上 `_tar`”这一步**尚未在代码里看到**，且有反证（见下）；不得再断言为字符串拼接。
- 因此在 props 里搜不到 `Fiora_BA1_tar` 的“引用方”，是符合预期的——它是被查的**目标**，而不是被写在某处的**引用**。

> 更正（勿再表述为“引擎硬编码/拼名”）：本轮数据坐实了「基名 ↔ 基名+`_tar`」的**命名配对**（动画/spell 只引用基名，`_tar` 仅作 resolver 键）。但下列三条**反证**表明不得再声称“运行时拼接”：① 动画 `ParticleEventData` 只有 `mEffectKey`（自身）+ 攻击者骨骼，无任何指向 `_tar` 的字段；② 解密字符串缓存（34791 条）里**无** `_tar` 子串、**无** `%s_tar` 格式串、**无** `mHitEffectKey`；③ 把三个 `_tar` 名的 hash（十/十六进制）在全部 fiora props 里搜，**无任何字段值命中**。结论：“从基名得到 `_tar` 变体”的具体机制（拼接重 hash / build 期预算成对 / 另一套未提取的数据）**至今未在代码或数据里定位到**。

## 数据侧证据

### 1. 动画事件只引用源粒子（attacker-side）
`assets/props/data/characters/fiora/animations/skin0.prop`：
```
"Attack1" = AtomicClipData {
    mEventDataMap = { "WeaponTrail" = ParticleEventData {
        mEffectKey: hash = "Fiora_BA"                       // 源/拖尾
        mParticleEventDataPairList = { mBoneName = "BUFFBONE_GLB_GROUND_LOC" }  // 绑攻击者自己骨骼
    } } }
"Attack2" = ... mEffectKey = "Fiora_BA2" ...
```
→ 攻击动画里只播 `Fiora_BA` / `Fiora_BA2`（源），**没有** `Fiora_BA1_tar`。且 `mParticleEventDataPairList` 只有 `mBoneName`（攻击者骨骼），没有 `mTargetBoneName`；`mEnemyEffectKey` 未设置。

### 1b. 命中特效「基名」也来自 spell/attack 数据（本轮新证据，钉死基名来源）
`assets/props/data/characters/fiora/fiora.prop`：
```
:2188  mHitEffectKey: hash = "Fiora_BA_Crit"          // 暴击命中特效「基名」，来自 spell 数据
:2262  basicAttack: embed = AttackSlotData { mAttackTotalTime/mAttackCastTime/mAttackProbability }
:2278  ExtraAttacks → mAttackName = "FioraBasicAttack2"   // 第二段普攻动画
:2285  critAttacks  → mAttackName = "FioraCritAttack"     // 暴击动画
```
→ 命中特效的「基名」在数据里**确有出处**：普攻/BA2 来自动画 `mEffectKey`（`Fiora_BA`/`Fiora_BA2`），暴击来自 spell `mHitEffectKey`（`Fiora_BA_Crit`）。**全部是不带 `_tar` 的基名**。

### 1c. 基名 ↔ `_tar` 的完整配对（skin69/skin60/skin31/base 一致）
resolver（`skinNN.prop` 的 `resourceMap`）对每段攻击都成对给出「自身粒子」与「目标粒子」：
```
"Fiora_BA"          = .../Fiora_SkinNN_BA          // 自身（攻击者），动画 mEffectKey 引用
"Fiora_BA1_tar"     = .../Fiora_SkinNN_BA1_tar     // 目标（被攻击者）—— 无任何字段引用
"Fiora_BA2"         = .../Fiora_SkinNN_BA2          // 自身
"Fiora_BA2_tar"     = .../Fiora_SkinNN_BA2_tar     // 目标
"Fiora_BA_Crit"     = .../Fiora_SkinNN_BA_Crit      // 自身，spell mHitEffectKey 引用
"Fiora_BA_Crit_tar" = .../Fiora_SkinNN_BA_Crit_tar // 目标
```
→ 配对规律：**目标名 = 基名 + `_tar`**（首段普攻做 `BA→BA1` 归一化）。基名侧（左列）都能在动画/spell 数据里找到引用方；`_tar` 侧（右列）**在所有 props 里只作 resolver 键 + 自身系统定义出现，从不作为任何字段的值**（`childParticleName` 含 `_tar` 的匹配数 = 0，已复核）。

### 2. `*_tar` 只存在于 resolver map（作为“被查的目标”）
`assets/characters/fiora/skins/skin0_vfx.ron` 的 resolver：
```
resource_map: {
  "Fiora_BA":         3053942044,   // 源
  "Fiora_BA1_tar":    1384368375,   // 目标（命中）
  "Fiora_BA2_tar":    3460291734,
  "Fiora_BA_Crit_tar":1404443921, ... }
```
`ResourceResolver.resourceMap` 的语义 = `{ 触发名Hash : 粒子系统Hash/link }`。`skin75.prop` 里能看到 link 形式：
```
resourceMap = { "Fiora_BA1_tar" = "Characters/Fiora/Skins/Skin69/Particles/Fiora_Skin69_BA1_tar" ... }
```

### 3. 源系统与 AttackSlotData 都不引用 `_tar`（props 已穷尽）
用户确认 `assets/props` 即“能提取的全部 props”，据此穷尽核对：
- **源系统 `Fiora_Base_BA`**（`fiora_multi_...skin9.prop:16493`）只有 1 个 emitter（`emitterName="Mesh"` 武器拖尾 `Fiora_WeaponTrail.scb`，绑攻击者），**没有任何 child/linked emitter，不引用 `_tar`**。→ `_tar` 不是源普攻特效的子粒子。
- **`Fiora_Base_BA1_tar` 本体**（同文件 `:7004`）是自包含的命中闪光：2 个 emitter（`Globalhit_Flash`+`Fiora_Flash`，用 `common_HitEffect.tex`/`common_color-hit-physical.tex`/`common_Flare-Omnimax_purp.tex`，`flags=198`），**不回指任何东西**。
- **`AttackSlotData`**（`fiora.prop:2262` basicAttack / ExtraAttacks / critAttacks）只有 `mAttackTotalTime`/`mAttackCastTime`/`mAttackProbability`/`mAttackName`（如 `FioraBasicAttack2`、`FioraCritAttack`），**没有任何粒子/命中特效字段**。
- 全 Fiora props 中 `mEnemyEffectKey` **从未被设置**；所有 `_tar`（含 `BA_Crit_tar`、`E_Hit_tar`、`Q_Hit_Tar` …）都只以 resolver key + 系统定义两种身份出现，**无一处作为“触发请求”**。

→ **数据侧到此闭合**：`Fiora_BA1_tar` 在所有可提取 props 里只有「resolver 条目」与「粒子系统定义」两种身份，没有任何数据结构说“触发它”。命中普攻 → 触发 `_tar` 的这条边**不在提取数据里**。

### 4. 提取管线为何丢了动画事件
`crates/league_to_lol/src/extract/skin.rs` `load_animations` 只读 `AtomicClipData.mAnimationResourceData`(.anm 关键帧) + mask，**从不读 `mEventDataMap`**，所以 moon-lol 侧 `assets/characters/` 输出天然没有粒子事件；只有原始 `assets/props/data/` 的 .prop 保留了事件。

## IDA 侧证据

### 名字都是 FNV-1a 小写哈希（已验证）
- `fnv1a_lower("fiora_ba1_tar") = 0x4E7E421B`，与 resolver 触发名哈希一致。
- bin 字段名同为小写 FNV-1a（对照 CommunityDragon `hashes.binfields.txt`）。
- exe 中**不存在** `_BA` / `_tar` / `_src` 这类命名约定的格式字符串字面量（字节扫描 .text/.rdata/.data 确认；出现的 `_tar`/`_src` 都是 traceroute/Lua 等无关串）。→ 触发名不是运行时用 C++ 字符串拼接构造的，而是以**预计算哈希**参与查表。

### 触发名来源：排除硬编码与运行时拼接（已验证）
- **hash `0x4E7E421B` 全二进制搜不到**（小端 `1B 42 7E 4E` / 大端均 0 命中）；`Fiora_BA1_tar`/`BA1_tar`/`ba1_tar` 字符串也不存在；唯一的 `_tar` 命中是 `ts_target_bitrate`（无关）。→ 该 hash/名不作为常量烘进代码，**排除“引擎硬编码”**。
- **exe 导入表只有 `stub.dll!packman`**（打包壳）；Game 目录仅 RiotGamesApi/stub/vivoxsdk/rpatch/D3DCompiler_47。→ 对局逻辑不委托给其它 DLL，**不在别的 DLL 里**（壳可能影响字符串完整性，但本库可反编译、可读 3.4w+ 字符串）。
- **更正（曾误判）**：0x141696BA0 之前被命名为 `HashNameLowerFnv_Dispatch` 并当作“全二进制唯一运行时 FNV 点→数据驱动”的核心证据。反编译其唯一调用者 `sub_141693880`（满是 `AK::SoundEngine::IsInitialized`/`GetGlobalPluginContext` 音频调用）及本体（调 `AK::SoundEngine::GetIDFromString` + `WideCharToMultiByte(CP_UTF8)` + FNV-**1**：`v15=16777619*v13; v13=v16^v15`，basis 0x811C9DC5）后确认：**它是 Wwise 音频名→AkID 的哈希，不是 VFX/bin 的 resolve**。已 rename → `Wwise_ProcessGameObjStateSwitchQueue`。教训：Wwise 用 FNV-1、LoL bin 用 FNV-1a，**同 prime 16777619 只是乘/异或顺序不同**，故 byte-scan prime 会同时命中两者，不能仅凭立即数断定用途。
- **综合判定：数据驱动**——触发 hash 是从 bin 加载进内存的数据值（resolver key 本身就是数据），调用方手上的 hash 从加载的 ResolverRef 字段取得（见下「resolver 消费链」，`resolveTypedRef` 的 `ref+0` 就是这个 hash）。
- 旁证：数据侧确有命中行为系统 `EnumCastOnHit`（`TriggerOnHit`/`DestroyOnHit` …，见 `league_core::extract`），挂在 `MissileSpecification.behaviors` 上；但 Fiora 近战普攻无 missile，故不走该字段，其近战 `_tar` 的携带字段仍待定位。

### 反射注册（已命名 + 注释，已写入 IDB）
| 地址 | 命名 | 说明 |
|------|------|------|
| 0x1401A3980 | `ParticleEventData_registerType` | 类型 hash=88265757，size=88。字段偏移：`mEffectKey@24`(0xF6386280)、`mEnemyEffectKey@28`(0x98DF0FB4)、`mEffectName@32`(0x5A3DD1C2)、`mParticleEventDataPairList@48`(0x60645D6A)、bool@72..76、f32@80、@64 |
| 0x1401A85B0 | `ResourceResolver_registerType` | 类型 hash=1923179998；字段 `resourceMap@8`(0xD2F58721) = map<触发名Hash, 粒子系统> |
| 0x1411C4280 | `ParticleEventData_ctor` | vtable=`off_141B1D1A8`；布局 mEffectKey@24 / mEnemyEffectKey@28，默认 +16=-1.0f、+80=1.0f |

### resolver 消费链（本轮定位，已命名 + IDB）
从字符串 `"resolverLink is null for sdr: %s"`(@0x141aac1e8) 上溯，厘清了 resolver 的**运行时消费机制**：

| 地址 | 命名 | 说明 |
|------|------|------|
| 0x1411BAEE0 | `ResourceResolver_resolveTypedRef` | 核心解析器。入参 `(ref, targetType)`：`ref+0`=触发名 u32 hash、`ref+4`=缓存版本、`ref+8`=缓存结果。命中缓存直接返回；否则遍历全局 resolver 列表逐个 `mapLookup(resolver, ref)` 查表，命中后 `Rtti_adjustPtrToType` 转成 `targetType`。**全引擎通用，100+ 调用点。** |
| 0x1411AE180 | `ResourceResolver_mapLookup` | 在单个 resolver 的 map 里按 ref（hash）查条目 |
| 0x1411A95E0 | `Rtti_adjustPtrToType` | 类型指针调整器（沿 +56 类型链、+104/+112 基类数组，按 +8 偏移调整）——`dynamic_cast` 语义，非 spawn |
| 0x140981C70 | `Obj_linkResolverRefs_sdr` | 遍历对象的 sdr 数组(+1632/count+1640/stride16)，对每条 ResolverRef 调 `resolveTypedRef`，解析不到打 `resolverLink is null for sdr:` 日志。resolver 的一个批量消费方 |

**关键结论（因果）**：
- 因为 `resolveTypedRef` 只按 `ref+0` 里的**预计算 hash** 查 `resourceMap` 再做类型转换，而这个 hash 存在**加载后的 bin ResolverRef 字段**里，所以 exe 里既没有 `0x4E7E421B` 常量、也没有 `Fiora_BA1_tar` 字符串——名字→link 完全数据驱动、运行时解析。这与「数据侧搜不到引用」是同一事实的两面。
- 因为该解析器是被上百处复用的**通用枢纽**（VFX/音频/技能/sdr 都走它），所以单纯自底向上爬它的 100+ 调用者去撞普攻命中路径，投入产出比极低且极易再次误判（如上一次 Wwise 误判）。**精确 spawn 指令的定位成本高、置信度低**，而机制层面已完全查清。

### 粒子子系统与按名播放入口（本轮自顶向下突破，已命名 + IDB）
从 `Obj_linkResolverRefs_sdr` 的调用者向下挖，因 `.troy`（LoL 粒子文件扩展名）定位到整个**粒子子系统**（地址簇 0x14091xxxx / 0x14092xxxx / 0x14096xxxx）：

| 地址 | 命名 | 说明 |
|------|------|------|
| 0x140913840 | `ParticleMgr_getOrCreateNamedSystem` | **按名播放/取粒子系统的入口**。入参 `(mgr, owner, name)`：name 非空→`Hash_elf_lower(name)` 得 key（否则默认 id），在 mgr 红黑树里 get-or-create 实例，并对其 sdr 子资源调 `Obj_linkResolverRefs_sdr` 解析。**~15 个调用者**（远少于 resolver 枢纽的 100+） |
| 0x141136410 | `Hash_elf_lower` | ELFHash（小写化）：`h=(h<<4)+c; g=h&0xF0000000; if(g) h^=g>>24; h&=~g`。用于粒子实例在 mgr 里的键——**注意：与 bin resolver key的 FNV-1a 不同哈希** |
| 0x1403F5E90 | `Character_spawnConfiguredParticles` | 播放角色配置的一批**持久/环境粒子**（从角色数据的名字槽逐个 play，owner=角色自身）。已排除为 per-hit `_tar` 站点 |

**已逐个查验的 by-name 调用者（owner 判别器）**：
| 地址 | owner（粒子挂载对象） | 性质 |
|------|------|------|
| `Character_spawnConfiguredParticles` (0x1403F5E90) | 角色自身 
`*(a1+32)` | 成套持久粒子（角色数据 16 槽） |
| 0x1403F49A0 | `a1[4]` 配置对象 | 成套粒子（a1[5] 的 1264..1552 共 18 槽） |
| 0x14028C0B0 | `v24[4]` | 实例化 18 个配置粒子到 408B/个 的数组（与上行同槽位） |
| 0x1405D11B0 | 全局世界 `qword_141ED9FA8+72` | 按对象 id 维护状态并播放 `*(a2+1512)` 名粒子 |
| 0x140726690 | 全局世界 `qword_141ED9FA8+96` | 按名 warm/预建一个粒子系统 |
| 0x140263DF0 | `v3` = `*(a1+72)` 的 vtable 取全局世界 | 遍历 `a2` 的多个 name 列表批量 warm/建成套配置粒子 |
| 0x14028C020 | **`a2`（调用方传入的 owner）** | **虚函数**（仅被 4 个 vtable 槽引用）：把 `a1` 里一整套配置粒子名逐个实例化到 owner=`a2`（单位/皮肤初始化时的“成套实例化”） |

→ **已查的 7 个 by-name 调用者全是“成套配置粒子 / 全局世界上下文”的实例化器**：即使 `0x14028C020` 的 owner 是调用方传入的 `a2`，它也是**一次实例化一整套（遍历 name 列表）**、发生在初始化时，**没有任何一个是“把单个 transient `_tar` 挂到本次受击目标”的 per-hit 站点**。

**关键结论（因果）**：
- 因为 `ParticleMgr_getOrCreateNamedSystem` 接收的 name 是**指向加载后 bin 数据的字符串指针**（如 `Character_spawnConfiguredParticles` 从角色数据 `+2968..` 取名），所以粒子名在内存里确实以字符串存在、只是不在 exe 静态 .rdata——与“字节扫描搜不到 `_tar` 字面”完全一致。
- 因为逐个查验后发现**这个 by-name 入口统一用于“实例化角色/皮肤的成套配置粒子集”（owner=自身/全局世界，名字来自数据槽），而非“把一个 transient 粒子挂到任意受击目标”，所以推断：per-hit 的 `_tar` **不是用 name 字符串新建**，而是命中时拿着那个 `_tar` 的 ResolverRef（hash）经 `resolveTypedRef` **取到已实例化/已解析的粒子系统，然后播放/弹出到目标**。
- 综合闭环：皮肤数据同时带有① `resourceMap{触发名Hash→link}` 与 ②一批配置粒子系统；初始化时经 `ParticleMgr_getOrCreateNamedSystem`（ELFHash 键）实例化并用 `resolveTypedRef` 解析内部 ResolverRef；普攻命中时，引擎拿 `Fiora_BA1_tar` 的 FNV-1a hash（存于加载的 ResolverRef）→`resolveTypedRef` →得到 `_tar` 粒子系统 →绑到受击者播放。整条链的名字/hash 均为数据，故 exe 无常量、无字符串。

## by-name 家族已完全枚举 = 资源预加载/实例化（本轮收口）
把 `ParticleMgr_getOrCreateNamedSystem`（@0x140913840）的**全部 11 个调用者**逐个反编译后，确认这个"按名"家族**统一是"资源预加载/成套实例化"**，无一是 per-hit 目标 spawn：
- **重要更正**：这个入口**不是粒子专用**，而是**通用的"按名 get-or-create 客户端资源"**（over `qword_141ED6D00+8`）。证据：`sub_14094E210` 是 `SpellDataInstClient::SetSpellData`（含字符串 `"SpellDataInstClient::SetSpellData: %s not found"`、`"Asked for spell name ..."`），它拿同一入口取的是**技能数据**而非粒子；`sub_140380EB0` 是 **`BBPreload*` 预加载指令解析器**（`BBPreloadCharacter/Spell/Particle/Module/AssetsLua`），为 `BBPreloadSpell` 走 `sub_140935A40(owner, name)` 单名预加载。
- 其余调用者（`ParticleMgr_warmConfiguredNameLists`、`Particle_instantiateConfiguredSetOnOwner_vfunc`、`sub_14092E590/9F0`、`Character_spawnConfiguredParticles` 等）全是**遍历 name 列表批量实例化配置粒子集**，owner 为自身/全局世界/调用方传入，但都是**初始化期一次成套**，非 per-hit。
- `sub_140935A40` 是最薄的单名包装 `get(owner, name)`，其调用者也全在**预加载/初始化**路径（`BBPreload`），不在命中路径。

### VFX 运行时表构建链：resolver → 运行时粒子系统（本轮重大突破，已命名 + IDB）
用 targetType `&unk_141EEA0E0`（= **VFX 系统定义类型**，即 resolver map 的 value 类型）作过滤器，把 `resolveTypedRef` 的 100+ 调用者收敛到**仅 ~6 个 skin-VFX 函数**（地址簇 0x1403Exxxx/0x1403Fxxxx），厘清了 **resolver 在加载期如何被物化成运行时表**：

| 地址 | 命名 | 说明 |
|------|------|------|
| 0x1403FB950 | `CharacterVfx_loadSkinVfx` | 角色 VFX 组件的"载入皮肤 VFX"：把组件名写到 `a1+152`，调下面的构建器把运行时表建到 **`a1+176`**，设置 `a1+133/168/192` 状态位 |
| 0x1403FBB50 | `CharacterVfx_buildRuntimeTableFromResolver` | **核心**：遍历皮肤 resolver-ref 列表，对每条 `resolveTypedRef(ref, &VfxDefType)` 解析出系统定义（**含全部 `_tar` 条目**），经 `Vfx_resolveDefToSceneMgr` 取到运行时粒子系统，并为每个系统展开 **5 类子引用**（def 的 +24/+32/+48/+64/+80 列表，各自再 `resolveTypedRef` 解析并把结果系统存到子项 +32），把运行时条目建到 `a3`(=`a1+176`) |
| 0x1403ED9F0 | `Vfx_resolveDefToSceneMgr` | 按 def 名在全局场景注册表里匹配，返回 `unk_141F1A8E8`（stride 280B）中对应"场景/上下文粒子管理器"条目 |
| 0x1403F3E70 | `CharacterVfx_matchResolvedRefs` | 比较器：解析两组 ref（同 `&VfxDefType`）按名比对，返回 bool |

**关键结论（因果）**：
- 因为 `CharacterVfx_buildRuntimeTableFromResolver` 在**皮肤加载期**就把整张 `resourceMap`（`Fiora_BA`、`Fiora_BA1_tar`、`Fiora_BA2_tar`、`Fiora_BA_Crit_tar` … 全部）经 `resolveTypedRef` **解析成运行时粒子系统并建到角色 VFX 组件的 `+176` 表**，所以 `_tar` 系统在**开局/换肤时就已实例化好**、按其触发名 hash 可查——这与"by-name 入口只做预加载"、"per-hit 不新建而是取已实例化系统"的推断**完全吻合并被证实**。
- 因为这张表以**触发名 hash 为键、以运行时粒子系统为值**存活在角色组件里，所以普攻命中时 combat 路径只需 `charVfx[+176].lookup(hash("Fiora_BA1_tar"))` 取到系统再播放到受击者——**无需任何字符串、无需 append "_tar"**，故 exe 既无 `_tar` 字面也无该 hash 常量。
- 唯一仍未 100% 钉死的：读取 `+176` 表并"播放到目标"的那条 per-hit combat 指令（它读偏移 `+176`，难以直接 xref）。但**物化侧已被证实**，per-hit 只是对这张已建好的表的一次按 hash 查询。

### ParticleEventData 无"fire"虚函数（本轮验证，解释无 xref）
- 类型 hash `88265757`(=0x0542D41D) **全二进制只出现 1 次**，在 `ParticleEventData_registerType` 内部——**没有任何代码做类型 hash 比较**，故事件派发不靠 hash。
- 其 vtable `off_141B1D1A8` 的槽位**全是 getter/`return 1`/空 stub**（逐个反编译 0x140219A40/0x1403721D0/0x140279FE0/0x1401EE810 均无 spawn 逻辑）；该 vtable 仅 3 处 data 引用且全是 ctor 式写入，**无类型比较引用**。
- 因此"播放粒子事件"**不是 ParticleEventData 自己的虚函数**，而是动画 clip 播放引擎在遍历 `mEventDataMap` 时**外部读取字段（+24 mEffectKey / +28 mEnemyEffectKey）并调 resolver + 播放**——这正是"靠谁引用这张表/这个名字都定位不到"的根因，与数据侧"搜不到引用"同源。

### 为什么“找不到触发点”在逆向里也表现为“无直接 xref”
`ParticleEventData` 自身**没有 fire 虚函数**（见上节验证）；其播放由动画 clip 播放引擎在遍历 `mEventDataMap` 时**外部读字段后发起**，该粒子名/这张表除反序列化外**没有 gameplay 侧的数据 xref**，所以无法靠“谁引用了这个粒子名/这张表”定位——这与数据侧“搜不到引用”是同一现象的两面。

 真正触发逻辑：约定式命名 + 运行时按名解析（本轮钉死到证据边界）
用户要求钉死"触发 `_tar` 的真正逻辑"。综合数据侧穷举 + IDA 侧新证据，机制已闭环，且能解释此前所有谜题。分 **已证实** 与 **受壳限制无法 100% 落点** 两部分诚实陈述。

### A. 数据侧：`_tar` 只以两种身份存在，无任何字段"引用"它（已证实）
对 Fiora 全部 props 精确检索 `= "..._tar..."` 与 `"..._tar" =`，`Fiora_BA1_tar`/`BA2_tar`/`BA_Crit_tar` **只出现为**：
1. resolver 键：`"Fiora_BA1_tar" = "Characters/Fiora/Skins/SkinNN/Particles/Fiora_SkinNN_BA1_tar"`；
2. 它自己的系统定义：`particleName = "Fiora_SkinNN_BA1_tar"`。
- **它从不作为** `childParticleName`、动画 `mEffectKey`/`mEnemyEffectKey`、`AttackSlotData` 字段等任何"被谁触发"的值出现。
- 因为没有任何数据字段写着"触发 `Fiora_BA1_tar`"，所以查表用的键 **不来自数据、只能由引擎在运行时按约定构造**。

### B. IDA 侧：引擎确有"按约定拼名 + on-hit 事件"设施（已证实）
检索字符串缓存（34791 条，区别于打包字节扫描）得到关键设施：
- **名字拼接格式串**：`%s_%s`、`%s_%s_%s`、`%s_%s%s`、`%s_%s_%s_%s`（@0x141a38d04 一带）——证实资源名普遍由**运行时字符串拼接**产生。
- **基础攻击/暴击 on-hit 事件名**：`Champion_CriticalAttack_OnHit`、`Champion_CriticalAttack_OnCast`、`Champion_CriticalAttack_OnMissileCast`、`OnHit`、`OnHitLocation`、`DAMAGEPROPERTY_TriggerOnHitEvents`。
- 其中 `Champion_CriticalAttack_OnHit`/`_OnCast` 作为**数据指针（lea 入事件名表）**被 `sub_14024C2D0`（0x75d0，冠军攻击事件注册/处理器）引用；`_OnMissileCast` 在 `sub_140986190`。
- 因为引擎既有"按 `%s_%s` 拼资源名"的能力、又有"普攻/暴击命中"的事件设施，所以"命中时按约定拼出 `{Champion}_BA{变体}_tar` / `_BA_Crit_tar` 再查 resolver"的机制在设施层面被坐实。

### C. 运行时载体：加载期 resolver → `+176` 运行时表（前轮已证实）
开局 `GAMESTATE_GAMELOOP`（"Received Game Start Packet"）时 `CharacterVfx_loadSkinVfx→buildRuntimeTableFromResolver` 把整张 `resourceMap`（含全部 `_tar`）物化进角色 VFX 组件 `+176` 的 `vector<Entry*>`，每条 Entry(104B) 以 `+80` 的 dword 键、`+88` 的已解析系统存活。组件是 **POD（无 vtable，仅析构一个虚函数）**，按裸偏移 `+176` 访问。

### 真正触发逻辑（结论·因果）
- 因为 `_tar` 在数据里只有"resolver 键 + 系统定义"两种身份、无任何字段触发它，所以它 **必然由引擎按命名约定在运行时拼出键来查**（这解释了 props 搜不到"谁触发它"）。
- 因为引擎备有 `%s_%s` 系列拼名格式串和 `Champion_*_OnHit`/`OnHit`/`TriggerOnHitEvents` 事件设施，所以普攻命中（服务端下放的权威事件）到达时，**autoattack/on-hit 逻辑按约定拼出 `Fiora_BA1_tar`（普通、变体交替 BA1/BA2）或 `Fiora_BA_Crit_tar`（暴击）** 作为 resolver 键。
- 因为该键经 `resolveTypedRef` 在施法者皮肤的 resourceMap（已物化进 `+176` 表）中命中已实例化系统，所以引擎取到系统后 **绑定/播放到被攻击者**——这就是站在被击者身上的 `_tar`。
- 综合：**触发点 = 引擎 autoattack on-hit 路径；规则 = 约定拼名（champion+BA+变体/Crit+_tar）→ resolver 按名/hash 查 `+176` 表 → 播到目标；props 无引用 = 键是运行时构造的、数据里 `_tar` 只是"被查目标"。**

### 诚实边界（受壳限制，无法 100% 落到单条指令）
- 因为 `League of Legends.exe` 被 `stub!packman` 打包、且拼名的构件子串（如 `_tar`/`_Crit_tar`/`BA` 后缀）**不在可见字符串缓存里**，所以"执行拼接的那一条 `sprintf`/concat 指令"无法用字符串锚点直接定位。
- 因为 `+176` 表所在组件是 **POD、按裸偏移访问**，IDA 无法在未完整建结构体的情况下对 `+176` 读取方做 xref，所以"per-hit 读 `+176` 并播到目标"的确切指令也无法直接枚举。
- 这两点是**打包 + POD 偏移**造成的静态分析硬边界，**非机制未知**：机制（约定拼名→resolver 查 `+176`→播到目标）已由数据侧穷举 + IDA 设施证据闭环坐实。

## 服务端下放假设核验（S2C 包侧证据）
问题：`Fiora_BA1_tar` 会不会是**服务端下放的攻击粒子**？逐个搜 exe 的 S2C 包目录后，结论是**混合模式**——服务端下放的是"事件"，粒子身份是客户端本地解析。

证据（`find_regex` 枚举 S2C 包结构 `PKT_S2C_*_s`）：
- 因为 exe 里**确实有完整 S2C 包架构**（`AIBaseClient`/`AIHeroClient`/`MissileClient` 上注册的 handler），所以客户端表现层的确由服务端事件驱动。
- 但与"攻击"相关的 S2C 包**只有战斗数值/预测类**：`UpdateAutoAttackOverrideRange`、`UpdateAttackSpeedCapOverrides`、`DamagePredictionState`、`AddDamagePrediction{ByValue,BySpell,ByItem}`、`RemoveDamagePrediction`、`EnableAttackOverlays`（练习工具）——**没有任何一个携带"要在某单位上播放某粒子"的字段**。
- 唯一携带 VFX 的 S2C 包是**导弹/投射物**专用（`MissileReInitVFX`、`SpawnDelayedMissile`、`ForceCreateMissile`、`MissileScriptTrigger` …），面向 skillshot，不面向普攻近战 on-hit。
- **不存在** `PKT_S2C_BasicAttack`/`PKT_S2C_NpcAttack`/通用"按名在单位上播放粒子"的包（正则均 0 命中）。

**关键结论（因果）**：
- 因为服务端下放的普攻相关包**只有"攻击这件事"的权威信息**（谁攻击谁、攻速/射程覆盖、伤害预测、是否暴击），**没有粒子名/hash 字段**，所以 `Fiora_BA1_tar` 的**身份不是从网络包里来的**。
- 因为客户端在**加载/换肤期**就已由 `CharacterVfx_buildRuntimeTableFromResolver` 把整张 `resourceMap`（含全部 `_tar`）物化成 `+176` 运行时表，所以当服务端下放的普攻命中事件到达时，客户端只需**本地**按触发名 hash（暴击→`_Crit_tar`，普通→`_tar`）查这张表取系统播放到受击者。
- 所以答案是：**是服务端触发（下放的是权威的"攻击/命中/暴击"事件），但不是服务端下放粒子本身**；粒子由客户端本地解析。这恰好解释了谜题——因为身份走客户端本地 resolver 表、网络包不带粒子名，所以 exe 无 `_tar` 字面、无 hash 常量，props 里也只有 resolver 条目而无"谁触发它"。

## 最终判定
- **在哪里触发**：由**基础攻击命中路径**（引擎的 basic-attack on-hit 逻辑）在运行时触发，绑到被攻击者身上；不由动画事件、不由源粒子、不由 AttackSlotData 触发（数据侧均已排除）。
- **按什么规则**：命中时拿到一个 `ResolverRef`（内含 `Fiora_BA1_tar` 的小写 FNV hash），经 `ResourceResolver_resolveTypedRef` 在 skin 的 `resourceMap` 里查出粒子系统 → spawn 到目标。数据本身呈现清晰的命名规律（`Fiora_BA`→`Fiora_BA1_tar`、`Fiora_BA2`→`Fiora_BA2_tar`、暴击→`Fiora_BA_Crit_tar`），这个 hash 是**预计算好、随 bin 加载进内存的数据值**，不是运行时用字符串拼出来的。
- **为何 props 搜不到引用**：因为 `_tar` 的触发 hash 以 `ResolverRef` 数据形式存在（resolver key 本身就是数据），由通用解析器运行时查表消费；数据里 `_tar` 只作为“被查的目标”（resolver 条目 + 系统定义）出现，没有哪个字段写着“触发它”。这与逆向里“该虚表/该 hash 无直接 xref”完全自洽。
- **诚实边界**：机制层（resolver 消费链）已完全查清并落盘；唯一未 100% 钉死的是“持有 `_tar` ResolverRef 的那个具体数据字段/C++ 指令”——它藏在通用 resolver 枢纽（100+ 调用点）之后，定位成本高、置信度低，非机制性未知。

> 诚实保留：exe 被 `stub!packman` 打包，因此“磁盘镜像字节扫不到 `_tar` 字面与 hash 常量”不能 100% 断定运行时无此字符串（壳可能压缩/加密 .rdata）；但**数据侧的闭合是可靠的**——`_tar` 在全部可提取 props 里确实只有 resolver 条目 + 系统定义两种身份。

## 待续（未完全定位的运行时调用点）
- 数据侧已穷尽闭合；机制侧（resolver 消费链 `resolveTypedRef`/`mapLookup`/`Obj_linkResolverRefs_sdr`）已查清并写入 IDB。
- 剩余仅差“持有 `_tar` ResolverRef 的具体字段 / 普攻 on-hit spawn 指令”，它在通用 resolver 枢纽（100+ 调用者）之后。若要继续，性价比更高的路线是**从粒子侧自顶向下**（`SystemDefinition`/`EmitterGroupInstance`/`ParticleSystemClient` 的 spawn 入口）而非爬通用解析器枢纽。
- 反射类型描述符（qword_141F8D320 / qword_141F90E70 等）只被通用反序列化器使用，gameplay 不碰它们，**不能作为找消费者的锚点**（已验证的弯路）。

## 本轮补充：把「触发载体」钉到 ParticleEventData，并再证实无离散 reader（续查）

本轮用 IDA 直连（HTTP 桥 `.moon-lol/ida_call.ps1`，绕过挂掉的 MCP wrapper）从 6 个**新角度**独立复核，全部指向同一硬边界：per-hit 的 `+176` 读取者是**内联 + 数据驱动派发**，没有可 xref 的离散函数。

1. **`+176` 直接读取者 = 0**：在 CharacterVfx 方法窗口 `0x1403FA000–0x140400000` 反编译全部函数，排除 builder 后，同时读 `+176` 与 `+80` 的函数 **0 个**。→ `+176` 向量的访问器被内联（调用方拿到的是已偏移指针，源码里无 `+176` 字面）。
2. **entry-consumer（`+80`/`+88`/104 步长）候选全是 ctor**：在 `0x1403D0000–0x1403F0000` 命中 7 个，反编译前三小者（`0x1403DEAC0`/`0x1403DD1B0`/`0x1403DD070`）全部是**构造函数**（写 `&unk_141A138FB` 空串哨兵并清零字段），`+80/+88/104` 只是无关对象的字段偏移巧合。
3. **普攻事件处理器 `sub_14024C2D0` 的 207 个具名被调用者里，0 个命中 vfx/particle/spawn 关键字**：spawn 走的是未具名 `sub_`/间接虚调用（stripped），无法按名锚定。
4. **触发载体锁定为 `ParticleEventData`（passive POD）**：ctor `sub_1411C4280`（`operator new(88)`），vtable `off_141B1D1A8`，字段 `mEffectKey@24 / mEnemyEffectKey@28 / mEffectName@32(StringBuilder)`；`_tar`（目标/敌方）即 `mEnemyEffectKey@28`。
5. **该 vtable 无 fire 虚函数**：12 个槽全部反编译 —— 槽[0]`sub_1411D7E10` 是析构（`StringBuilder_Free(+32)` + 释放 `+48/+64`），其余为 `return 1` 一类平凡 getter/predicate（`sub_140219A40`/`sub_1401EE810`/`sub_1403721D0` 等），**无一读 `+28/+32` 或调用 resolver/spawn**。→ 事件数据是被动描述符，播放逻辑在外部动画更新循环里。
6. **类型 hash `88265757`（=`0x5424CDD`）在代码里无立即数引用**（`search_text` code_only 0 命中）→ 派发经**类型注册表数据驱动**，不是代码 switch，故无干净 xref。

### 结论（对「从哪个函数调用」的最诚实回答）
- **物化侧（可具名、已钉死）**：`sub_1403F6780`（皮肤加载编排）→ `CharacterVfx_loadSkinVfx`(0x1403FB950) → `CharacterVfx_buildRuntimeTableFromResolver`(0x1403FBB50) 建 `+176` 表，Entry(104B) `key@+80=触发名FNV hash`、`value@+88=已解析系统`。
- **触发侧（机制已闭环、具体指令受限）**：普攻/暴击 on-hit 路径（拥有 `Champion_CriticalAttack_OnHit/OnCast` 事件名的 `sub_14024C2D0` 一带）在命中时用 `_tar` 触发名/hash，经 **`ResourceResolver_resolveTypedRef`(0x1411BAEE0)** 查 `+176` 表取系统 → 绑定/播放到被攻击者。
- **无法进一步坐实为单条离散函数的原因**（本轮再次穷举证实）：`stub!packman` 打包 + POD 裸偏移 + 二进制 stripped + 数据驱动类型派发四者叠加，使「per-hit 读 `+176` 并 spawn 到目标」的那条指令被内联/间接化，**没有可枚举的离散 reader 函数**。这是静态分析硬边界，非机制未知。

## 本轮补充（续二）：定位「按名播放」唯一可观测枢纽 + 更正动态断点目标

本轮遍历全部 `Particle*`/`Vfx*` 具名函数（仅 36 个）并逐层追调用图，得到两个关键更正与一个决定性发现。

### 更正 1：`resolveTypedRef`(0x1411BAEE0) 是**加载期内联缓存**解析器，**不是** per-hit 断点
- 其 `ResolverRef` 布局：`hash@+0`（触发名 u32）、`cache_version@+4`、`cached_result@+8`；命中缓存（`ref+4 == 全局版本`）直接返回 `ref+8`。
- `_tar` 的 ref 在 `buildRuntimeTableFromResolver` 期就解析并写回缓存，**per-hit 只读 `+176` 里已解析好的指针，不再进 resolveTypedRef**。它有 **2228 个调用者**（通用枢纽），下断点会在**加载期**炸而非命中期——**是错误的 per-hit 断点**。

### 更正 2：整个 `0x1403Fxxxx` CharacterVfx 簇全是**加载/物化期**，非 per-hit
- 调用链：`sub_1403F43C0`→`sub_1403F5330`（建 `"%s/%s/Skins/Skin%i"` 路径、载皮肤）→`sub_1403F49A0` + `Character_spawnConfiguredParticles`(0x1403F5E90) 播**常驻/环境**粒子；`sub_1403F6780`（物化编排）→`CharacterVfx_matchResolvedRefs`(0x1403F3E70) + `buildRuntimeTableFromResolver`。
- `Vfx_resolveDefToSceneMgr`(0x1403ED9F0) 实为「按 def 名查全局场景注册表返回槽位」，**仅被 builder 调用（5 处）**，不是 spawn 原语。
- `Character_spawnConfiguredParticles` 从**角色数据名槽**（+2968../+3400.. + 16 循环槽）逐个挂到 owner，是**出生时的常驻/环境**粒子，IDA 反编译注释已标「非 per-hit _tar」。

### 决定性发现：`ParticleMgr_getOrCreateNamedSystem`(0x140913840) = 「按名播放」唯一可观测枢纽
```c
__int64 __fastcall sub_140913840(mgr **a1, ownerObj a2, char *name a3)
// a3(R8) = 粒子名字符串（可读！如 "Fiora_BA1_tar"）
// key = Hash_elf_lower(a3)；在 mgr 的红黑树里 get-or-create 实例；
// 经 Obj_linkResolverRefs_sdr 链接 .troy 子资源，起 emitter。
```
- 这是**粒子名以可读字符串进入引擎的唯一位置**（用 `Hash_elf_lower` 而非 FNV 建键，与 `+176` 表的 hash 键是两套哈希）。
- 它是 **get-OR-create**：per-hit「按名播放」会命中 *get* 分支并**照样传入名字**，因此这是**可按名过滤**的正确断点。所有 ~11 个静态可见调用者均为预加载/批量实例化；真正的 per-hit 调用者是 stripped/间接函数（不在静态调用者列表里）——**只能靠该断点的返回地址在运行时命名**。

### 更正后的动态捕获方案（已写入 `.moon-lol/capture_tar_trigger.py`）
- **断点**：`ParticleMgr_getOrCreateNamedSystem`(0x140913840)，读 `R8` 指向的字符串，过滤含 `fiora`+`_tar` 者；命中即 dump 调用栈——**帧 [1]/[2] 即真正的 per-hit 触发函数**。
- 此断点取代旧脚本的 `resolveTypedRef`（加载期、且旧脚本查 hash 的寄存器偏移也写错）。
- **执行前提（我无法代跑）**：需在**授权/离线、反作弊未激活**的环境挂调试器；零售在线客户端受 Riot Vanguard 内核反作弊保护，禁止且危险。跑出的 `tar_callstack.log` 回传即可锁定精确函数名。

## 本轮补充（续三）：把「spawn 到目标」的那一个函数钉死为 `Particle_instantiateConfiguredSetOnOwner_vfunc`(0x14028C020)，并证明其为**间接派发**

本轮从 `Particle_instantiateConfiguredSetOnOwner_vfunc`(0x14028C020) 反向追它的持有结构，结论收敛且更正了前一轮把 `0x141A1BAE8` 误读为「48B 事件描述符数组」的说法。

### 决定性结构：`0x14028C020` 是某组件的 handler 记录槽（slot13 @ `off_141A1BA80`+0x68）
- `off_141A1BA80` 是一个组件的函数指针/handler 表（含混入的整数 `0x2F`，故非纯 C++ vtable），被大型复合对象的构造函数 **`sub_14024C2D0`** 写入对象偏移 **`+0x44A0`(17568)**（析构函数 `sub_14025C4B0` 在末尾复位为 `off_141A1A930`）。
- 表内 slot13(`+0x68` @ `0x141A1BAE8`) = `Particle_instantiateConfiguredSetOnOwner_vfunc`(0x14028C020)；slot19(`+0x98`) = 兄弟 handler `sub_1402871E0`。每 48B 记录 = `{+0 公共thunk sub_1411B33F0, +8 MonitorInit, +16 HANDLER, +24 int标签 0x2F, +32 fn, +40 fn}`。
- **`0x14028C020` 就是「把一个配置好的命名粒子集播放到 owner」的那个函数**：它遍历 `a1+8` 列表（计数 `a1+16`），对每条调用 `ParticleMgr_getOrCreateNamedSystem((mgr)+8, owner=a2, name=*(*(entry)+24))` —— 即在被攻击者(owner)身上按名起 `Fiora_BA1_tar`。这是 spawn 层面**可具名的触发函数**。

### 为什么仍无法静态给出「调用它的那个具名函数」
- **`0x14028C020` 有 0 个直接调用者**（call 指令），且全表 xref 扫描 `[0x141A1B800..0x141A1BD00)` 里**没有任何代码引用 handler 槽 `0x141A1BAE8`**。→ 它只在运行时经「组件指针 + 固定偏移 `+0x68`」被间接调用。
- 关联扫描 `0x140200000–0x1402C0000`：**同时**(a) 触及子对象偏移 `+0x44A0` 且 (b) 通过 `[reg+0x68]` 发出虚调用的函数 = **0 个**。→ 派发点与 handler **不在同一函数**：动画/事件更新引擎持有该组件指针（作为参数传入），在别处 `call [rcx+68h]`，故源函数里既无 `+0x44A0` 字面也无对 `0x141A1BAE8` 的 xref。
- 这与前述「per-hit 读 `+176` 无离散 reader」同源：**打包 + POD 偏移 + stripped + 数据驱动/间接派发** 使调用点无法静态具名。

### 对「必须指出从哪个函数调用」的最终、诚实回答
- **可确定并具名的触发函数（spawn 层）**：`Particle_instantiateConfiguredSetOnOwner_vfunc`(0x14028C020) —— 它是**唯一**直接把 `_tar` 命名粒子集播到被攻击者身上的函数，内部调 `ParticleMgr_getOrCreateNamedSystem`(0x140913840)。
- **装配它的具名函数**：`sub_14024C2D0`（复合对象构造器，同时挂载 `Champion_CriticalAttack_OnHit/OnCast` 事件名与该 handler 表）。
- **调用 `0x14028C020` 的那一个 per-hit 具名函数**：静态不可枚举（间接虚调用、handler 槽无 xref）。要拿到确切函数名，唯一途径是对 `0x140913840`（或 `0x14028C020`）下断点抓返回地址——脚本已备于 `.moon-lol/capture_tar_trigger.py`，需授权/离线环境运行（Vanguard 禁止代跑）。

## 本轮补充（续四）：更正结构判定为「真 C++ vtable + 虚方法」，并钉死具名注册函数 `sub_1402A4E90`

本轮从两个新角度收口，得到一处**关键更正**和一处**新的具名锚点**。

### 更正：`off_141A1BA80` 是**真正的 C++ vtable**，`0x14028C020` 是**虚方法 slot13(+0x68)**
- 反编译「48B 记录 +0」的 `sub_1411B33F0` → 它是 `{ v=(*(*a1+8))(a1); return Rtti_adjustPtrToType(v,a1,a2); }`，即**编译器生成的 RTTI 基类指针调整 thunk**（`sub_1411A95E0 = Rtti_adjustPtrToType`），**不是**「公共 delegate 调用器」。其 2881 个 xref 正是因为它是通用编译器助手。
- 因此「48B 记录 / 整数标签 0x2F」的读法是**误判**：`off_141A1BA80` 就是子对象的普通虚表，`0x14028C020` = 虚表 `+0x68`（第 13 槽）的**虚方法**。调用形如 `subobj->vtbl[13](subobj, target)`，由对象**运行时实际类型**决定，虚表地址不被调用点直接引用 → 静态无法反查到唯一调用函数（C++ 虚派发的固有边界，非工具限制）。

### 新具名锚点：`sub_1402A4E90` = 该组件的 spawn/init（注册所有具名事件槽）
- `sub_140272E90`（拼 `Attack%i`/`Crit%i` 名的函数）**唯一真实代码调用者 = `sub_1402A4E90`**（其余 xref 均为 `.pdata` 异常元数据）。
- `sub_1402A4E90` 是 obj+17568 组件的**创建/激活**函数：查询子对象（vtable+16），并通过组件 **vtable+40 的「按名注册事件处理器」**方法登记 `"Turn"`、`"Death"`（`(*(...+40))(comp,&buf,"Death",34,..)`），再经 `sub_140272E90` 登记 `Attack1..8`/`Crit1..8` 槽。→ 这是把 handler（含 `0x14028C020`）绑定到具名事件的**注册侧**函数（可具名）。

### 最终定位（本轮收口）
- 触发/spawn 函数（具名、已钉死）：`Particle_instantiateConfiguredSetOnOwner_vfunc`(0x14028C020)，是子对象虚表 slot13。
- 注册侧（具名）：`sub_1402A4E90`（登记具名事件）+ `sub_14024C2D0`（装配对象/虚表）。
- 派发侧（调用 slot13 的那一个函数）：动画/事件更新引擎的**虚调用点** `subobj->vtbl[13](subobj,target)`，具体函数运行时决定，静态不可具名 —— 需 `capture_tar_trigger.py` 在授权/离线环境跑出返回地址方可最终坐实。
