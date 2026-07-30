# 配置 → 描述符 反序列化 & 默认值表

> 上级：[shader.md](../shader.md)

## 三、配置 → 描述符 反序列化

因为存在一套基于字符串键（内部哈希）的属性解析器，所以反序列化入口链为：

| 函数 | 职责 |
|------|------|
| sub_1412CE0F0 | 顶层：遍历 `GroupPart`+N，读 `Type`(Simple/Complex)，调 `ParticleEmitterDescriptor_Init` 分配 1136B 描述符，再分发到子解析器 |
| sub_1412CE810 | 每 emitter 通用属性（importance、+720 标志位） |
| sub_1412CEB10 | complex emitter 大量渲染属性映射 |
| sub_1412D0730 | simple emitter 分支 |

读取函数签名（**最后一个参数就是「配置缺失时的默认值」**）：

| 函数 | 类型 |
|------|------|
| sub_1411477B0(ctx,out,section,key,default) | string |
| sub_141147EC0(...,default) | bool |
| sub_141147470(...,default) | int |
| sub_141147670(...,default) | float |
| sub_141147A20(...,default) | vec2 |
| sub_141147CA0(...,default) | vec |

但大多数属性并不在解析器给 default，而是「配置里没有就保留 `ParticleEmitterDescriptor_Init` 写入的初始值」，因此真正的默认值以下表（构造函数）为准。

---

## 四、运行时描述符默认值表（来自 sub_1412A4530）

常量速查：`0x3F800000`=1.0、`0x40400000`=3.0、`0x7F7FFFFF`=FLT_MAX、`0x3F000000`=0.5。

| 偏移 | 默认值 | 含义 / 对应配置键 |
|------|------|------|
| +16 dword | 0x07000000 | +17 p-backfaceon=0、+18 p-texture-pixelate=0、+19=7 |
| +20 | 0x02000000 | +20 rendermode=0、+23 importance=2（Medium→3） |
| +96 | 新建 primitive | `*(+96)+8` = renderType 字节(0=quad…) |
| +104 | 0 | 预加载/override shader（空） |
| +112 word | 0 | pass |
| +114 | 0 | flag-disable-z / disable-fow 位域 |
| +115 | 1 | flag-force-animated-mesh-z-write=**1** |
| +116 | 0 | teamcolor-correction flag=0 |
| +117 | 0 | blendMode(rendermode)=0 |
| +118 | 0 | e-stencil-ref=0 |
| +119 | 5 | e-alpharef=**5**（ALPHA_TEST 阈值=val/255） |
| +124,+128 | {0,0} | e-depthbiasfactors=**{0,0}**（与哨兵 qword_141F76628={0,0} 比较，不等才开启） |
| +132 | 0 | p-alphaslicerange=0（SLICE_RANGE，>0 才上传） |
| +136 | 1.0 | — |
| +140,+144 | {1.0,1.0} | p-texdiv=**{1,1}**（TEXTURE_INFO 纹理分块） |
| +152 | 空字符串 | 主纹理(p-texture)路径=空 |
| +168 | DefaultColorOverlifetime.png | p-rgba / 颜色查表纹理 |
| +184 | DefaultFalloff.png | p-falloff-texture |
| +224..+236 | {1,1,1,1} | e-color-modulate=**白色**（kColorFactor） |
| +248 | 3.0 | p-life=**3.0** |
| +264,+268 | FLT_MAX | e-life=**FLT_MAX**（无限） |
| +288/+360/+408/+432/+576 | {0,0,0} | p-vel/p-accel/p-drag/p-orbitvel/p-postoffset 默认零向量 |
| +336 | {1,1,1,1} | e-rgba=白色 |
| +456 | 1.0 | e-framerate=**1.0** |
| +496 | 0 | e-rate=0 |
| +544,+548 | FLT_MAX | e-active / e-period=**FLT_MAX** |
| +624..+636 | {1,1,1,1} | e-censor-modulate=白色 |
| +700/+1000/+1056 | {1,1,1} | 默认单位缩放曲线(p-scale 等) |
| +720 dword | 0x00202000 | 标志位：**0x200000=e-local-orient 默认开** + 0x2000 |
| +820 | 1.0 | — |
| +828 | 0 | PARTICLE_DEPTH_PUSH_PULL=0 |
| +1104,+1108 | {1.0,1.0} | — |
| +1120,+1124 | {0.5,0.5} | — |

对应用户配置例子：`isLocalOrientation=false` 会清掉默认开的 +720&0x200000；`depthBiasFactors={-1,-80}` 写入 +124/+128；`blendMode` 写 +117；`birthColor` 写 +224（kColorFactor）。

### shader 端关键常量默认值

| 地址 | 值 | 用途 |
|------|------|------|
| qword_141F76628 | {0,0} | depthbias 哨兵默认（等于它则不写深度偏移） |
| xmmword_141B2B610 | {1,1,1,0} | PS 路径 TEXTURE_INFO_2 默认 |
| xmmword_141A3EA40 | {0,0,0,1} | vFresnel/FRESNEL 默认 |
| xmmword_141A1FD70 | {1,1,1,1} | 白色（team-color/颜色默认） |
| dword_141A17D80 | 1.0 | 求倒数分子 |
| dword_141A1FC28 | ≈1e-8 | fadeRange 除零保护 epsilon |
| 立即数 4CBEBC20h | ≈1.0e8 | fadeDistance(beginOut)≤0 时的兜底值 |
| xmmword_141E49AA0 | 单位矩阵行 | mWorld 默认 |
