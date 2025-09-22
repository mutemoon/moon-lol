#[derive(Serialize, Deserialize, Debug, Clone, Component)]
#[serde(rename_all = "camelCase")]
pub struct VfxEmitterDefinitionData {
    pub spawn_shape: Option<VfxEmitterDefinitionDataSpawnShape>,
    pub acceleration: Option<ValueVector3>,
    pub alpha_erosion_definition: Option<VfxAlphaErosionDefinitionData>,
    pub alpha_ref: Option<u8>,
    pub audio: Option<VfxEmitterAudio>,
    pub bind_weight: Option<ValueFloat>,
    pub birth_acceleration: Option<ValueVector3>,
    pub birth_color: Option<ValueColor>,
    pub birth_drag: Option<ValueVector3>,
    pub birth_frame_rate: Option<ValueFloat>,
    pub birth_orbital_velocity: Option<ValueVector3>,
    pub birth_rotation0: Option<ValueVector3>,
    pub birth_rotational_acceleration: Option<ValueVector3>,
    pub birth_rotational_velocity0: Option<ValueVector3>,
    pub birth_scale0: Option<ValueVector3>,
    pub birth_uv_offset: Option<ValueVector2>,
    pub birth_uv_rotate_rate: Option<ValueFloat>,
    pub birth_uv_scroll_rate: Option<ValueVector2>,
    pub birth_velocity: Option<ValueVector3>,
    pub blend_mode: Option<u8>,
    pub censor_modulate_value: Option<Vec4>,
    pub chance_to_not_exist: Option<f32>,
    pub child_particle_set_definition: Option<VfxChildParticleSetDefinitionData>,
    pub color_look_up_offsets: Option<Vec2>,
    pub color_look_up_scales: Option<Vec2>,
    pub color_look_up_type_x: Option<u8>,
    pub color_look_up_type_y: Option<u8>,
    pub color_render_flags: Option<u8>,
    pub color: Option<ValueColor>,
    pub colorblind_visibility: Option<u8>,
    pub custom_material: Option<VfxMaterialDefinitionData>,
    pub depth_bias_factors: Option<Vec2>,
    pub direction_velocity_min_scale: Option<f32>,
    pub direction_velocity_scale: Option<f32>,
    pub disable_backface_cull: Option<bool>,
    pub disabled: Option<bool>,
    pub distortion_definition: Option<VfxDistortionDefinitionData>,
    pub does_cast_shadow: Option<bool>,
    pub does_lifetime_scale: Option<bool>,
    pub drag: Option<ValueVector3>,
    pub emission_mesh_name: Option<String>,
    pub emission_mesh_scale: Option<f32>,
    pub emission_surface_definition: Option<VfxEmissionSurfaceData>,
    pub emitter_linger: Option<f32>,
    pub emitter_name: Option<String>,
    pub emitter_position: Option<ValueVector3>,
    pub emitter_uv_scroll_rate: Option<Vec2>,
    pub falloff_texture: Option<String>,
    pub field_collection_definition: Option<VfxFieldCollectionDefinitionData>,
    pub filtering: Option<VfxEmitterFiltering>,
    pub flex_birth_rotational_velocity0: Option<FlexValueVector3>,
    pub flex_birth_uv_offset: Option<FlexValueVector2>,
    pub flex_birth_uv_scroll_rate: Option<FlexValueVector2>,
    pub flex_birth_velocity: Option<FlexValueVector3>,
    pub flex_instance_scale: Option<FlexTypeFloat>,
    pub flex_particle_lifetime: Option<FlexValueFloat>,
    pub flex_rate: Option<FlexValueFloat>,
    pub flex_scale_birth_scale: Option<FlexTypeFloat>,
    pub flex_shape_definition: Option<VfxFlexShapeDefinitionData>,
    pub frame_rate: Option<f32>,
    pub has_post_rotate_orientation: Option<bool>,
    pub has_variable_start_time: Option<bool>,
    pub importance: Option<u8>,
    pub is_direction_oriented: Option<bool>,
    pub is_emitter_space: Option<bool>,
    pub is_following_terrain: Option<bool>,
    pub is_ground_layer: Option<bool>,
    pub is_local_orientation: Option<bool>,
    pub is_random_start_frame: Option<bool>,
    pub is_rotation_enabled: Option<bool>,
    pub is_single_particle: Option<bool>,
    pub is_texture_pixelated: Option<bool>,
    pub is_uniform_scale: Option<bool>,
    pub legacy_simple: Option<VfxEmitterLegacySimple>,
    pub lifetime: Option<f32>,
    pub linger: Option<VfxLingerDefinitionData>,
    pub material_override_definitions: Option<Vec<VfxMaterialOverrideDefinitionData>>,
    pub maximum_rate_by_velocity: Option<f32>,
    pub mesh_render_flags: Option<u8>,
    pub misc_render_flags: Option<u8>,
    pub modulation_factor: Option<Vec4>,
    pub num_frames: Option<u16>,
    pub offset_life_scaling_symmetry_mode: Option<u8>,
    pub offset_lifetime_scaling: Option<Vec3>,
    pub palette_definition: Option<VfxPaletteDefinitionData>,
    pub particle_color_texture: Option<String>,
    pub particle_is_local_orientation: Option<bool>,
    pub particle_lifetime: Option<ValueFloat>,
    pub particle_linger_type: Option<u8>,
    pub particle_linger: Option<f32>,
    pub particle_uv_rotate_rate: Option<IntegratedValueFloat>,
    pub particle_uv_scroll_rate: Option<IntegratedValueVector2>,
    pub particles_share_random_value: Option<bool>,
    pub pass: Option<i16>,
    pub period: Option<f32>,
    pub post_rotate_orientation_axis: Option<Vec3>,
    pub primitive: Option<VfxEmitterDefinitionDataPrimitive>,
    pub rate_by_velocity_function: Option<ValueVector2>,
    pub rate: Option<ValueFloat>,
    pub reflection_definition: Option<VfxReflectionDefinitionData>,
    pub render_phase_override: Option<u8>,
    pub rotation_override: Option<Vec3>,
    pub rotation0: Option<IntegratedValueVector3>,
    pub scale_override: Option<Vec3>,
    pub scale0: Option<ValueVector3>,
    pub slice_technique_range: Option<f32>,
    pub soft_particle_params: Option<VfxSoftParticleDefinitionData>,
    pub sort_emitters_by_pos: Option<bool>,
    pub start_frame: Option<u16>,
    pub stencil_mode: Option<u8>,
    pub stencil_ref: Option<u8>,
    pub stencil_reference_id: Option<u32>,
    pub tex_address_mode_base: Option<u8>,
    pub tex_div: Option<Vec2>,
    pub texture_flip_u: Option<bool>,
    pub texture_flip_v: Option<bool>,
    pub texture_mult: Option<VfxTextureMultDefinitionData>,
    pub texture: Option<String>,
    pub time_active_during_period: Option<f32>,
    pub time_before_first_emission: Option<f32>,
    pub translation_override: Option<Vec3>,
    pub unk_0xcb13aff1: Option<f32>,
    pub unk_0xd1ee8634: Option<bool>,
    pub use_emission_mesh_normal_for_birth: Option<bool>,
    pub use_navmesh_mask: Option<bool>,
    pub uv_mode: Option<u8>,
    pub uv_parallax_scale: Option<f32>,
    pub uv_rotation: Option<ValueFloat>,
    pub uv_scale: Option<ValueVector2>,
    pub uv_scroll_clamp: Option<bool>,
    pub uv_transform_center: Option<Vec2>,
    pub velocity: Option<ValueVector3>,
    pub world_acceleration: Option<IntegratedValueVector3>,
    pub write_alpha_only: Option<bool>,
}

VfxEmitterDefinitionData 是从游戏中解包的配置文件的定义

#version 450

layout(set = 2, binding = 2) uniform texture2D TEXTURE_texture;
layout(set = 2, binding = 3) uniform sampler TEXTURE_sampler;
layout(set = 2, binding = 4) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 2, binding = 5) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 2, binding = 6) uniform texture2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 7) uniform sampler CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler;
layout(set = 2, binding = 8) uniform texture2D CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 9) uniform sampler CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler;

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec4 TEXCOORD1;
layout(location = 2) in vec2 TEXCOORD2;
layout(location = 0) out vec4 SV_Target0;

void main()
{
    vec4 _43 = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);
    vec4 _47 = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), TEXCOORD1.zw);
    vec4 _49 = TEXCOORD0 * (_43 * _47);
    vec3 _50 = _49.xyz;
    vec4 _56 = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(dot(_50, vec3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875)), 0.5));
    vec3 _63 = vec3(0.0);
    if (_56.w > 0.0)
    {
        _63 = _56.xyz;
    }
    else
    {
        _63 = _50;
    }
    vec4 _68 = vec4(_63, _49.w);
    vec3 _74 = _68.xyz * texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2).w;
    SV_Target0 = vec4(_74.x, _74.y, _74.z, _68.w);
}


疑惑1: PARTICLE_COLOR_TEXTURE 在 VfxEmitterDefinitionData 中有一个属性定义 particle_color_texture，而 PIXEL_COLOR_REMAP_RAMP 在 VfxEmitterDefinitionData 却没有

> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/distortion_mesh_ps.ps.glsl
  - 输出目录: assets/shaders/ps/distortion_mesh
  - 找到 3 个基础宏定义: ["MASKED", "COLORPALETTE_COLORBLIND", "ALPHA_TEST"]
  - 生成了 8 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/distortion_mesh_vs.vs.glsl
  - 输出目录: assets/shaders/vs/distortion_mesh
  - 找到 1 个基础宏定义: ["MASKED"]
  - 生成了 2 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/distortion_ps.ps.glsl
  - 输出目录: assets/shaders/ps/distortion
  - 找到 2 个基础宏定义: ["COLORPALETTE_COLORBLIND", "ALPHA_TEST"]
  - 生成了 4 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/distortion_vs.vs.glsl
  - 输出目录: assets/shaders/vs/distortion
  - 找到 0 个基础宏定义: []
  - 生成了 1 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/mesh_ps_slice.ps.glsl
  - 输出目录: assets/shaders/ps/mesh_slice
  - 找到 7 个基础宏定义: ["SEPARATE_ALPHA_UV", "DISABLE_FOW", "REFLECTIVE", "MASKED", "COLORPALETTE_COLORBLIND", "ALPHA_TEST", "MULT_PASS"]
  - 生成了 128 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/mesh_ps.ps.glsl
  - 输出目录: assets/shaders/ps/mesh
  - 找到 12 个基础宏定义: ["SEPARATE_ALPHA_UV", "DISABLE_FOW", "REFLECTIVE", "MASKED", "SCREEN_SPACE_UV", "FOW_IGNORE_VISIBILITY", "COLORPALETTE_COLORBLIND", "SOFT_PARTICLES", "PALETTIZE_TEXTURES", "ALPHA_EROSION", "ALPHA_TEST", "MULT_PASS"]
  - 生成了 4096 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/mesh_vs.vs.glsl
  - 输出目录: assets/shaders/vs/mesh
  - 找到 8 个基础宏定义: ["LOCAL_SPACE_UV", "SEPARATE_ALPHA_UV", "DISABLE_FOW", "USE_VERTEX_COLORS", "REFLECTIVE", "MASKED", "SCREEN_SPACE_UV", "MULT_PASS"]
  - 生成了 256 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_ps_fixedalphauv.ps.glsl
  - 输出目录: assets/shaders/ps/quad_fixedalphauv
  - 找到 7 个基础宏定义: ["DISABLE_FOW", "MASKED", "FOW_IGNORE_VISIBILITY", "COLORPALETTE_COLORBLIND", "PALETTIZE_TEXTURES", "ALPHA_TEST", "MULT_PASS"]
  - 生成了 128 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_ps_slice.ps.glsl
  - 输出目录: assets/shaders/ps/quad_slice
  - 找到 4 个基础宏定义: ["DISABLE_FOW", "MASKED", "COLORPALETTE_COLORBLIND", "ALPHA_TEST"]
  - 生成了 16 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_ps.ps.glsl
  - 输出目录: assets/shaders/ps/quad
  - 找到 9 个基础宏定义: ["DISABLE_FOW", "MASKED", "FOW_IGNORE_VISIBILITY", "COLORPALETTE_COLORBLIND", "SOFT_PARTICLES", "PALETTIZE_TEXTURES", "ALPHA_EROSION", "ALPHA_TEST", "MULT_PASS"]
  - 生成了 512 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_screenspaceuv.ps.glsl
  - 输出目录: assets/shaders/ps/quad_screenspaceuv
  - 找到 5 个基础宏定义: ["DISABLE_FOW", "FOW_IGNORE_VISIBILITY", "COLORPALETTE_COLORBLIND", "PALETTIZE_TEXTURES", "ALPHA_TEST"]
  - 生成了 32 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_screenspaceuv.vs.glsl
  - 输出目录: assets/shaders/vs/quad_screenspaceuv
  - 找到 1 个基础宏定义: ["DISABLE_FOW"]
  - 生成了 2 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_vs_fixedalphauv.vs.glsl
  - 输出目录: assets/shaders/vs/quad_fixedalphauv
  - 找到 3 个基础宏定义: ["DISABLE_FOW", "MASKED", "MULT_PASS"]
  - 生成了 8 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/quad_vs.vs.glsl
  - 输出目录: assets/shaders/vs/quad
  - 找到 4 个基础宏定义: ["DISABLE_FOW", "MASKED", "ALPHA_EROSION", "MULT_PASS"]
  - 生成了 16 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/shadow_mesh_ps.ps.glsl
  - 输出目录: assets/shaders/ps/shadow_mesh
  - 找到 2 个基础宏定义: ["HARDWARE_PCF", "ALPHA_EROSION"]
  - 生成了 4 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/shadow_mesh_vs.vs.glsl
  - 输出目录: assets/shaders/vs/shadow_mesh
  - 找到 0 个基础宏定义: []
  - 生成了 1 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/shadow_quad_ps.ps.glsl
  - 输出目录: assets/shaders/ps/shadow_quad
  - 找到 2 个基础宏定义: ["HARDWARE_PCF", "ALPHA_EROSION"]
  - 生成了 4 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/shadow_quad_vs.vs.glsl
  - 输出目录: assets/shaders/vs/shadow_quad
  - 找到 2 个基础宏定义: ["HARDWARE_PCF", "ALPHA_EROSION"]
  - 生成了 4 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/simple_projected_ps.ps.glsl
  - 输出目录: assets/shaders/ps/simple_projected
  - 找到 4 个基础宏定义: ["DISABLE_FOW", "FOW_IGNORE_VISIBILITY", "COLORPALETTE_COLORBLIND", "ALPHA_TEST"]
  - 生成了 16 种组合
> 处理文件: assets/ASSETS/shaders/hlsl/particlesystem/simple_projected_vs.vs.glsl
  - 输出目录: assets/shaders/vs/simple_projected
  - 找到 1 个基础宏定义: ["DISABLE_FOW"]
  - 生成了 2 种组合

疑惑2: 这里有所有粒子系统的 shader 的宏定义，可以看到并没有将 PARTICLE_COLOR_TEXTURE PIXEL_COLOR_REMAP_RAMP 作为宏定义，而是必须传的值，而 particle_color_texture 属性有时候是空值，那会传什么 sampler 给 gpu 呢？

                     sub_1017ced50:
00000001017ced50         push       rbp                                         ; CODE XREF=sub_1017ca550+1274, sub_1017ca550+1342
00000001017ced51         mov        rbp, rsp
00000001017ced54         push       r15
00000001017ced56         push       r14
00000001017ced58         push       r13
00000001017ced5a         push       r12
00000001017ced5c         push       rbx
00000001017ced5d         push       rax
00000001017ced5e         mov        rbx, r9
00000001017ced61         mov        r15, r8
00000001017ced64         mov        r12d, ecx
00000001017ced67         mov        r14, rdx
00000001017ced6a         mov        r13d, esi
00000001017ced6d         mov        eax, edi
00000001017ced6f         and        al, 0xfe
00000001017ced71         test       esi, esi
00000001017ced73         je         loc_1017cedb2

00000001017ced75         cmp        r13d, 0x2
00000001017ced79         je         loc_1017ceddb

00000001017ced7b         cmp        r13d, 0x1
00000001017ced7f         jne        loc_1017cee6e

00000001017ced85         test       r12b, r12b
00000001017ced88         je         loc_1017cee22

00000001017ced8e         cmp        dil, 0x1
00000001017ced92         lea        rcx, qword [aParticlesystem_1021c8c18]      ; "ParticleSystem/SHADOW_MESH_VS.vs"
00000001017ced99         lea        rdx, qword [aParticlesystem_1021c8c39]      ; "ParticleSystem/MESH_VS.vs"
00000001017ceda0         cmove      rdx, rcx
00000001017ceda4         cmp        al, 0x2
00000001017ceda6         lea        rsi, qword [aParticlesystem_1021c8bf3]      ; "ParticleSystem/DISTORTION_MESH_VS.vs"
00000001017cedad         jmp        loc_1017cee62

                     loc_1017cedb2:
00000001017cedb2         test       r12b, r12b                                  ; CODE XREF=sub_1017ced50+35
00000001017cedb5         je         loc_1017cee01

00000001017cedb7         cmp        dil, 0x1
00000001017cedbb         lea        rcx, qword [aParticlesystem_1021c8b5d]      ; "ParticleSystem/SHADOW_QUAD_VS.vs"
00000001017cedc2         lea        rdx, qword [aParticlesystem_1021c8b7e]      ; "ParticleSystem/QUAD_VS.vs"
00000001017cedc9         cmove      rdx, rcx
00000001017cedcd         cmp        al, 0x2
00000001017cedcf         lea        rsi, qword [aParticlesystem]                ; "ParticleSystem/DISTORTION_VS.vs"
00000001017cedd6         jmp        loc_1017cee62

                     loc_1017ceddb:
00000001017ceddb         test       r12b, r12b                                  ; CODE XREF=sub_1017ced50+41
00000001017cedde         je         loc_1017cee43

00000001017cede0         cmp        dil, 0x1
00000001017cede4         lea        rcx, qword [aSkinnedmeshpar_1021c8cd9]      ; "SkinnedMesh/PARTICLE_SHADOW_VS.vs"
00000001017cedeb         lea        rdx, qword [aSkinnedmeshpar_1021c8cfb]      ; "SkinnedMesh/PARTICLE_VS.vs"
00000001017cedf2         cmove      rdx, rcx
00000001017cedf6         cmp        al, 0x2
00000001017cedf8         lea        rsi, qword [aSkinnedmeshpar]                ; "SkinnedMesh/PARTICLE_DISTORTION_VS.vs"
00000001017cedff         jmp        loc_1017cee62

                     loc_1017cee01:
00000001017cee01         cmp        dil, 0x1                                    ; CODE XREF=sub_1017ced50+101
00000001017cee05         lea        rcx, qword [aParticlesystem_1021c8bb8]      ; "ParticleSystem/SHADOW_QUAD_PS.ps"
00000001017cee0c         lea        rdx, qword [aParticlesystem_1021c8bd9]      ; "ParticleSystem/QUAD_PS.ps"
00000001017cee13         cmove      rdx, rcx
00000001017cee17         cmp        al, 0x2
00000001017cee19         lea        rsi, qword [aParticlesystem_1021c8b98]      ; "ParticleSystem/DISTORTION_PS.ps"
00000001017cee20         jmp        loc_1017cee62

                     loc_1017cee22:
00000001017cee22         cmp        dil, 0x1                                    ; CODE XREF=sub_1017ced50+56
00000001017cee26         lea        rcx, qword [aParticlesystem_1021c8c78]      ; "ParticleSystem/SHADOW_MESH_PS.ps"
00000001017cee2d         lea        rdx, qword [aParticlesystem_1021c8c99]      ; "ParticleSystem/MESH_PS.ps"
00000001017cee34         cmove      rdx, rcx
00000001017cee38         cmp        al, 0x2
00000001017cee3a         lea        rsi, qword [aParticlesystem_1021c8c53]      ; "ParticleSystem/DISTORTION_MESH_PS.ps"
00000001017cee41         jmp        loc_1017cee62

                     loc_1017cee43:
00000001017cee43         cmp        dil, 0x1                                    ; CODE XREF=sub_1017ced50+142
00000001017cee47         lea        rcx, qword [aSkinnedmeshpar_1021c8d3c]      ; "SkinnedMesh/PARTICLE_SHADOW_PS.ps"
00000001017cee4e         lea        rdx, qword [aSkinnedmeshpar_1021c8d5e]      ; "SkinnedMesh/PARTICLE_PS.ps"
00000001017cee55         cmove      rdx, rcx
00000001017cee59         cmp        al, 0x2
00000001017cee5b         lea        rsi, qword [aSkinnedmeshpar_1021c8d16]      ; "SkinnedMesh/PARTICLE_DISTORTION_PS.ps"

                     loc_1017cee62:
00000001017cee62         cmovne     rsi, rdx                                    ; argument #2 for method sub_1015e7280, CODE XREF=sub_1017ced50+93, sub_1017ced50+134, sub_1017ced50+175, sub_1017ced50+208, sub_1017ced50+241
00000001017cee66         mov        rdi, r15                                    ; argument #1 for method sub_1015e7280
00000001017cee69         call       sub_1015e7280                               ; sub_1015e7280

                     loc_1017cee6e:
00000001017cee6e         movzx      eax, byte [r14]                             ; CODE XREF=sub_1017ced50+47
00000001017cee72         cmp        eax, 0x2
00000001017cee75         je         loc_1017cee99

00000001017cee77         cmp        eax, 0x1
00000001017cee7a         jne        loc_1017ceee8

00000001017cee7c         test       r13d, r13d
00000001017cee7f         je         loc_1017ceec2

00000001017cee81         lea        rsi, qword [aScreenspaceuv]                 ; argument #2 for method sub_10193b820, "SCREEN_SPACE_UV"
00000001017cee88         lea        rdx, qword [a1]                             ; argument #3 for method sub_10193b820, "1"
00000001017cee8f         mov        rdi, rbx                                    ; argument #1 for method sub_10193b820
00000001017cee92         call       sub_10193b820                               ; sub_10193b820
00000001017cee97         jmp        loc_1017ceee8

                     loc_1017cee99:
00000001017cee99         lea        rsi, qword [aSeparatealphau]                ; argument #2 for method sub_10193b820, "SEPARATE_ALPHA_UV", CODE XREF=sub_1017ced50+293
00000001017ceea0         lea        rdx, qword [a1]                             ; argument #3 for method sub_10193b820, "1"
00000001017ceea7         mov        rdi, rbx                                    ; argument #1 for method sub_10193b820
00000001017ceeaa         call       sub_10193b820                               ; sub_10193b820
00000001017ceeaf         test       r13d, r13d
00000001017ceeb2         jne        loc_1017ceee8

00000001017ceeb4         test       r12b, r12b
00000001017ceeb7         je         loc_1017ceed0

