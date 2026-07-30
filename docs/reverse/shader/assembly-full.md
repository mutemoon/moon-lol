# Full 版完整装配逻辑（Rust 伪代码）

> 上级：[shader.md](../shader.md)。核心概览（公共流程/两版差异/虚表分发）见 [assembly.md](assembly.md)。
> 来源：`ShaderEffect_SetupParticleShader_Full`(sub_1412B7330) + `ShaderEffect_BuildShaderPathAndDefines`(sub_1412DD450) + `ShaderEffect_BindTextureSlot`(sub_1412AD400) 三者反编译逐行核对。涵盖 shader 选择、宏开关、cbuffer 上传、纹理+采样器数据填充、逐-pass 渲染态。

## 纹理槽 → 常量名映射表（`off_141B26570`，BindTextureSlot 用 slot 索引取名）

| slot | 常量名 | Full 版用途 |
|------|------|------|
| 0 | `TEXTURE` | 主纹理（非 renderType7） |
| 1 | `PARTICLE_COLOR_TEXTURE` | color-over-life（强制 0x0101 线性-clamp） |
| 2 | `FALLOFF_TEXTURE` | falloff |
| 3 | `NORMAL_MAP` | distortion 法线 |
| 4 | `SAMPLER_BACK_BUFFER_COPY` | 引擎托管（不自建采样器） |
| 5 | `sDepthTexture` | 引擎托管 |
| 6 | `SAMPLER_FOW` | 切片资源复用位（off_141E4D520） |
| 7 | `TEXTUREMULT` | 第二纹理（过滤取自身） |
| 8 | `sPalettesTexture` | 调色板（过滤取 paletteObj[+16]） |
| 9 | `sAlphaErosionTexture` | alpha 侵蚀（过滤取 erosionObj[+34]） |
| 10 | `REFLECTION_MAP` | 反射（强制 0x0101） |
| 11 | `DIFFUSE_MAP` | renderType7 漫反射 |

## 装配主链（简化伪代码）

