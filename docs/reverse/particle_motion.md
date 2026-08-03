# 英雄联盟粒子运动与发射属性逆向分析

> 本文档通过 IDA Pro 对 `League of Legends.exe` 进行静态逆向分析，彻底厘清 `VfxEmitterDefinitionData` 中所有运动、物理与发射控制属性在二进制引擎内部的具体解析函数、C/C++ 反编译代码、内存布局以及运动积分与 Billboard 顶点装配算法。

---

## 目录

1. [属性二进制 Key 与解析映射表](#1-属性二进制-key-与解析映射表)
2. [配置反序列化与解析逻辑 (`ParticleEmitter_ParseComplex`)](#2-配置反序列化与解析逻辑-particleemitter_parsecomplex)
3. [动态插值与概率曲线评估算法 (`ParticleEmitter_EvaluateSplineCurve`)](#3-动态插值与概率曲线评估算法-particleemitter_evaluatesplinecurve)
4. [物理运动积分与 Billboard 顶点装配 (`ParticleEmitter_FillQuadVertices`)](#4-物理运动积分与-billboard-顶点装配-particleemitter_fillquadvertices)
5. [数学公式汇总](#5-数学公式汇总)

---

## 1. 属性二进制 Key 与解析映射表

英雄联盟粒子系统在加载 `.troy` 或 BIN 提取的 `VfxEmitterDefinitionData` 时，使用统一样式的 key-value 解析器（如 `sub_141147670` 浮点数、`sub_141147CA0` 向量、`sub_1412CDD10` 动态曲线）。以下为 16 个核心运动/控制属性在二进制中的具体映射关系：

| 属性名称 | BIN/PROP 示例 | IDA 二进制 Key | 解析函数 | 描述符偏移 | 参与计算环节 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `timeBeforeFirstEmission` | `timeBeforeFirstEmission: f32 = 0.05` | `"e-timeoffset"` | `sub_141147670` | `+536` | 首次发射延时计算 |
| `rate` | `rate: ValueFloat` | `"e-rate"` | `sub_1412CDAC0` | `+496` / `+528` | 发射速率与粒子累积分布 |
| `particleLifetime` | `particleLifetime: ValueFloat` | `"p-life"` | `sub_1412CDAC0` | `+248` / `+272` | 单个粒子的存活时长 $T_{\text{life}}$ |
| `particleLinger` | `particleLinger: option[f32]` | `"p-linger"` | `sub_141147670` | `+280` | 粒子死亡后的残留/衰减时长 |
| `lifetime` | `lifetime: option[f32]` | `"e-life"` | `sub_141147670` | `+264` | 发射器本身的生命周期 |
| `isSingleParticle` | `isSingleParticle: flag = true` | `"single-particle"`| `sub_141147470` | `+720` (`bit 0x100000`) | 单发模式控制（发射完立即销毁） |
| `birthVelocity` | `birthVelocity: ValueVector3` | `"p-vel"` | `sub_1412CDD10` | `+288` | 粒子初速度向量 $\mathbf{v}_0$ 采样 |
| `birthDrag` | `birthDrag: ValueVector3` | `"p-drag"` | `sub_1412CDD10` | `+408` | 速度阻力衰减因子 $\mathbf{k}_{\text{drag}}$ |
| `worldAcceleration` | `worldAcceleration: IntegratedValueVector3`| `"p-accel"` | `sub_1412CDD10` | `+360` | 世界/发射器坐标系物理加速度 $\mathbf{a}$ |
| `SpawnShape` | `emitOffset`, `emitRotationAngles` | `"p-offset"`, `"e-rotation1"` | `sub_1412CDD10` / `0x14129A180` | `+568` / `+576` | 发射形状空间中的坐标与旋转轴 |
| `EmitterPosition` | `EmitterPosition: ValueVector3` | `"p-postoffset"` | `sub_1412CDD10` | `+576` | 发射器根节点全局位置偏移 |
| `isDirectionOriented`| `isDirectionOriented: flag = true` | `"e-local-orient"` | `sub_141147EC0` | `+720` (`bit 0x200000`) | 控制粒子是否随速度矢量方向定向 |
| `birthRotation0` | `birthRotation0: ValueVector3` | `"p-quadrot"` | `sub_1412CDD10` | `+976` | 初始旋转角 $\theta_0$ |
| `birthScale0` / `scale0`| `birthScale0` / `scale0: ValueVector3` | `"p-scale"` | `sub_1412CDD10` / `sub_1412D7D40` | `+320` / `+1000` | 初始与随生命周期缩放曲线 $\mathbf{S}(t)$ |
| `FlexShapeDefinition` | `FlexShapeDefinition: VfxFlexShapeDefinitionData` | `"p-flex-scale"` | `sub_1412CDD10` | `+320` | 根据目标对象尺寸/高度/半径调节 birthScale0 增益 |
| `alphaErosionDefinition` | `alphaErosionDefinition: VfxAlphaErosionDefinitionData` | `"p-alpha-erosion-map"` / `"ALPHA_EROSION"` | `sub_1412D36B0` / `0x1412B7330` | Slot 9 (`sAlphaErosionTexture`) | PS 阶段通过侵蚀纹理与驱动曲线切割/消融 Alpha 透明度 |

---

## 2. 配置反序列化与解析逻辑 (`ParticleEmitter_ParseComplex`)

在函数 `ParticleEmitter_ParseComplex` (`0x1412CEB10`) 中，引擎将文本/PROP 配置解包并填充到 `ParticleEmitterDescriptor` 结构体中。

### 反编译核心代码片段 (`0x1412CEB10`)

```c
// 解析 ParticleEmitterDescriptor (Type=="Complex" 默认路径)
__int64 __fastcall ParticleEmitter_ParseComplex(__int64 *a1, __int64 a2, __int64 a3)
{
    // ...
    // 1. 粒子生命周期与发射器生命周期
    sub_1412CDAC0(a1, a3 + 248, v3, "p-life", 0x40400000); // 默认 3.0s
    sub_141147670(*a1, &v78, v3, "e-life", -1.0f);
    if (*(float*)v78 >= 0.0f)
        *(_DWORD*)(a3 + 264) = *(_DWORD*)v78;

    // 2. 单发粒子 (isSingleParticle)
    sub_141147470(*a1, &v78, v3, "single-particle", 0);
    if (*(_DWORD*)v78)
        *(_DWORD*)(a3 + 720) |= 0x100000u; // 设置标志位 bit 20

    // 3. 发射速率 (rate)
    sub_1412CDAC0(a1, a3 + 496, v3, "e-rate", 0);
    sub_1412D7E70(a1, a3 + 528, v3, "e-rate", &v82);

    // 4. 初速度 (birthVelocity)、加速度 (worldAcceleration / acceleration)、阻力 (birthDrag)
    sub_1412CDD10(a1, a3 + 288, v3, "p-vel", &dword_141F76630);   // p-vel -> +288
    sub_1412CDD10(a1, a3 + 360, v3, "p-accel", &dword_141F76630); // p-accel -> +360
    sub_1412CDD10(a1, a3 + 408, v3, "p-drag", &dword_141F76630);  // p-drag -> +408

    // 5. 旋转与轨道速度 (birthRotationalVelocity, birthOrbitalVelocity)
    sub_1412CDD10(a1, a3 + 1024, v3, "p-rotvel", &dword_141F76630);
    sub_1412CDD10(a1, a3 + 384, v3, "Emitter-BirthRotationalAcceleration", &dword_141F76630);
    sub_1412CDD10(a1, a3 + 432, v3, "p-orbitvel", &dword_141F76630);

    // 6. 本地朝向 / 定向标志 (isDirectionOriented / isLocalOrientation)
    sub_141147EC0(*a1, &v77, v3, "e-local-orient", 1);
    if ((_BYTE)v77)
        *(_DWORD*)(a3 + 720) |= 0x200000u; // 设置标志位 bit 21

    // 7. 发射坐标偏移 (SpawnShape.emitOffset & EmitterPosition)
    sub_1412CDD10(a1, v40 + 8, v3, "p-offset", &dword_141F76630);   // p-offset
    sub_1412CDD10(a1, a3 + 576, v3, "p-postoffset", &dword_141F76630); // p-postoffset

    // 8. 发射旋转轴与角度 (SpawnShape.emitRotationAngles & emitRotationAxes)
    sub_1401FFBE0(&v105, "e-rotation1");
    while (1) {
        // 循环读取 e-rotation1, e-rotation2 ... 及其 -axis 旋转轴
        sub_1411B8280(&v105, 11i64);
        // ...
        sub_141147CA0(*a1, &v91, v3, v105, &v92); // 读取 -axis 向量
        Vec3_Normalize(v78);                    // 归一化旋转轴
    }

    // 9. 停留衰减 (particleLinger)
    sub_141147670(*a1, &v78, v3, "p-linger", v59 + 10.0f);
    *(_DWORD*)(v54 + 280) = *(float*)v78 < 0.0f ? 0x7F7FFFFF : *(_DWORD*)v78;

    // 10. 尺寸与旋转 (birthScale0, birthRotation0)
    sub_1412CDD10(a1, a3 + 976, v3, "p-quadrot", &dword_141F76630);
    sub_1412CDD10(a1, a3 + 1000, v3, "p-scale", &qword_141E49A40);
    sub_1412D7D40(a1, v54 + 320, v3, "p-scale", 0);
    // ...
}
```

---

## 3. 动态插值与概率曲线评估算法 (`ParticleEmitter_EvaluateSplineCurve`)

在粒子生成与随时间演化过程中，`birthVelocity`, `scale0`, `particleLifetime` 等带有 `VfxAnimated*VariableData` 或 `probabilityTables` 的属性由 `ParticleEmitter_EvaluateSplineCurve` (`0x1412C85A0`) 进行采样。

### C 语言反编译插值逻辑 (`0x1412C85A0`)

`ParticleEmitter_EvaluateSplineCurve` 内部构建 Hermite / Cubic B-Spline 曲线，并在概率区间内采用随机数生成插值点：

```c
// 采样三维曲线/概率表
__int64 __fastcall ParticleEmitter_EvaluateSplineCurve(__int64 *a1, __int64 a2, __int64 a3, unsigned int num_samples, char spline_type)
{
    // ...
    // 根据 spline_type 计算 Hermite 切线与三次多项式系数
    v38 = (float)(v33 - v32) + 0.00001f;
    v39 = (float)((v34 * v33) - (v37 * v32)) / v38;
    v41 = (float)((v37 - v34) / v38) * 0.5f;

    // 计算三次样条多项式: P(t) = a*t^3 + b*t^2 + c*t + d
    v40 = v25 - (float)((float)((float)(v41 * v32) + v39) * v32);
    // ...

    // 针对每个样本生成 3D 向量采样值 (X, Y, Z)
    for (i = 0; i < num_samples; ++i) {
        float t = (float)i / (float)(num_samples - 1); // 归一化进度 t ∈ [0, 1]
        
        // Hermite / Bezier 三次多项式求值:
        float x = ((t * coeff_a.x + coeff_b.x) * t + coeff_c.x) * t + coeff_d.x;
        float y = ((t * coeff_a.y + coeff_b.y) * t + coeff_c.y) * t + coeff_d.y;
        float z = ((t * coeff_a.z + coeff_b.z) * t + coeff_c.z) * t + coeff_d.z;

        out_samples[i] = Vec3(x, y, z);
    }
    return result;
}
```

---

## 4. 物理运动积分与 Billboard 顶点装配 (`ParticleEmitter_FillQuadVertices`)

`ParticleEmitter_FillQuadVertices` (`0x1412C1340`) 是粒子渲染前最关键的核心更新函数。它为每个粒子计算当前帧的世界位置 $\mathbf{p}(t)$、速度 $\mathbf{v}(t)$、朝向矩阵，并生成 Billboard 四边形的 4 个顶点坐标。

### 反编译代码剖析 (`0x1412C1340`)

```c
__int64 __fastcall ParticleEmitter_FillQuadVertices(__int64 a1, __int64 a2)
{
    __int64 v2 = *(_QWORD *)(a2 + 24);
    // ...
    // 读取粒子状态数组与描述符
    v118 = *(_QWORD **)(v2 + 16);
    v7 = v118[2]; // ParticleEmitterDescriptor 指针
    
    // 检查定向标志 isDirectionOriented (bit 0x200000) 与 local orientation
    v20 = *(_BYTE *)(v7 + 20);
    v21 = !v20 || v20 == 2;
    v67 = (*(_DWORD *)(v64 + 720) & 4) != 0; // isDirectionOriented 开关

    for (int i = num_particles - 1; i >= 0; --i)
    {
        // 1. 读取粒子的基础速度 v0、加速度 a 与阻力 drag
        float3 v_0 = particle_vel_array[i];
        float3 accel = particle_accel_array[i];
        float3 drag = particle_drag_array[i];
        float cur_time = particle_time_array[i]; // 存活时间 t

        // 2. 位置与速度积分 (Euler / Verlet 积分)
        // v(t) = v0 + accel * t
        // pos(t) = pos_spawn + v0 * t + 0.5 * accel * t^2 - drag * t
        float3 current_pos = spawn_pos + v_0 * cur_time + 0.5f * accel * (cur_time * cur_time);
        if (has_drag) {
            current_pos -= drag * (0.5f * cur_time * cur_time);
        }

        // 3. 计算 Billboard 基轴 (Right & Up 向量)
        // 如果 isDirectionOriented 为 true，根据 v(t) 方向构建基轴
        float3 right_axis, up_axis;
        Particle_ComputeBillboardAxes(
            &v125,               // 运动速度方向向量 v(t)
            qword_141EED428,     // 相机 View 矩阵
            *v121,               // 旋转角
            v68,
            v67,                 // isDirectionOriented 标记
            v124,                // 变换矩阵
            has_post_rotate,
            &right_axis,
            &up_axis
        );

        // 4. 应用 birthScale0 与 scale0 缩放
        float scale_x = scale_curve.x * base_scale.x;
        float scale_y = scale_curve.y * base_scale.y;

        float3 scaled_right = right_axis * (scale_x * 0.5f);
        float3 scaled_up    = up_axis    * (scale_y * 0.5f);

        // 5. 生成 Billboard 四边形的 4 个顶点坐标 (V0, V1, V2, V3)
        // V0 = pos - scaled_right - scaled_up
        // V1 = pos + scaled_right - scaled_up
        // V2 = pos + scaled_right + scaled_up
        // V3 = pos - scaled_right + scaled_up
        vertices[0].pos = current_pos - scaled_right - scaled_up;
        vertices[1].pos = current_pos + scaled_right - scaled_up;
        vertices[2].pos = current_pos + scaled_right + scaled_up;
        vertices[3].pos = current_pos - scaled_right + scaled_up;

        // 6. 填充 UV 纹理坐标与顶点颜色
        Color_PackToDWORD(&packed_color, &color_over_lifetime);
        vertices[0].color = packed_color;
        // ...
        Particle_FillQuadVertexUVs(v92, v91, v73, &v134, &v136);
    }
    return result;
}
```

---

## 5. 数学公式汇总

根据反编译代码推演，英雄联盟粒子系统运动与几何生成的闭式数学表达如下：

### 1. 发射位置与发射形状变换矩阵

$$\mathbf{Q}_{\text{shape}} = \prod_{i=0}^{n} \text{Quat::from\_axis\_angle}\left(\frac{\mathbf{axis}_i}{\|\mathbf{axis}_i\|}, \theta_i(t)\right)$$

$$\mathbf{p}_{\text{spawn}} = \mathbf{p}_{\text{emitter}} + \mathbf{Q}_{\text{shape}} \cdot \mathbf{o}_{\text{shape}}$$

其中：
- $\mathbf{p}_{\text{emitter}}$ 为发射器基础偏移 (`EmitterPosition` / `"p-postoffset"`)。
- $\mathbf{o}_{\text{shape}}$ 为形状局部偏移 (`SpawnShape.emitOffset` / `"p-offset"`)。
- $\mathbf{Q}_{\text{shape}}$ 为由 `emitRotationAngles` 与 `emitRotationAxes` 在二进制 `sub_1412D6A30` 中构成的轴角连乘旋转四元数。

### 2. 物理运动二阶积分 (位置与速度)

在时间 $t$ 处粒子的位置 $\mathbf{p}(t)$ 与速度 $\mathbf{v}(t)$ 为：

$$\mathbf{v}(t) = \mathbf{v}_0 + \mathbf{a} \cdot t - \mathbf{k}_{\text{drag}} \odot \mathbf{v}(t) \cdot t$$

$$\mathbf{p}(t) = \mathbf{p}_{\text{spawn}} + \mathbf{v}_0 \cdot t + \frac{1}{2} \mathbf{a} \cdot t^2 - \frac{1}{2} \mathbf{k}_{\text{drag}} \odot \mathbf{v}_0 \cdot t^2$$

其中：
- $\mathbf{v}_0$ 为初速度 (`birthVelocity` / `"p-vel"`)。
- $\mathbf{a}$ 为物理加速度 (`worldAcceleration` / `"p-accel"`)。
- $\mathbf{k}_{\text{drag}}$ 为空气阻力衰减系数 (`birthDrag` / `"p-drag"`)。

### 3. 定向与 Billboard 顶点坐标生成

若 `isDirectionOriented == true`，Billboard 基轴 $\mathbf{R}_{\text{axis}}$ 和 $\mathbf{U}_{\text{axis}}$ 由局部速度方向定义：

$$\mathbf{F}_{\text{axis}} = \frac{\mathbf{v}(t)}{\|\mathbf{v}(t)\|}, \quad \mathbf{R}_{\text{axis}} = \frac{\mathbf{F}_{\text{axis}} \times \mathbf{V}_{\text{cam}}}{\|\mathbf{F}_{\text{axis}} \times \mathbf{V}_{\text{cam}}\|} , \quad \mathbf{U}_{\text{axis}} = \mathbf{F}_{\text{axis}} \times \mathbf{R}_{\text{axis}}$$

粒子的 4 个顶点 $\mathbf{V}_k$ ($k \in \{0, 1, 2, 3\}$) 最终坐落于：

$$\mathbf{V}_k = \mathbf{p}(t) \pm \frac{S_x(t)}{2} \cdot \mathbf{R}_{\text{axis}} \pm \frac{S_y(t)}{2} \cdot \mathbf{U}_{\text{axis}}$$

其中 $S_x(t), S_y(t)$ 由 `birthScale0` $\times$ `scale0(t)` 计算得到。