00000001017ceeb9         lea        rsi, qword [aParticlesystem_1021c8de5]      ; "ParticleSystem/QUAD_VS_FixedAlphaUV.vs"
00000001017ceec0         jmp        loc_1017ceee0

                     loc_1017ceec2:
00000001017ceec2         test       r12b, r12b                                  ; CODE XREF=sub_1017ced50+303
00000001017ceec5         je         loc_1017ceed9

00000001017ceec7         lea        rsi, qword [aParticlesystem_1021c8d79]      ; "ParticleSystem/QUAD_ScreenSpaceUV.vs"
00000001017ceece         jmp        loc_1017ceee0

                     loc_1017ceed0:
00000001017ceed0         lea        rsi, qword [aParticlesystem_1021c8e0c]      ; "ParticleSystem/QUAD_PS_FixedAlphaUV.ps", CODE XREF=sub_1017ced50+359
00000001017ceed7         jmp        loc_1017ceee0

                     loc_1017ceed9:
00000001017ceed9         lea        rsi, qword [aParticlesystem_1021c8d9e]      ; argument #2 for method sub_1015e7280, "ParticleSystem/QUAD_ScreenSpaceUV.ps", CODE XREF=sub_1017ced50+373

                     loc_1017ceee0:
00000001017ceee0         mov        rdi, r15                                    ; argument #1 for method sub_1015e7280, CODE XREF=sub_1017ced50+368, sub_1017ced50+382, sub_1017ced50+391
00000001017ceee3         call       sub_1015e7280                               ; sub_1015e7280

                     loc_1017ceee8:
00000001017ceee8         cmp        byte [r14+2], 0x0                           ; CODE XREF=sub_1017ced50+298, sub_1017ced50+327, sub_1017ced50+354
00000001017ceeed         je         loc_1017cef0a

00000001017ceeef         lea        rsi, qword [aReflective]                    ; argument #2 for method sub_10193b820, "REFLECTIVE"
00000001017ceef6         lea        rdx, qword [a1]                             ; argument #3 for method sub_10193b820, "1"
00000001017ceefd         mov        rdi, rbx                                    ; argument #1 for method sub_10193b820
00000001017cef00         call       sub_10193b820                               ; sub_10193b820
00000001017cef05         test       r13d, r13d
00000001017cef08         je         loc_1017cef35

                     loc_1017cef0a:
00000001017cef0a         cmp        byte [r14+1], 0x0                           ; CODE XREF=sub_1017ced50+413
00000001017cef0f         je         loc_1017cef59

                     loc_1017cef11:
00000001017cef11         lea        rsi, qword [aMultpass]                      ; argument #2 for method sub_10193b820, "MULT_PASS", CODE XREF=sub_1017ced50+519
00000001017cef18         lea        rdx, qword [a1]                             ; argument #3 for method sub_10193b820, "1"
00000001017cef1f         mov        rdi, rbx                                    ; argument #1 for method sub_10193b820
00000001017cef22         add        rsp, 0x8
00000001017cef26         pop        rbx
00000001017cef27         pop        r12
00000001017cef29         pop        r13
00000001017cef2b         pop        r14
00000001017cef2d         pop        r15
00000001017cef2f         pop        rbp
00000001017cef30         jmp        sub_10193b820                               ; sub_10193b820
                        ; endp

                     loc_1017cef35:
00000001017cef35         lea        rax, qword [aParticlesystem_1021c8c39]      ; "ParticleSystem/MESH_VS.vs", CODE XREF=sub_1017ced50+440
00000001017cef3c         lea        rsi, qword [aParticlesystem_1021c8c99]      ; "ParticleSystem/MESH_PS.ps"
00000001017cef43         test       r12b, r12b
00000001017cef46         cmovne     rsi, rax                                    ; argument #2 for method sub_1015e7280
00000001017cef4a         mov        rdi, r15                                    ; argument #1 for method sub_1015e7280
00000001017cef4d         call       sub_1015e7280                               ; sub_1015e7280
00000001017cef52         cmp        byte [r14+1], 0x0
00000001017cef57         jne        loc_1017cef11

                     loc_1017cef59:
00000001017cef59         add        rsp, 0x8                                    ; CODE XREF=sub_1017ced50+447
00000001017cef5d         pop        rbx
00000001017cef5e         pop        r12
00000001017cef60         pop        r13
00000001017cef62         pop        r14
00000001017cef64         pop        r15
00000001017cef66         pop        rbp
00000001017cef67         ret
                        ; endp
00000001017cef68         align      16

从反汇编结果来看，这里有选择宏定义的逻辑，可能需要继续追溯从哪里获取 opengl 的 PARTICLE_COLOR_TEXTURE PIXEL_COLOR_REMAP_RAMP 的 sampler