```rust
fn setup_particle_shader_full(fx: &mut ShaderEffect, ctx, path_base, force) -> bool {
    let desc = fx.descriptor();
    let render_type = desc.primitive.read_u8(8);       // *(primitive+8)
    if render_type == 12 { return false; }             // 12 型直接放弃

    // (A) override shader 短路（+104）
    if let Some(ovr) = desc.override_shader {
        fx.shader = shader_load_override(ovr, fx);      // sub_1412B6D30
        if let Some(sh) = fx.shader {
            fx.apply_team_color = shader_has_constant(sh, hash("APPLY_TEAM_COLOR_CORRECTION"));
            return true;
        }
    }

    // (B) 拼路径 + 加载主 shader（失败则追加 '*' 通配重试）
    let mut sb = StringBuilder::new();
    if force || fx.shader.is_none() { sb.from_object(desc); sb.push_char(':'); sb.concat(path_base); }
    else { sb.append(shader_variant_key(fx.shader, desc)); }
    fx.shader = shader_manager_load(&sb, force);        // ShaderManager_LoadShader
    while !force && fx.shader.is_none() { sb.push_char('*'); fx.shader = shader_manager_load(&sb, false); }

    // (C) 尺寸类常量
    set_const_vec4(fx.shader, "TEXTURE_INFO",   vec4(desc.tex_div_x, inv0(desc.tex_div_y), inv0(desc.tex_div_x), 0.0));
    let sec = get_secondary_texture_info(fx);           // sub_1412C73E0
    set_const_vec4(fx.shader, "TEXTURE_INFO_2", vec4(sec.width, inv0(sec.height), inv0(sec.width), 0.0));

    // (D) 颜色 / team-color（值来自 desc.color_modulate +224）
    declare_const_used(fx.shader, "kColorFactor", 1);
    declare_const_used(fx.shader, "APPLY_TEAM_COLOR_CORRECTION", 1);
    if fx.apply_team_color {
        let v = if desc.team_color_corr & 1 != 0 { load_const(0x141A1FD70) } else { vec4_zero() };
        set_const_vec4_by_handle(fx.shader, HANDLE_TEAM_COLOR, v);
    }

    // (E) UV 变换 + 扭曲
    declare_const_used(fx.shader, "vParticleUVTransformMult", 3);
    declare_const_used(fx.shader, "vParticleUVTransform", 3);
    if let Some(dist) = desc.distortion {               // +80
        set_const_vec4(fx.shader, "DistortionPower", vec4(dist.read_f32(0), 0.0, 0.0, 0.0));
        bind_texture_slot(fx, &mut fx.tex_distortion, dist.ptr()+8, 3, ctx);   // NORMAL_MAP
    }

    // (F) 调色板 / alpha 侵蚀（Full 版特有）
    if let Some(pal) = desc.palette {                   // +64
        bind_texture_slot(fx, &mut fx.tex_palette, pal, 8, ctx);
        declare_const_used(fx.shader, "cPaletteSelectMain", 1);
        declare_const_used(fx.shader, "cPaletteSrcMixerMain", 1);
    }
    if let Some(ero) = desc.alpha_erosion {             // +56
        bind_texture_slot(fx, &mut fx.tex_erosion, ero.ptr()+48, 9, ctx);
        declare_const_used(fx.shader, "cAlphaErosionParams", 1);
        declare_const_used(fx.shader, "cAlphaErosionTextureMixer", 1);
    }

    // (G) 基础纹理组
    bind_texture_slot(fx, &mut fx.tex_color_over_life, desc.color_tex,   1, ctx);
    bind_texture_slot(fx, &mut fx.tex_falloff,         desc.falloff_tex, 2, ctx);
    bind_texture_slot(fx, &mut fx.tex_secondary,       sec.ptr(),        7, ctx);

    // (H) 切片 SLICE_RANGE（>0 才启用）
    if desc.slice_range > 0.0 {                         // +132
        set_const_vec4(fx.shader, "SLICE_RANGE",
            vec4(desc.slice_range, 1.0/(desc.slice_range*desc.slice_range), 0.0, 0.0));
        bind_texture_slot(fx, &mut tmp, &off_141E4D520, 6, ctx);
    }

    // (I) 反射（Full 版特有）
    if let Some(refl) = desc.reflection {               // +72
        bind_texture_slot(fx, &mut fx.tex_reflection, refl, 10, ctx);
        declare_const_used(fx.shader, "vReflection", 1);
        declare_const_used(fx.shader, "vReflectionFColor", 1);
    }

    // (J) 固定常量：Fresnel(0,0,0,1) / depth push-pull(+828)
    declare_const_used(fx.shader, "COLOR_LOOKUP_UV", 1);
    declare_const_used(fx.shader, "vFresnel", 1);
    declare_const_used(fx.shader, "FRESNEL", 1);
    let fres = load_const(0x141A3EA40);
    set_const_vec4_by_handle(fx.shader, hash("vFresnel"), fres);
    set_const_vec4_by_handle(fx.shader, hash("FRESNEL"),  fres);
    set_const_vec4(fx.shader, "PARTICLE_DEPTH_PUSH_PULL", vec4(desc.depth_push_pull, 0.0, 0.0, 0.0));

    // (K) 软粒子（默认 NULL → 内部直接返回）
    set_soft_particle_params(fx);                       // sub_1412E39D0

    // (L) renderType==7：漫反射三纹理 + 固定采样态
    if render_type == 7 {
        declare_const_used(fx.shader, "COLOR_UV", 1);
        declare_const_used(fx.shader, "MODULATE_COLOR", 1);
        bind_texture_slot(fx, &mut fx.tex_diffuse, desc.primary_tex, 11, ctx);
        for name in ["DIFFUSE_MAP", "FALLOFF_TEXTURE", "PARTICLE_COLOR_TEXTURE"] {
            let slot = get_texture_slot(fx.shader, hash(name));
            slot.write_u16(16, 257); slot.write_u8(18, 1);   // 0x0101 线性-clamp
        }
    } else {
        bind_texture_slot(fx, &mut fx.tex_diffuse, desc.primary_tex, 0, ctx);
    }

    // (M) 遍历 7 个 pass：路径/宏 + 渲染态
    for pass in shader_pass_render_pass_table().iter().take(7) {   // byte_141E4D7D0 7x48B
        if !shader_pass_is_valid(fx, pass.id) { continue; }
        let variant = shader_defines_get_data(shader_pass_get_defines(fx.shader, pass.define_name));
        build_shader_path_and_defines(fx, ctx, pass.id, variant);
        apply_pass_render_state(fx, pass.id, render_type, variant);
    }

    // (N) finalize → 生成 D3D shader 对象
    shader_manager_finalize(fx.shader, 2)               // ShaderManager_FinalizeShader
}
```

## shader 选择 + 宏开关（BuildShaderPathAndDefines）

pass 语义：`a3==1`→shadow；`a3∈{2,3}`→distortion；其它→普通。geometry(`v14`)：0=quad/1=mesh/2=skinned。

```rust
// renderType==7 特例：走 Environment/UNLIT_DECAL_*，仅支持 MASKED/MULT_PASS/ALPHA_EROSION/PALETTIZE
if render_type == 7 {
    out.vs = hash("ASSETS/Shaders/HLSL/Environment/UNLIT_DECAL_VS.vs");
    out.ps = hash("ASSETS/Shaders/HLSL/Environment/UNLIT_DECAL_PS.ps");
    if fx.gap24 { /* 按能力位加上述四宏 */ }
    return;
}
// PS/VS 基路径 = f(geometry, distortion, shadow)：
//   quad:  DISTORTION_PS / SHADOW_QUAD_PS / QUAD_PS   (+ VS 对应)
//   mesh:  DISTORTION_MESH_PS / SHADOW_MESH_PS / MESH_PS
//   skin:  SkinnedMesh/PARTICLE_DISTORTION_PS / _SHADOW_PS / _PS
// UV 模式(desc.gap200[0]): 1=SCREEN_SPACE_UV, 2=SEPARATE_ALPHA_UV, 3..5=LOCAL_SPACE_UV
//   quad 时 1/2 改为追加 QUAD_ScreenSpaceUV / QUAD_(PS|VS)_FixedAlphaUV 片段
// 追加片段/宏：REFLECTIVE(quad 追加 MESH_PS/VS)、MULT_PASS、切片(QUAD/MESH/PARTICLE_PS_Slice)
// 全局宏：SOFT_PARTICLES、MASKED、DISABLE_FOW、ALPHA_EROSION、PALETTIZE_TEXTURES、USE_VERTEX_COLORS
// 落地：out.vs/ps = hash("ASSETS/Shaders/HLSL/" + 拼好的路径)
```

宏开关触发条件（Full）：

| 宏 | 条件 |
|------|------|
| `SOFT_PARTICLES` | `ctx.softParticleAllowed && ctx.detailLevel>=2 && desc.softParticle!=NULL` |
| `MASKED` | `gap24 && (miscFlags & 0x4000)` |
| `ALPHA_EROSION` | `gap24 && desc.alphaErosion!=NULL` |
| `PALETTIZE_TEXTURES` | `gap24 && palette!=NULL && paletteLayers>0` |
| `REFLECTIVE` | `geom==quad && gap24 && texReflection!=NULL` |
| `MULT_PASS` | 多 pass 场景（secondary 复用） |
| `DISABLE_FOW` | `(flagsDisableZ&4) || zoneFlag[176]==1` |
| `USE_VERTEX_COLORS` | pass 资源 +125 字节非零 |
| `SCREEN/SEPARATE/LOCAL_SPACE_UV` | 由 `desc.gap200[0]`(uv 模式 1/2/3..5) |

## 纹理 + 采样器数据填充（BindTextureSlot）

```rust
fn bind_texture_slot(fx, slot_cache, tex_src, slot, ctx) -> bool {
    let desc = fx.descriptor();
    // (a) 采样器状态：多数槽过滤取 desc.filter_mode(+201)，映射 0→point,1→(2,2),2→(1,1),3→(3,3)
    let mut ss = SamplerState::from_filter_mode(desc.filter_mode);
    ss.mip = (desc.mip_flag == 0);                       // *(desc+18)==0 → 开 mip
    match slot {
        1 | 10 => ss.set_raw(0x0101, 1),                 // 强制线性-clamp
        4 | 5  => return slot_cache.is_some(),            // 引擎托管，不自建
        7      => ss = SamplerState::from_filter_mode(get_secondary_texture_info(fx).filter),
        8      => { ss = SamplerState::from_filter_mode(desc.palette.read_u8(16)); ss.mip = false; }
        9      => { ss = SamplerState::from_filter_mode(desc.alpha_erosion.read_u8(34)); ss.set_mip_raw(0x0101); }
        _      => {}
    }
    // (b) 解析/加载纹理到槽缓存
    resolve_texture(ctx, slot_cache, tex_src, ss);       // sub_1412CDF80
    let name = hash(off_141B26570[slot]);
    // (c) 缺纹理回退到全局默认（与 PIXEL_COLOR_REMAP_RAMP 同源）
    let tex = if let Some(t)=*slot_cache { t.add_ref(); t } else {
        let reg = g_render_texture_registry();
        if tex_src.read_u32(8)!=0 || slot==0 { reg.read_ptr(192) /*1x1 透明黑*/ } else { reg.read_ptr(184) }
    };
    // (d) 写入 shader：采样器状态 + 纹理句柄 + 常量名
    bind_resource_to_shader(fx.shader, name, ss, tex);   // sub_1412FFD10
    slot_cache.is_some()
}
```

## 逐-pass 渲染态（apply_pass_render_state，主函数内联）

因为混合/深度/剔除/模板都来自描述符字段，所以逐 pass 按下表组装 13 字节渲染态块后调 `ShaderPass_UpdateRenderState`：

| 渲染态 | 来源字段 |
|------|------|
| `ALPHA_TEST` + `AlphaTestReferenceValue` | `alphaRef`(+119)!=0 → 值=alphaRef/255 |
| 混合模式 | `blendMode`(+117)：1=alpha,2/3=加法族,4=预乘 |
| 模板 ref/mask | `blendMode!=0` 时取 `stencilRef`/`stencilWriteMask` |
| 深度偏移 | `depthBias0/1`(+124/+128) != 哨兵(qword_141F76628) |
| 剔除 | `backFaceOn`(+17) 或无 pass 槽且非特定 renderType → 关背面 |
| ZWrite | renderType 11/17 或有 pass 槽 → 按 `forceAnimMeshZWrite`(+115)&1 |
| shadow pass(a3==1) | 固定态，跳过混合/深度分支 |