```
        ; ================ B E G I N N I N G   O F   P R O C E D U R E ================

        ; Variables:
        ;    var_30: int64_t, -48
        ;    var_138: int8_t, -312
        ;    var_140: int64_t, -320
        ;    var_148: int64_t, -328
        ;    var_158: int64_t, -344
        ;    var_15C: int32_t, -348
        ;    var_160: int8_t, -352
        ;    var_161: int8_t, -353
        ;    var_162: int8_t, -354
        ;    var_164: int32_t, -356
        ;    var_166: int32_t, -358
        ;    var_16B: int8_t, -363
        ;    var_16C: int64_t, -364
        ;    var_170: int64_t, -368
        ;    var_178: int64_t, -376
        ;    var_180: int64_t, -384
        ;    var_188: int64_t, -392
        ;    var_18C: int32_t, -396
        ;    var_190: int32_t, -400
        ;    var_194: int64_t, -404
        ;    var_198: int64_t, -408
        ;    var_19C: int32_t, -412
        ;    var_1A4: int64_t, -420
        ;    var_1A8: int32_t, -424
        ;    var_1AC: int32_t, -428
        ;    var_1B4: int64_t, -436
        ;    var_1B8: int32_t, -440
        ;    var_1D0: int8_t, -464
        ;    var_1E0: -480
        ;    var_1F0: int8_t, -496
        ;    var_200: int64_t, -512


                     sub_1017c8b20:
00000001017c8b20         push       rbp
00000001017c8b21         mov        rbp, rsp
00000001017c8b24         push       r15
00000001017c8b26         push       r14
00000001017c8b28         push       r13
00000001017c8b2a         push       r12
00000001017c8b2c         push       rbx
00000001017c8b2d         sub        rsp, 0x1d8
00000001017c8b34         mov        qword [rbp+var_188], rcx
00000001017c8b3b         mov        qword [rbp+var_180], rdx
00000001017c8b42         mov        qword [rbp+var_178], rsi
00000001017c8b49         mov        rax, qword [___stack_chk_guard_10228c320]   ; ___stack_chk_guard_10228c320
00000001017c8b50         mov        rax, qword [rax]
00000001017c8b53         mov        qword [rbp+var_30], rax
00000001017c8b57         mov        rax, qword [rdi+0x10]
00000001017c8b5b         mov        rcx, qword [rax+0x60]
00000001017c8b5f         movzx      r14d, byte [rcx+8]
00000001017c8b64         cmp        r14b, 0xc
00000001017c8b68         jne        loc_1017c8b71

00000001017c8b6a         xor        ebx, ebx
00000001017c8b6c         jmp        loc_1017c98de

                     loc_1017c8b71:
00000001017c8b71         mov        r15d, r8d                                   ; CODE XREF=sub_1017c8b20+72
00000001017c8b74         mov        rbx, rdi
00000001017c8b77         mov        r13, qword [byte_10240fe18+8]               ; 0x10240fe20
00000001017c8b7e         mov        rsi, qword [rax+0x68]
00000001017c8b82         test       rsi, rsi
00000001017c8b85         je         loc_1017c8bf8

00000001017c8b87         lea        rdi, qword [rbp+var_148]                    ; Begin of try block (catch block at 0x1017c9a67), argument #1 for method sub_1017c9b20
00000001017c8b8e         mov        rdx, rbx                                    ; argument #3 for method sub_1017c9b20
00000001017c8b91         call       sub_1017c9b20                               ; sub_1017c9b20
00000001017c8b96         mov        rdi, qword [rbp+var_148]                    ; End of try block started at 0x1017c8b87
00000001017c8b9d         test       rdi, rdi
00000001017c8ba0         je         loc_1017c8ba7

00000001017c8ba2         call       sub_1018477c0                               ; sub_1018477c0, Begin of try block (catch block at 0x1017c9a35)

                     loc_1017c8ba7:
00000001017c8ba7         mov        rdi, qword [rbx+0x78]                       ; CODE XREF=sub_1017c8b20+128
00000001017c8bab         test       rdi, rdi
00000001017c8bae         je         loc_1017c8bb5

00000001017c8bb0         call       sub_1018477e0                               ; sub_1018477e0

                     loc_1017c8bb5:
00000001017c8bb5         mov        rdi, qword [rbp+var_148]                    ; End of try block started at 0x1017c8ba2, CODE XREF=sub_1017c8b20+142
00000001017c8bbc         mov        qword [rbx+0x78], rdi
00000001017c8bc0         test       rdi, rdi
00000001017c8bc3         je         loc_1017c8bf8

00000001017c8bc5         call       sub_1018477e0                               ; sub_1018477e0, Begin of try block (catch block at 0x1017c9a18)
00000001017c8bca         mov        r12, qword [rbx+0x78]                       ; End of try block started at 0x1017c8bc5
00000001017c8bce         test       r12, r12
00000001017c8bd1         je         loc_1017c8bf8

00000001017c8bd3         lea        rdi, qword [aApplyteamcolor]                ; Begin of try block (catch block at 0x1017c9a67), argument #1 for method sub_10193b4b0, "APPLY_TEAM_COLOR_CORRECTION"
00000001017c8bda         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c8bdf         mov        rdi, r12                                    ; argument #1 for method sub_101844100
00000001017c8be2         mov        rsi, rax                                    ; argument #2 for method sub_101844100
00000001017c8be5         call       sub_101844100                               ; sub_101844100
00000001017c8bea         test       rax, rax                                    ; End of try block started at 0x1017c8bd3
00000001017c8bed         setne      byte [rbx+0x19]
00000001017c8bf1         mov        bl, 0x1
00000001017c8bf3         jmp        loc_1017c98de

                     loc_1017c8bf8:
00000001017c8bf8         movabs     rax, 0x10000000000                          ; CODE XREF=sub_1017c8b20+101, sub_1017c8b20+163, sub_1017c8b20+177
00000001017c8c02         mov        qword [rbp+var_140], rax
00000001017c8c09         lea        rax, qword [rbp+var_138]
00000001017c8c10         mov        qword [rbp+var_148], rax
00000001017c8c17         mov        byte [rbp+var_138], 0x0
00000001017c8c1e         test       r15b, r15b
00000001017c8c21         je         loc_1017c8c42

00000001017c8c23         mov        rdi, qword [rbx+0x78]
00000001017c8c27         test       rdi, rdi
00000001017c8c2a         je         loc_1017c8c42

00000001017c8c2c         call       sub_10183fd60                               ; sub_10183fd60, Begin of try block (catch block at 0x1017c9a50)
00000001017c8c31         lea        rdi, qword [rbp+var_148]                    ; argument #1 for method sub_1015e7280
00000001017c8c38         mov        rsi, rax                                    ; argument #2 for method sub_1015e7280
00000001017c8c3b         call       sub_1015e7280                               ; sub_1015e7280
00000001017c8c40         jmp        loc_1017c8c69

                     loc_1017c8c42:
00000001017c8c42         mov        rax, qword [rbx+0x10]                       ; CODE XREF=sub_1017c8b20+257, sub_1017c8b20+266
00000001017c8c46         mov        rdx, qword [rax]
00000001017c8c49         mov        rax, qword [rbp+var_178]
00000001017c8c50         mov        rcx, qword [rax+0x40]
00000001017c8c54         lea        rsi, qword [aSs_1021cff18]                  ; "%s:%s"
00000001017c8c5b         lea        rdi, qword [rbp+var_148]
00000001017c8c62         xor        eax, eax
00000001017c8c64         call       sub_1015e5b8c+4

                     loc_1017c8c69:
00000001017c8c69         xorps      xmm0, xmm0                                  ; End of try block started at 0x1017c8c2c, CODE XREF=sub_1017c8b20+288
00000001017c8c6c         movaps     xmmword [rbp+var_1E0], xmm0
00000001017c8c73         movaps     xmmword [rbp+var_1F0], xmm0
00000001017c8c7a         mov        rdx, qword [rbp+var_148]                    ; argument #3 for method sub_101848f20
00000001017c8c81         movzx      ecx, r15b                                   ; Begin of try block (catch block at 0x1017c9a30), argument #4 for method sub_101848f20
00000001017c8c85         lea        rdi, qword [rbp+var_170]                    ; argument #1 for method sub_101848f20
00000001017c8c8c         lea        r8, qword [rbp+var_1F0]                     ; argument #5 for method sub_101848f20
00000001017c8c93         mov        rsi, r13                                    ; argument #2 for method sub_101848f20
00000001017c8c96         call       sub_101848f20                               ; sub_101848f20
00000001017c8c9b         mov        rdi, qword [rbp+var_170]                    ; End of try block started at 0x1017c8c81
00000001017c8ca2         test       rdi, rdi
00000001017c8ca5         je         loc_1017c8cac

00000001017c8ca7         call       sub_1018477c0                               ; sub_1018477c0, Begin of try block (catch block at 0x1017c9a4e)

                     loc_1017c8cac:
00000001017c8cac         mov        rdi, qword [rbx+0x78]                       ; CODE XREF=sub_1017c8b20+389
00000001017c8cb0         test       rdi, rdi
00000001017c8cb3         je         loc_1017c8cba

00000001017c8cb5         call       sub_1018477e0                               ; sub_1018477e0

                     loc_1017c8cba:
00000001017c8cba         mov        rdi, qword [rbp+var_170]                    ; End of try block started at 0x1017c8ca7, CODE XREF=sub_1017c8b20+403
00000001017c8cc1         mov        qword [rbx+0x78], rdi
00000001017c8cc5         test       rdi, rdi
00000001017c8cc8         je         loc_1017c8ce6

00000001017c8cca         call       sub_1018477e0                               ; sub_1018477e0, Begin of try block (catch block at 0x1017c9a1f)
00000001017c8ccf         mov        rdi, qword [rbx+0x78]                       ; End of try block started at 0x1017c8cca
00000001017c8cd3         test       r15b, r15b
00000001017c8cd6         jne        loc_1017c8d78

00000001017c8cdc         test       rdi, rdi
00000001017c8cdf         je         loc_1017c8cf2

00000001017c8ce1         jmp        loc_1017c8d78

                     loc_1017c8ce6:
00000001017c8ce6         test       r15b, r15b                                  ; CODE XREF=sub_1017c8b20+424
00000001017c8ce9         je         loc_1017c8cf2

00000001017c8ceb         xor        edi, edi
00000001017c8ced         jmp        loc_1017c8d78

                     loc_1017c8cf2:
00000001017c8cf2         lea        r15, qword [rbp+var_148]                    ; CODE XREF=sub_1017c8b20+447, sub_1017c8b20+457
00000001017c8cf9         lea        r12, qword [rbp+var_170]
00000001017c8d00         jmp        loc_1017c8d17
00000001017c8d02         align      16

                     loc_1017c8d10:
00000001017c8d10         xor        edi, edi                                    ; CODE XREF=sub_1017c8b20+584
00000001017c8d12         test       rdi, rdi
00000001017c8d15         jne        loc_1017c8d78

                     loc_1017c8d17:
00000001017c8d17         mov        rdi, r15                                    ; Begin of try block (catch block at 0x1017c9aa6), argument #1 for method sub_1015e7d70, CODE XREF=sub_1017c8b20+480, sub_1017c8b20+598
00000001017c8d1a         mov        esi, 0x2a                                   ; argument #2 for method sub_1015e7d70
00000001017c8d1f         call       sub_1015e7d70                               ; sub_1015e7d70
00000001017c8d24         mov        rdx, qword [rbp+var_148]                    ; End of try block started at 0x1017c8d17, argument #3 for method sub_101848f20
00000001017c8d2b         mov        rdi, r12                                    ; Begin of try block (catch block at 0x1017c9aa4), argument #1 for method sub_101848f20
00000001017c8d2e         mov        rsi, r13                                    ; argument #2 for method sub_101848f20
00000001017c8d31         xor        ecx, ecx                                    ; argument #4 for method sub_101848f20
00000001017c8d33         xor        r8d, r8d                                    ; argument #5 for method sub_101848f20
00000001017c8d36         call       sub_101848f20                               ; sub_101848f20
00000001017c8d3b         mov        rdi, qword [rbp+var_170]                    ; End of try block started at 0x1017c8d2b
00000001017c8d42         test       rdi, rdi
00000001017c8d45         je         loc_1017c8d4c

00000001017c8d47         call       sub_1018477c0                               ; sub_1018477c0, Begin of try block (catch block at 0x1017c9aa8)

                     loc_1017c8d4c:
00000001017c8d4c         mov        rdi, qword [rbx+0x78]                       ; CODE XREF=sub_1017c8b20+549
00000001017c8d50         test       rdi, rdi
00000001017c8d53         je         loc_1017c8d5a

00000001017c8d55         call       sub_1018477e0                               ; sub_1018477e0

                     loc_1017c8d5a:
00000001017c8d5a         mov        rdi, qword [rbp+var_170]                    ; End of try block started at 0x1017c8d47, CODE XREF=sub_1017c8b20+563
00000001017c8d61         mov        qword [rbx+0x78], rdi
00000001017c8d65         test       rdi, rdi
00000001017c8d68         je         loc_1017c8d10

00000001017c8d6a         call       sub_1018477e0                               ; sub_1018477e0, Begin of try block (catch block at 0x1017c9a89)
00000001017c8d6f         mov        rdi, qword [rbx+0x78]                       ; End of try block started at 0x1017c8d6a
00000001017c8d73         test       rdi, rdi
00000001017c8d76         je         loc_1017c8d17

                     loc_1017c8d78:
00000001017c8d78         mov        rax, qword [rbx+0x10]                       ; CODE XREF=sub_1017c8b20+438, sub_1017c8b20+449, sub_1017c8b20+461, sub_1017c8b20+501
00000001017c8d7c         movsd      xmm0, qword [rax+0x8c]
00000001017c8d84         xorps      xmm1, xmm1
00000001017c8d87         cmpneqps   xmm1, xmm0
00000001017c8d8b         movaps     xmm2, xmmword [aNst3112badweak+144]         ; 0x1020738f0
00000001017c8d92         divps      xmm2, xmm0
00000001017c8d95         andps      xmm2, xmm1
00000001017c8d98         movss      dword [rbp+var_1B8], xmm0
00000001017c8da0         movlps     qword [rbp+var_1B4], xmm2
00000001017c8da7         mov        dword [rbp+var_1AC], 0x0
00000001017c8db1         lea        rsi, qword [aTextureinfo]                   ; Begin of try block (catch block at 0x1017c9a2b), argument #2 for method sub_1018442d0, "TEXTURE_INFO"
00000001017c8db8         lea        rdx, qword [rbp+var_1B8]                    ; argument #3 for method sub_1018442d0
00000001017c8dbf         call       sub_1018442d0                               ; sub_1018442d0
00000001017c8dc4         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c8db1
00000001017c8dc8         mov        r13, qword [rax+0x230]
00000001017c8dcf         test       r13, r13
00000001017c8dd2         jne        loc_1017c8dea

00000001017c8dd4         movzx      eax, byte [byte_1024ab9f0]                  ; byte_1024ab9f0
00000001017c8ddb         lea        r13, qword [qword_1024ab948]                ; qword_1024ab948
00000001017c8de2         test       al, al
00000001017c8de4         je         loc_1017c9907

                     loc_1017c8dea:
00000001017c8dea         movsd      xmm0, qword [r13+0x10]                      ; CODE XREF=sub_1017c8b20+690, sub_1017c8b20+3573, sub_1017c8b20+3797
00000001017c8df0         xorps      xmm1, xmm1
00000001017c8df3         cmpneqps   xmm1, xmm0
00000001017c8df7         movaps     xmm2, xmmword [aNst3112badweak+144]         ; 0x1020738f0
00000001017c8dfe         divps      xmm2, xmm0
00000001017c8e01         andps      xmm2, xmm1
00000001017c8e04         movss      dword [rbp+var_1A8], xmm0
00000001017c8e0c         movlps     qword [rbp+var_1A4], xmm2
00000001017c8e13         mov        dword [rbp+var_19C], 0x0
00000001017c8e1d         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_1018442d0
00000001017c8e21         lea        rsi, qword [aTextureinfo2]                  ; Begin of try block (catch block at 0x1017c9aa0), argument #2 for method sub_1018442d0, "TEXTURE_INFO_2"
00000001017c8e28         lea        rdx, qword [rbp+var_1A8]                    ; argument #3 for method sub_1018442d0
00000001017c8e2f         call       sub_1018442d0                               ; sub_1018442d0
00000001017c8e34         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c8e38         lea        rsi, qword [aKcolorfactor]                  ; argument #2 for method sub_101844140, "kColorFactor"
00000001017c8e3f         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c8e44         call       sub_101844140                               ; sub_101844140
00000001017c8e49         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c8e4d         lea        rsi, qword [aApplyteamcolor]                ; argument #2 for method sub_101844140, "APPLY_TEAM_COLOR_CORRECTION"
00000001017c8e54         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c8e59         call       sub_101844140                               ; sub_101844140
00000001017c8e5e         cmp        byte [rbx+0x19], 0x0
00000001017c8e62         je         loc_1017c8eb6

00000001017c8e64         mov        rax, qword [rbx+0x10]
00000001017c8e68         test       byte [rax+0x74], 0x1
00000001017c8e6c         jne        loc_1017c8e91

00000001017c8e6e         xorps      xmm0, xmm0
00000001017c8e71         movaps     xmmword [rbp+var_170], xmm0
00000001017c8e78         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101845a60
00000001017c8e7c         mov        rsi, qword [qword_1024aab18]                ; argument #2 for method sub_101845a60, qword_1024aab18
00000001017c8e83         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_101845a60
00000001017c8e8a         call       sub_101845a60                               ; sub_101845a60
00000001017c8e8f         jmp        loc_1017c8eb6

                     loc_1017c8e91:
00000001017c8e91         movaps     xmm0, xmmword [aNst3112badweak+48]          ; 0x102073890, CODE XREF=sub_1017c8b20+844
00000001017c8e98         movaps     xmmword [rbp+var_170], xmm0
00000001017c8e9f         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101845a60
00000001017c8ea3         mov        rsi, qword [qword_1024aab18]                ; argument #2 for method sub_101845a60, qword_1024aab18
00000001017c8eaa         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_101845a60
00000001017c8eb1         call       sub_101845a60                               ; sub_101845a60

                     loc_1017c8eb6:
00000001017c8eb6         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140, CODE XREF=sub_1017c8b20+834, sub_1017c8b20+879
00000001017c8eba         lea        rsi, qword [aVparticleuvtra]                ; argument #2 for method sub_101844140, "vParticleUVTransformMult"
00000001017c8ec1         mov        edx, 0x3                                    ; argument #3 for method sub_101844140
00000001017c8ec6         call       sub_101844140                               ; sub_101844140
00000001017c8ecb         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c8ecf         lea        rsi, qword [aVparticleuvtra_1021c894c]      ; argument #2 for method sub_101844140, "vParticleUVTransform"
00000001017c8ed6         mov        edx, 0x3                                    ; argument #3 for method sub_101844140
00000001017c8edb         call       sub_101844140                               ; sub_101844140
00000001017c8ee0         mov        rdx, qword [rbx+0x10]                       ; End of try block started at 0x1017c8e21
00000001017c8ee4         mov        rax, qword [rdx+0x50]
00000001017c8ee8         test       rax, rax
00000001017c8eeb         je         loc_1017c8f5e

00000001017c8eed         movss      xmm0, dword [rax]
00000001017c8ef1         movss      dword [rbp+var_170], xmm0
00000001017c8ef9         xorps      xmm0, xmm0
00000001017c8efc         movlps     qword [rbp+var_16C], xmm0
00000001017c8f03         mov        dword [rbp+var_164], 0x0
00000001017c8f0d         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_1018442d0
00000001017c8f11         lea        rsi, qword [aDistortionpowe]                ; Begin of try block (catch block at 0x1017c9a4c), argument #2 for method sub_1018442d0, "DistortionPower"
00000001017c8f18         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_1018442d0
00000001017c8f1f         call       sub_1018442d0                               ; sub_1018442d0
00000001017c8f24         lea        rsi, qword [rbx+0x38]                       ; argument #2 for method sub_1017c9ff0
00000001017c8f28         mov        rax, qword [rbx+0x10]
00000001017c8f2c         mov        rdx, qword [rax+0x50]
00000001017c8f30         add        rdx, 0x8                                    ; argument #3 for method sub_1017c9ff0
00000001017c8f34         mov        rax, qword [rbp+var_178]
00000001017c8f3b         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c8f3f         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c8f42         mov        ecx, 0x3                                    ; argument #4 for method sub_1017c9ff0
00000001017c8f47         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c8f4e         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c8f55         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c8f5a         mov        rdx, qword [rbx+0x10]                       ; End of try block started at 0x1017c8f11

                     loc_1017c8f5e:
00000001017c8f5e         cmp        qword [rdx+0x40], 0x0                       ; CODE XREF=sub_1017c8b20+971
00000001017c8f63         je         loc_1017c8fe2

00000001017c8f65         lea        rdi, qword [rbp+var_170]                    ; Begin of try block (catch block at 0x1017c9a1a), argument #1 for method sub_101891f70
00000001017c8f6c         mov        esi, 0x6                                    ; argument #2 for method sub_101891f70
00000001017c8f71         call       sub_101891f70                               ; sub_101891f70
00000001017c8f76         lea        rsi, qword [rbx+0x58]                       ; End of try block started at 0x1017c8f65, argument #2 for method sub_1017c9ff0
00000001017c8f7a         mov        rax, qword [rbx+0x10]
00000001017c8f7e         mov        rdx, qword [rax+0x40]                       ; argument #3 for method sub_1017c9ff0
00000001017c8f82         mov        rax, qword [rbp+var_178]                    ; Begin of try block (catch block at 0x1017c9a72)
00000001017c8f89         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c8f8d         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c8f90         mov        ecx, 0x8                                    ; argument #4 for method sub_1017c9ff0
00000001017c8f95         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c8f9c         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c8fa3         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c8fa8         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c8fac         lea        rsi, qword [aCpaletteselect]                ; argument #2 for method sub_101844140, "cPaletteSelectMain"
00000001017c8fb3         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c8fb8         call       sub_101844140                               ; sub_101844140
00000001017c8fbd         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c8fc1         lea        rsi, qword [aCpalettesrcmix]                ; argument #2 for method sub_101844140, "cPaletteSrcMixerMain"
00000001017c8fc8         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c8fcd         call       sub_101844140                               ; sub_101844140
00000001017c8fd2         lea        rdi, qword [rbp+var_170]                    ; End of try block started at 0x1017c8f82, argument #1 for method sub_101891fb0
00000001017c8fd9         call       sub_101891fb0                               ; sub_101891fb0
00000001017c8fde         mov        rdx, qword [rbx+0x10]

                     loc_1017c8fe2:
00000001017c8fe2         mov        rax, qword [rdx+0x38]                       ; CODE XREF=sub_1017c8b20+1091
00000001017c8fe6         test       rax, rax
00000001017c8fe9         je         loc_1017c904b

00000001017c8feb         lea        rsi, qword [rbx+0x60]                       ; argument #2 for method sub_1017c9ff0
00000001017c8fef         add        rax, 0x30
00000001017c8ff3         mov        rcx, qword [rbx+8]
00000001017c8ff7         mov        rcx, qword [rcx+0x30]
00000001017c8ffb         mov        qword [rsp+0x200+var_200], rcx              ; Begin of try block (catch block at 0x1017c9aa0), argument #7 for method sub_1017c9ff0
00000001017c8fff         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c9002         mov        rdx, rax                                    ; argument #3 for method sub_1017c9ff0
00000001017c9005         mov        ecx, 0x9                                    ; argument #4 for method sub_1017c9ff0
00000001017c900a         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c9011         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c9018         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c901d         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c9021         lea        rsi, qword [aCalphaerosionp]                ; argument #2 for method sub_101844140, "cAlphaErosionParams"
00000001017c9028         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c902d         call       sub_101844140                               ; sub_101844140
00000001017c9032         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c9036         lea        rsi, qword [aCalphaerosiont]                ; argument #2 for method sub_101844140, "cAlphaErosionTextureMixer"
00000001017c903d         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c9042         call       sub_101844140                               ; sub_101844140
00000001017c9047         mov        rdx, qword [rbx+0x10]

                     loc_1017c904b:
00000001017c904b         lea        rsi, qword [rbx+0x30]                       ; argument #2 for method sub_1017c9ff0, CODE XREF=sub_1017c8b20+1225
00000001017c904f         add        rdx, 0xa8                                   ; argument #3 for method sub_1017c9ff0
00000001017c9056         mov        rax, qword [rbp+var_178]
00000001017c905d         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c9061         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c9064         mov        ecx, 0x1                                    ; argument #4 for method sub_1017c9ff0
00000001017c9069         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c9070         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c9077         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c907c         lea        rsi, qword [rbx+0x40]                       ; argument #2 for method sub_1017c9ff0
00000001017c9080         mov        edx, 0xb8
00000001017c9085         add        rdx, qword [rbx+0x10]                       ; argument #3 for method sub_1017c9ff0
00000001017c9089         mov        rax, qword [rbp+var_178]
00000001017c9090         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c9094         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c9097         mov        ecx, 0x2                                    ; argument #4 for method sub_1017c9ff0
00000001017c909c         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c90a3         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c90aa         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c90af         lea        rsi, qword [rbx+0x48]                       ; argument #2 for method sub_1017c9ff0
00000001017c90b3         mov        rax, qword [rbp+var_178]
00000001017c90ba         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c90be         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c90c1         mov        rdx, r13                                    ; argument #3 for method sub_1017c9ff0
00000001017c90c4         mov        ecx, 0x7                                    ; argument #4 for method sub_1017c9ff0
00000001017c90c9         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c90d0         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c90d7         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c90dc         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c8ffb
00000001017c90e0         movss      xmm0, dword [rax+0x84]
00000001017c90e8         xorps      xmm1, xmm1
00000001017c90eb         ucomiss    xmm0, xmm1
00000001017c90ee         jbe        loc_1017c919a

00000001017c90f4         movss      dword [rbp+var_170], xmm0
00000001017c90fc         xorps      xmm2, xmm2
00000001017c90ff         movlps     qword [rbp+var_16C], xmm2
00000001017c9106         mov        dword [rbp+var_164], 0x0
00000001017c9110         ucomiss    xmm0, xmm1
00000001017c9113         jne        loc_1017c9117

00000001017c9115         jnp        loc_1017c912f

                     loc_1017c9117:
00000001017c9117         mulss      xmm0, xmm0                                  ; CODE XREF=sub_1017c8b20+1523
00000001017c911b         movss      xmm1, dword [float_value_1E_minus_05+4]     ; 0x102073854
00000001017c9123         divss      xmm1, xmm0
00000001017c9127         movss      dword [rbp+var_16C], xmm1

                     loc_1017c912f:
00000001017c912f         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_1018442d0, CODE XREF=sub_1017c8b20+1525
00000001017c9133         lea        rsi, qword [aSlicerange]                    ; Begin of try block (catch block at 0x1017c9a13), argument #2 for method sub_1018442d0, "SLICE_RANGE"
00000001017c913a         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_1018442d0
00000001017c9141         call       sub_1018442d0                               ; sub_1018442d0
00000001017c9146         mov        qword [rbp+var_198], 0x0                    ; End of try block started at 0x1017c9133
00000001017c9151         mov        rax, qword [rbp+var_178]                    ; Begin of try block (catch block at 0x1017c99ff)
00000001017c9158         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c915c         lea        rdx, qword [qword_10240f950]                ; argument #3 for method sub_1017c9ff0, qword_10240f950
00000001017c9163         lea        rsi, qword [rbp+var_198]                    ; argument #2 for method sub_1017c9ff0
00000001017c916a         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c916d         mov        ecx, 0x6                                    ; argument #4 for method sub_1017c9ff0
00000001017c9172         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c9179         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c9180         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c9185         mov        rdi, qword [rbp+var_198]                    ; End of try block started at 0x1017c9151
00000001017c918c         test       rdi, rdi
00000001017c918f         je         loc_1017c9196

00000001017c9191         call       sub_1018605d0                               ; sub_1018605d0, Begin of try block (catch block at 0x1017c99fa)

                     loc_1017c9196:
00000001017c9196         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c9191, CODE XREF=sub_1017c8b20+1647

                     loc_1017c919a:
00000001017c919a         mov        rdx, qword [rax+0x48]                       ; CODE XREF=sub_1017c8b20+1486
00000001017c919e         test       rdx, rdx
00000001017c91a1         je         loc_1017c91f7

00000001017c91a3         lea        rsi, qword [rbx+0x50]                       ; argument #2 for method sub_1017c9ff0
00000001017c91a7         mov        rax, qword [rbp+var_178]                    ; Begin of try block (catch block at 0x1017c9aa0)
00000001017c91ae         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c91b2         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c91b5         mov        ecx, 0xa                                    ; argument #4 for method sub_1017c9ff0
00000001017c91ba         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c91c1         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c91c8         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c91cd         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c91d1         lea        rsi, qword [aVreflection]                   ; argument #2 for method sub_101844140, "vReflection"
00000001017c91d8         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c91dd         call       sub_101844140                               ; sub_101844140
00000001017c91e2         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c91e6         lea        rsi, qword [aVreflectionfco]                ; argument #2 for method sub_101844140, "vReflectionFColor"
00000001017c91ed         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c91f2         call       sub_101844140                               ; sub_101844140

                     loc_1017c91f7:
00000001017c91f7         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140, CODE XREF=sub_1017c8b20+1665
00000001017c91fb         lea        rsi, qword [aColorlookupuv]                 ; argument #2 for method sub_101844140, "COLOR_LOOKUP_UV"
00000001017c9202         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c9207         call       sub_101844140                               ; sub_101844140
00000001017c920c         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c9210         lea        rsi, qword [aVfresnel]                      ; argument #2 for method sub_101844140, "vFresnel"
00000001017c9217         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c921c         call       sub_101844140                               ; sub_101844140
00000001017c9221         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c9225         lea        rsi, qword [aFresnel]                       ; argument #2 for method sub_101844140, "FRESNEL"
00000001017c922c         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c9231         call       sub_101844140                               ; sub_101844140
00000001017c9236         movaps     xmm0, xmmword [float_value_0_98+16]         ; End of try block started at 0x1017c91a7, 0x10207a160
00000001017c923d         movaps     xmmword [rbp+var_1D0], xmm0
00000001017c9244         mov        r15, qword [rbx+0x78]
00000001017c9248         lea        rdi, qword [aVfresnel]                      ; Begin of try block (catch block at 0x1017c9a87), argument #1 for method sub_10193b4b0, "vFresnel"
00000001017c924f         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c9254         lea        rdx, qword [rbp+var_1D0]                    ; argument #3 for method sub_101845a60
00000001017c925b         mov        rdi, r15                                    ; argument #1 for method sub_101845a60
00000001017c925e         mov        rsi, rax                                    ; argument #2 for method sub_101845a60
00000001017c9261         call       sub_101845a60                               ; sub_101845a60
00000001017c9266         mov        r15, qword [rbx+0x78]
00000001017c926a         lea        rdi, qword [aFresnel]                       ; argument #1 for method sub_10193b4b0, "FRESNEL"
00000001017c9271         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c9276         lea        rdx, qword [rbp+var_1D0]                    ; argument #3 for method sub_101845a60
00000001017c927d         mov        rdi, r15                                    ; argument #1 for method sub_101845a60
00000001017c9280         mov        rsi, rax                                    ; argument #2 for method sub_101845a60
00000001017c9283         call       sub_101845a60                               ; sub_101845a60
00000001017c9288         mov        rdi, qword [rbx+0x78]                       ; End of try block started at 0x1017c9248, argument #1 for method sub_1018442d0
00000001017c928c         mov        rax, qword [rbx+0x10]
00000001017c9290         movss      xmm0, dword [rax+0x33c]
00000001017c9298         movss      dword [rbp+var_170], xmm0
00000001017c92a0         xorps      xmm0, xmm0
00000001017c92a3         movlps     qword [rbp+var_16C], xmm0
00000001017c92aa         mov        dword [rbp+var_164], 0x0
00000001017c92b4         lea        rsi, qword [aParticledepthp]                ; Begin of try block (catch block at 0x1017c9a26), argument #2 for method sub_1018442d0, "PARTICLE_DEPTH_PUSH_PULL"
00000001017c92bb         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_1018442d0
00000001017c92c2         call       sub_1018442d0                               ; sub_1018442d0
00000001017c92c7         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c92b4
00000001017c92cb         mov        rax, qword [rax+0x30]
00000001017c92cf         test       rax, rax
00000001017c92d2         je         loc_1017c93ed

00000001017c92d8         movss      xmm0, dword [rax]
00000001017c92dc         movss      dword [rbp+var_170], xmm0
00000001017c92e4         movsd      xmm2, qword [rax+4]
00000001017c92e9         movaps     xmm0, xmmword [float_value_minus_200+8]     ; 0x102092b70
00000001017c92f0         movaps     xmm1, xmm2
00000001017c92f3         cmpnleps   xmm1, xmm0
00000001017c92f7         cmpltps    xmm0, xmm2
00000001017c92fb         blendps    xmm0, xmm1, 0xd
00000001017c9301         movshdup   xmm3, xmm2
00000001017c9305         movss      xmm1, dword [float_value_1E_minus_05+4]     ; 0x102073854
00000001017c930d         movaps     xmm4, xmm1
00000001017c9310         divss      xmm4, xmm3
00000001017c9314         insertps   xmm2, xmm4, 0x10
00000001017c931a         movaps     xmm3, xmmword [double_value_4_9389E61]      ; double_value_4_9389E61
00000001017c9321         blendvps   xmm3, xmm2
00000001017c9326         movlps     qword [rbp+var_16C], xmm3
00000001017c932d         movss      xmm0, dword [rax+0xc]
00000001017c9332         ucomiss    xmm0, dword [float_value_1E_minus_08]       ; float_value_1E_minus_08
00000001017c9339         jbe        loc_1017c9354

00000001017c933b         divss      xmm1, xmm0
00000001017c933f         movss      dword [rbp+var_164], xmm1
00000001017c9347         movsx      rax, byte [rax+0x10]
00000001017c934c         cmp        rax, 0x2
00000001017c9350         jbe        loc_1017c936f

00000001017c9352         jmp        loc_1017c93bf

                     loc_1017c9354:
00000001017c9354         movss      xmm1, dword [float_value_1E08]              ; float_value_1E08, CODE XREF=sub_1017c8b20+2073
00000001017c935c         movss      dword [rbp+var_164], xmm1
00000001017c9364         movsx      rax, byte [rax+0x10]
00000001017c9369         cmp        rax, 0x2
00000001017c936d         ja         loc_1017c93bf

                     loc_1017c936f:
00000001017c936f         lea        rcx, qword [double_value_0_0520834+544]     ; 0x102092f40, CODE XREF=sub_1017c8b20+2096
00000001017c9376         movss      xmm0, dword [rcx+rax*4]
00000001017c937b         lea        rcx, qword [double_value_0_0520834+556]     ; 0x102092f4c
00000001017c9382         movss      xmm1, dword [rcx+rax*4]
00000001017c9387         lea        rcx, qword [double_value_0_0520834+568]     ; 0x102092f58
00000001017c938e         movss      xmm2, dword [rcx+rax*4]
00000001017c9393         lea        rcx, qword [double_value_0_0520834+580]     ; 0x102092f64
00000001017c939a         movss      xmm3, dword [rcx+rax*4]
00000001017c939f         movss      dword [rbp+var_198], xmm0
00000001017c93a7         movss      dword [rbp+var_194], xmm1
00000001017c93af         movss      dword [rbp+var_190], xmm2
00000001017c93b7         movss      dword [rbp+var_18C], xmm3

                     loc_1017c93bf:
00000001017c93bf         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_1018442d0, CODE XREF=sub_1017c8b20+2098, sub_1017c8b20+2125
00000001017c93c3         lea        rsi, qword [aCsoftparticlep]                ; Begin of try block (catch block at 0x1017c9a87), argument #2 for method sub_1018442d0, "cSoftParticleParams"
00000001017c93ca         lea        rdx, qword [rbp+var_170]                    ; argument #3 for method sub_1018442d0
00000001017c93d1         call       sub_1018442d0                               ; sub_1018442d0
00000001017c93d6         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_1018442d0
00000001017c93da         lea        rsi, qword [aCsoftparticlec]                ; argument #2 for method sub_1018442d0, "cSoftParticleControl"
00000001017c93e1         lea        rdx, qword [rbp+var_198]                    ; argument #3 for method sub_1018442d0
00000001017c93e8         call       sub_1018442d0                               ; sub_1018442d0

                     loc_1017c93ed:
00000001017c93ed         cmp        r14b, 0x7                                   ; CODE XREF=sub_1017c8b20+1970
00000001017c93f1         jne        loc_1017c94c5

00000001017c93f7         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c93fb         lea        rsi, qword [aColoruv]                       ; argument #2 for method sub_101844140, "COLOR_UV"
00000001017c9402         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c9407         call       sub_101844140                               ; sub_101844140
00000001017c940c         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844140
00000001017c9410         lea        rsi, qword [aModulatecolor]                 ; argument #2 for method sub_101844140, "MODULATE_COLOR"
00000001017c9417         mov        edx, 0x1                                    ; argument #3 for method sub_101844140
00000001017c941c         call       sub_101844140                               ; sub_101844140
00000001017c9421         lea        rsi, qword [rbx+0x28]                       ; argument #2 for method sub_1017c9ff0
00000001017c9425         mov        edx, 0x98
00000001017c942a         add        rdx, qword [rbx+0x10]                       ; argument #3 for method sub_1017c9ff0
00000001017c942e         mov        rax, qword [rbp+var_178]
00000001017c9435         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c9439         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c943c         mov        ecx, 0xb                                    ; argument #4 for method sub_1017c9ff0
00000001017c9441         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c9448         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c944f         call       sub_1017c9ff0                               ; sub_1017c9ff0
00000001017c9454         mov        r15, qword [rbx+0x78]                       ; End of try block started at 0x1017c93c3
00000001017c9458         lea        rdi, qword [aDiffusemap]                    ; Begin of try block (catch block at 0x1017c9a83), argument #1 for method sub_10193b4b0, "DIFFUSE_MAP"
00000001017c945f         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c9464         mov        rdi, r15                                    ; argument #1 for method sub_101843cd0
00000001017c9467         mov        rsi, rax                                    ; argument #2 for method sub_101843cd0
00000001017c946a         call       sub_101843cd0                               ; sub_101843cd0
00000001017c946f         mov        word [rax+0x10], 0x101
00000001017c9475         mov        byte [rax+0x12], 0x1
00000001017c9479         mov        r15, qword [rbx+0x78]
00000001017c947d         lea        rdi, qword [aFallofftexture]                ; argument #1 for method sub_10193b4b0, "FALLOFF_TEXTURE"
00000001017c9484         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c9489         mov        rdi, r15                                    ; argument #1 for method sub_101843cd0
00000001017c948c         mov        rsi, rax                                    ; argument #2 for method sub_101843cd0
00000001017c948f         call       sub_101843cd0                               ; sub_101843cd0
00000001017c9494         mov        word [rax+0x10], 0x101
00000001017c949a         mov        byte [rax+0x12], 0x1
00000001017c949e         mov        r15, qword [rbx+0x78]
00000001017c94a2         lea        rdi, qword [aParticlecolort]                ; argument #1 for method sub_10193b4b0, "PARTICLE_COLOR_TEXTURE"
00000001017c94a9         call       sub_10193b4b0                               ; sub_10193b4b0
00000001017c94ae         mov        rdi, r15                                    ; argument #1 for method sub_101843cd0
00000001017c94b1         mov        rsi, rax                                    ; argument #2 for method sub_101843cd0
00000001017c94b4         call       sub_101843cd0                               ; sub_101843cd0
00000001017c94b9         mov        word [rax+0x10], 0x101                      ; End of try block started at 0x1017c9458
00000001017c94bf         mov        byte [rax+0x12], 0x1
00000001017c94c3         jmp        loc_1017c94f5

                     loc_1017c94c5:
00000001017c94c5         lea        rsi, qword [rbx+0x28]                       ; argument #2 for method sub_1017c9ff0, CODE XREF=sub_1017c8b20+2257
00000001017c94c9         mov        edx, 0x98
00000001017c94ce         add        rdx, qword [rbx+0x10]                       ; argument #3 for method sub_1017c9ff0
00000001017c94d2         mov        rax, qword [rbp+var_178]                    ; Begin of try block (catch block at 0x1017c9a87)
00000001017c94d9         mov        qword [rsp+0x200+var_200], rax              ; argument #7 for method sub_1017c9ff0
00000001017c94dd         mov        rdi, rbx                                    ; argument #1 for method sub_1017c9ff0
00000001017c94e0         xor        ecx, ecx                                    ; argument #4 for method sub_1017c9ff0
00000001017c94e2         mov        r8, qword [rbp+var_180]                     ; argument #5 for method sub_1017c9ff0
00000001017c94e9         mov        r9, qword [rbp+var_188]                     ; argument #6 for method sub_1017c9ff0
00000001017c94f0         call       sub_1017c9ff0                               ; sub_1017c9ff0

                     loc_1017c94f5:
00000001017c94f5         mov        r12d, r14d                                  ; End of try block started at 0x1017c94d2, CODE XREF=sub_1017c8b20+2467
00000001017c94f8         and        r14b, 0xf7
00000001017c94fc         mov        byte [rbp+var_178], r14b
00000001017c9503         xor        r14d, r14d
00000001017c9506         lea        r15, qword [byte_1024d0c30]                 ; byte_1024d0c30
00000001017c950d         jmp        loc_1017c9530
00000001017c950f         align      16

                     loc_1017c9510:
00000001017c9510         mov        rdi, r13                                    ; Begin of try block (catch block at 0x1017c9ab9), argument #1 for method sub_101846900, CODE XREF=sub_1017c8b20+3264, sub_1017c8b20+3274, sub_1017c8b20+3287
00000001017c9513         lea        rsi, qword [rbp+var_170]                    ; argument #2 for method sub_101846900
00000001017c951a         call       sub_101846900                               ; sub_101846900

                     loc_1017c951f:
00000001017c951f         add        r14, 0x30                                   ; End of try block started at 0x1017c9510, CODE XREF=sub_1017c8b20+2591
00000001017c9523         cmp        r14, 0x150
00000001017c952a         je         loc_1017c98b6

                     loc_1017c9530:
00000001017c9530         movzx      esi, byte [r14+r15]                         ; argument #2 for method sub_1017ca3b0, CODE XREF=sub_1017c8b20+2541
00000001017c9535         mov        rdi, rbx                                    ; Begin of try block (catch block at 0x1017c9abd), argument #1 for method sub_1017ca3b0
00000001017c9538         call       sub_1017ca3b0                               ; sub_1017ca3b0
00000001017c953d         test       al, al                                      ; End of try block started at 0x1017c9535
00000001017c953f         je         loc_1017c951f

00000001017c9541         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101843bd0
00000001017c9545         mov        rsi, qword [r14+r15+8]                      ; argument #2 for method sub_101843bd0
00000001017c954a         call       sub_101843bd0                               ; sub_101843bd0, Begin of try block (catch block at 0x1017c9aa2)
00000001017c954f         mov        rdi, rax                                    ; End of try block started at 0x1017c954a, Begin of try block (catch block at 0x1017c9abb), argument #1 for method sub_101844740
00000001017c9552         call       sub_101844740                               ; sub_101844740
00000001017c9557         mov        r13, rax
00000001017c955a         movzx      edx, byte [r14+r15]                         ; argument #3 for method sub_1017ca550
00000001017c955f         mov        rdi, rbx                                    ; argument #1 for method sub_1017ca550
00000001017c9562         mov        rsi, qword [rbp+var_180]                    ; argument #2 for method sub_1017ca550
00000001017c9569         mov        rcx, rax                                    ; argument #4 for method sub_1017ca550
00000001017c956c         call       sub_1017ca550                               ; sub_1017ca550
00000001017c9571         mov        dword [rbp+var_15C], 0x0                    ; End of try block started at 0x1017c954f
00000001017c957b         xorps      xmm0, xmm0
00000001017c957e         movlps     qword [rbp+var_158], xmm0
00000001017c9585         lea        rax, qword [rbp+var_16C]
00000001017c958c         mov        qword [rax+6], 0x0
00000001017c9594         mov        qword [rax], 0x0
00000001017c959b         mov        dword [rbp+var_170], 0x3f
00000001017c95a5         mov        byte [rbp+var_160], 0xf
00000001017c95ac         mov        rax, qword [rbx+0x10]
00000001017c95b0         mov        ecx, dword [rax+0x2d0]
00000001017c95b6         test       cl, 0x1
00000001017c95b9         sete       byte [rbp+var_161]
00000001017c95c0         cmp        byte [r14+r15], 0x1
00000001017c95c5         jne        loc_1017c95e0

00000001017c95c7         mov        byte [rbp+var_160], 0x1f
00000001017c95ce         mov        word [rbp+var_16C], 0x311
00000001017c95d7         jmp        loc_1017c975c
00000001017c95dc         align      32

                     loc_1017c95e0:
00000001017c95e0         mov        dl, 0x1f                                    ; CODE XREF=sub_1017c8b20+2725
00000001017c95e2         test       ecx, ecx
00000001017c95e4         jns        loc_1017c95ef

00000001017c95e6         mov        byte [rbp+var_160], 0x8
00000001017c95ed         mov        dl, 0x18

                     loc_1017c95ef:
00000001017c95ef         movzx      esi, byte [rax+0x14]                        ; CODE XREF=sub_1017c8b20+2756
00000001017c95f3         movzx      edi, byte [rax+0x72]
00000001017c95f7         mov        ecx, edi
00000001017c95f9         not        cl
00000001017c95fb         and        cl, 0x1
00000001017c95fe         mov        byte [rbp+var_16C], cl
00000001017c9604         test       dil, 0x2
00000001017c9608         je         loc_1017c96d2

00000001017c960e         cmp        sil, 0x1
00000001017c9612         jne        loc_1017c96d2

                     loc_1017c9618:
00000001017c9618         or         cl, 0x4                                     ; case 1, CODE XREF=sub_1017c8b20+3023
00000001017c961b         mov        byte [rbp+var_16C], cl
00000001017c9621         mov        dword [rbp+var_166], 0x7070606

                     loc_1017c962b:
00000001017c962b         mov        byte [rbp+var_162], 0x0                     ; CODE XREF=sub_1017c8b20+3044, sub_1017c8b20+3328, sub_1017c8b20+3363, sub_1017c8b20+3387, sub_1017c8b20+3473

                     loc_1017c9632:
00000001017c9632         mov        byte [rbp+var_16B], 0x3                     ; CODE XREF=sub_1017c8b20+3001, sub_1017c8b20+3339, sub_1017c8b20+3418, sub_1017c8b20+3449
00000001017c9639         cmp        byte [rax+0x77], 0x0
00000001017c963d         je         loc_1017c96a4

00000001017c963f         mov        rdi, r13
00000001017c9642         add        rdi, 0x40                                   ; argument #1 for method sub_10193b6b0
00000001017c9646         lea        rsi, qword [aAlphatest]                     ; Begin of try block (catch block at 0x1017c9ab9), argument #2 for method sub_10193b6b0, "ALPHA_TEST"
00000001017c964d         mov        edx, 0x1                                    ; argument #3 for method sub_10193b6b0
00000001017c9652         call       sub_10193b6b0                               ; sub_10193b6b0
00000001017c9657         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c9646
00000001017c965b         movzx      eax, byte [rax+0x77]
00000001017c965f         xorps      xmm0, xmm0
00000001017c9662         cvtsi2ss   xmm0, eax
00000001017c9666         divss      xmm0, dword [float_value_255]               ; float_value_255
00000001017c966e         movss      dword [rbp+var_198], xmm0
00000001017c9676         xorps      xmm0, xmm0
00000001017c9679         movlps     qword [rbp+var_194], xmm0
00000001017c9680         mov        dword [rbp+var_18C], 0x0
00000001017c968a         mov        rdi, r13                                    ; Begin of try block (catch block at 0x1017c9a85), argument #1 for method sub_101847a10
00000001017c968d         lea        rsi, qword [aAlphatestrefer]                ; argument #2 for method sub_101847a10, "AlphaTestReferenceValue"
00000001017c9694         lea        rdx, qword [rbp+var_198]                    ; argument #3 for method sub_101847a10
00000001017c969b         call       sub_101847a10                               ; sub_101847a10
00000001017c96a0         mov        rax, qword [rbx+0x10]                       ; End of try block started at 0x1017c968a

                     loc_1017c96a4:
00000001017c96a4         movzx      ecx, byte [rax+0x75]                        ; CODE XREF=sub_1017c8b20+2845
00000001017c96a8         cmp        rcx, 0x4
00000001017c96ac         ja         loc_1017c9745

00000001017c96b2         lea        rdx, qword [switch_table_1017c9b04]         ; switch_table_1017c9b04
00000001017c96b9         movsxd     rcx, dword [rdx+rcx*4]
00000001017c96bd         add        rcx, rdx
00000001017c96c0         jmp        rcx                                         ; switch statement using table at 0x1017c9b04, with 1 cases, 0x1017c975c
00000001017c96c2         or         byte [rbp-0x16c], 0x2
00000001017c96c9         mov        byte [rbp-0x16a], 0x0
00000001017c96d0         jmp        sub_1017c8b20+3095

                     loc_1017c96d2:
00000001017c96d2         movzx      esi, sil                                    ; CODE XREF=sub_1017c8b20+2792, sub_1017c8b20+2802
00000001017c96d6         cmp        esi, 0x8
00000001017c96d9         ja         loc_1017c9632

00000001017c96df         mov        esi, esi
00000001017c96e1         lea        rdi, qword [switch_table_1017c9ae0]         ; switch_table_1017c9ae0
00000001017c96e8         movsxd     rsi, dword [rdi+rsi*4]
00000001017c96ec         add        rsi, rdi
00000001017c96ef         jmp        rsi                                         ; switch statement using table at 0x1017c9ae0, with 9 cases, 0x1017c9618,0x1017c96f1,0x1017c980d,0x1017c9825,0x1017c9830,0x1017c9848,0x1017c9860,0x1017c987f,0x1017c989e

                     loc_1017c96f1:
00000001017c96f1         or         cl, 0x4                                     ; case 0, CODE XREF=sub_1017c8b20+3023
00000001017c96f4         mov        byte [rbp+var_16C], cl
00000001017c96fa         mov        dword [rbp+var_166], 0x1010101
00000001017c9704         jmp        loc_1017c962b
00000001017c9709         or         byte [rbp-0x16c], 0x2
00000001017c9710         mov        byte [rbp-0x16a], 0x6
00000001017c9717         jmp        sub_1017c8b20+3109
00000001017c9719         or         byte [rbp-0x16c], 0x2
00000001017c9720         mov        byte [rbp-0x16a], 0x7
00000001017c9727         jmp        sub_1017c8b20+3109
00000001017c9729         or         byte [rbp-0x16c], 0x2
00000001017c9730         mov        byte [rbp-0x16a], 0x7
00000001017c9737         mov        byte [rbp-0x167], 0x2                       ; CODE XREF=sub_1017c8b20+2992
00000001017c973e         or         byte [rbp-0x160], 0x20

                     loc_1017c9745:
00000001017c9745         movzx      eax, byte [rax+0x76]                        ; CODE XREF=sub_1017c8b20+2956, sub_1017c8b20+3063, sub_1017c8b20+3079
00000001017c9749         mov        byte [r13+4], al
00000001017c974d         mov        rax, qword [rbx+0x10]
00000001017c9751         mov        eax, dword [rax+0x78]
00000001017c9754         mov        dword [r13], eax
00000001017c9758         mov        rax, qword [rbx+0x10]

                     loc_1017c975c:
00000001017c975c         movsd      xmm0, qword [rax+0x7c]                      ; case 0, CODE XREF=sub_1017c8b20+2743, sub_1017c8b20+2976
00000001017c9761         lea        rcx, qword [qword_1024cff88]                ; qword_1024cff88
00000001017c9768         movsd      xmm1, qword [rcx]
00000001017c976c         cmpneqps   xmm1, xmm0
00000001017c9770         unpcklps   xmm1, xmm1
00000001017c9773         movmskpd   ecx, xmm1
00000001017c9777         test       ecx, ecx
00000001017c9779         je         loc_1017c9789

00000001017c977b         or         byte [rbp+var_16C], 0x8
00000001017c9782         movlps     qword [rbp+var_158], xmm0

                     loc_1017c9789:
00000001017c9789         cmp        byte [rax+0x11], 0x0                        ; CODE XREF=sub_1017c8b20+3161
00000001017c978d         je         loc_1017c97b7

                     loc_1017c978f:
00000001017c978f         movzx      ecx, byte [rbp+var_16C]                     ; CODE XREF=sub_1017c8b20+3305
00000001017c9796         and        cl, 0xef
00000001017c9799         mov        byte [rbp+var_16C], cl
00000001017c979f         cmp        r12b, 0x7
00000001017c97a3         jne        loc_1017c97d3

                     loc_1017c97a5:
00000001017c97a5         or         cl, 0x11                                    ; CODE XREF=sub_1017c8b20+3249
00000001017c97a8         mov        byte [rbp+var_16C], cl
00000001017c97ae         mov        byte [rbp+var_16B], 0x6
00000001017c97b5         jmp        loc_1017c97dc

                     loc_1017c97b7:
00000001017c97b7         cmp        dword [rbx+0x1c], 0xffffffff                ; CODE XREF=sub_1017c8b20+3181
00000001017c97bb         je         loc_1017c97fc

                     loc_1017c97bd:
00000001017c97bd         movzx      ecx, byte [rbp+var_16C]                     ; CODE XREF=sub_1017c8b20+3299, sub_1017c8b20+3307
00000001017c97c4         or         cl, 0x10
00000001017c97c7         mov        byte [rbp+var_16C], cl
00000001017c97cd         cmp        r12b, 0x7
00000001017c97d1         je         loc_1017c97a5

                     loc_1017c97d3:
00000001017c97d3         mov        edx, r12d                                   ; CODE XREF=sub_1017c8b20+3203
00000001017c97d6         cmp        r12b, 0xb
00000001017c97da         je         loc_1017c97e6

                     loc_1017c97dc:
00000001017c97dc         cmp        dword [rbx+0x1c], 0xffffffff                ; CODE XREF=sub_1017c8b20+3221
00000001017c97e0         je         loc_1017c9510

                     loc_1017c97e6:
00000001017c97e6         test       byte [rax+0x73], 0x1                        ; CODE XREF=sub_1017c8b20+3258
00000001017c97ea         je         loc_1017c9510

00000001017c97f0         or         byte [rbp+var_160], 0x10
00000001017c97f7         jmp        loc_1017c9510

                     loc_1017c97fc:
00000001017c97fc         cmp        byte [rbp+var_178], 0x3                     ; CODE XREF=sub_1017c8b20+3227
00000001017c9803         je         loc_1017c97bd

00000001017c9805         cmp        dword [rbx+0x24], 0xffffffff
00000001017c9809         je         loc_1017c978f

00000001017c980b         jmp        loc_1017c97bd

                     loc_1017c980d:
00000001017c980d         or         cl, 0x4                                     ; case 2, CODE XREF=sub_1017c8b20+3023
00000001017c9810         mov        byte [rbp+var_16C], cl
00000001017c9816         mov        dword [rbp+var_166], 0x7030000
00000001017c9820         jmp        loc_1017c962b

                     loc_1017c9825:
00000001017c9825         mov        byte [rbp+var_160], dl                      ; case 3, CODE XREF=sub_1017c8b20+3023
00000001017c982b         jmp        loc_1017c9632

                     loc_1017c9830:
00000001017c9830         or         cl, 0x4                                     ; case 4, CODE XREF=sub_1017c8b20+3023
00000001017c9833         mov        byte [rbp+var_16C], cl
00000001017c9839         mov        dword [rbp+var_166], 0x1010606
00000001017c9843         jmp        loc_1017c962b

                     loc_1017c9848:
00000001017c9848         or         cl, 0x4                                     ; case 5, CODE XREF=sub_1017c8b20+3023
00000001017c984b         mov        byte [rbp+var_16C], cl
00000001017c9851         mov        dword [rbp+var_166], 0x7070101
00000001017c985b         jmp        loc_1017c962b

                     loc_1017c9860:
00000001017c9860         or         cl, 0x4                                     ; case 6, CODE XREF=sub_1017c8b20+3023
00000001017c9863         mov        byte [rbp+var_16C], cl
00000001017c9869         mov        dword [rbp+var_166], 0x1010101
00000001017c9873         mov        byte [rbp+var_162], 0x3
00000001017c987a         jmp        loc_1017c9632

                     loc_1017c987f:
00000001017c987f         or         cl, 0x4                                     ; case 7, CODE XREF=sub_1017c8b20+3023
00000001017c9882         mov        byte [rbp+var_16C], cl
00000001017c9888         mov        dword [rbp+var_166], 0x1010101
00000001017c9892         mov        byte [rbp+var_162], 0x4
00000001017c9899         jmp        loc_1017c9632

                     loc_1017c989e:
00000001017c989e         or         cl, 0x4                                     ; case 8, CODE XREF=sub_1017c8b20+3023
00000001017c98a1         mov        byte [rbp+var_16C], cl
00000001017c98a7         mov        dword [rbp+var_166], 0x1080109
00000001017c98b1         jmp        loc_1017c962b

                     loc_1017c98b6:
00000001017c98b6         mov        rdi, qword [rbx+0x78]                       ; argument #1 for method sub_101844870, CODE XREF=sub_1017c8b20+2570
00000001017c98ba         mov        esi, 0x2                                    ; Begin of try block (catch block at 0x1017c9a21), argument #2 for method sub_101844870
00000001017c98bf         call       sub_101844870                               ; sub_101844870
00000001017c98c4         mov        ebx, eax                                    ; End of try block started at 0x1017c98ba, Begin of try block
00000001017c98c6         lea        rdi, qword [rbp+var_1F0]                    ; argument #1 for method sub_101819d00
00000001017c98cd         call       sub_101819d00                               ; sub_101819d00
00000001017c98d2         lea        rdi, qword [rbp+var_148]                    ; argument #1 for method sub_1015e6aa0
00000001017c98d9         call       sub_1015e6aa0                               ; sub_1015e6aa0

                     loc_1017c98de:
00000001017c98de         mov        rax, qword [___stack_chk_guard_10228c320]   ; ___stack_chk_guard_10228c320, CODE XREF=sub_1017c8b20+76, sub_1017c8b20+211
00000001017c98e5         mov        rax, qword [rax]
00000001017c98e8         cmp        rax, qword [rbp+var_30]
00000001017c98ec         jne        loc_1017c9902

00000001017c98ee         mov        eax, ebx
00000001017c98f0         add        rsp, 0x1d8
00000001017c98f7         pop        rbx
00000001017c98f8         pop        r12
00000001017c98fa         pop        r13
00000001017c98fc         pop        r14
00000001017c98fe         pop        r15
00000001017c9900         pop        rbp
00000001017c9901         ret
                        ; endp

                     loc_1017c9902:
00000001017c9902         call       imp___stubs____stack_chk_fail               ; __stack_chk_fail, CODE XREF=sub_1017c8b20+3532
                        ; endp

                     loc_1017c9907:
00000001017c9907         lea        rdi, qword [byte_1024ab9f0]                 ; byte_1024ab9f0, CODE XREF=sub_1017c8b20+708
00000001017c990e         call       imp___stubs____cxa_guard_acquire            ; __cxa_guard_acquire
00000001017c9913         test       eax, eax
00000001017c9915         je         loc_1017c8dea

00000001017c991b         lea        rax, qword [qword_value_0+5888]             ; 0x102388140
00000001017c9922         mov        rax, qword [rax]
00000001017c9925         mov        qword [qword_1024ab948], rax                ; qword_1024ab948
00000001017c992c         lea        r13, qword [qword_1024ab948]                ; qword_1024ab948
00000001017c9933         mov        qword [qword_1024ab950], 0x0                ; qword_1024ab950
00000001017c993e         movsd      xmm0, qword [aNst3112badweak+144]           ; 0x1020738f0
00000001017c9946         movsd      qword [double_1024ab958], xmm0              ; double_1024ab958
00000001017c994e         mov        word [word_1024ab960], 0x100                ; word_1024ab960
00000001017c9957         movsd      xmm0, qword [aNnnnnnnn+112]                 ; 0x102074ad0
00000001017c995f         movsd      qword [double_1024ab964], xmm0              ; double_1024ab964
00000001017c9967         mov        dword [dword_1024ab970], 0x0                ; dword_1024ab970
00000001017c9971         mov        qword [qword_1024ab978], 0x0                ; qword_1024ab978
00000001017c997c         movabs     rax, 0x3f8000003f800000
00000001017c9986         mov        qword [qword_1024ab980], rax                ; qword_1024ab980
00000001017c998d         mov        qword [qword_1024ab9c8], 0x0                ; qword_1024ab9c8
00000001017c9998         mov        dword [dword_1024ab9d0], 0x0                ; dword_1024ab9d0
00000001017c99a2         xorps      xmm0, xmm0
00000001017c99a5         movups     xmmword [qword_1024ab980+8], xmm0           ; 0x1024ab988
00000001017c99ac         movups     xmmword [qword_1024ab980+24], xmm0          ; 0x1024ab998
00000001017c99b3         movups     xmmword [qword_1024ab980+40], xmm0          ; 0x1024ab9a8
00000001017c99ba         movups     xmmword [qword_1024ab980+52], xmm0          ; 0x1024ab9b4
00000001017c99c1         movups     xmmword [dword_1024ab9d0+8], xmm0           ; 0x1024ab9d8
00000001017c99c8         mov        qword [qword_1024ab9e8], 0x0                ; qword_1024ab9e8
00000001017c99d3         lea        rdi, qword [sub_1017dcae0]                  ; sub_1017dcae0
00000001017c99da         lea        rdx, qword [0x100000000]                    ; 0x100000000
00000001017c99e1         mov        rsi, r13
00000001017c99e4         call       imp___stubs____cxa_atexit                   ; __cxa_atexit
00000001017c99e9         lea        rdi, qword [byte_1024ab9f0]                 ; byte_1024ab9f0
00000001017c99f0         call       imp___stubs____cxa_guard_release            ; __cxa_guard_release
00000001017c99f5         jmp        loc_1017c8dea
00000001017c99fa         jmp        sub_1017c9a50+57                            ; Begin of catch block for try block at 0x1017c9191
00000001017c99ff         mov        rbx, rax                                    ; Begin of catch block for try block at 0x1017c9151
00000001017c9a02         lea        rdi, qword [rbp-0x198]
00000001017c9a09         call       sub_1000d0800                               ; sub_1000d0800
00000001017c9a0e         jmp        sub_1017c9aa0+32
00000001017c9a13         jmp        sub_1017c9aa0+29                            ; Begin of catch block for try block at 0x1017c9133
00000001017c9a18         jmp        sub_1017c9a50+57                            ; Begin of catch block for try block at 0x1017c8bc5
00000001017c9a1a         jmp        sub_1017c9aa0+29                            ; Begin of catch block for try block at 0x1017c8f65
00000001017c9a1f         jmp        sub_1017c9a50+57                            ; Begin of catch block for try block at 0x1017c8cca
00000001017c9a21         jmp        sub_1017c9aa0+29                            ; Begin of catch block for try block at 0x1017c98ba
00000001017c9a26         jmp        sub_1017c9aa0+29                            ; Begin of catch block for try block at 0x1017c92b4
00000001017c9a2b         jmp        sub_1017c9aa0+29                            ; Begin of catch block for try block at 0x1017c8db1
```

搜索字符串 PARTICLE_COLOR_TEXTURE 找到了两处函数用到了，这一大段是第一处，反汇编出大概的伪代码