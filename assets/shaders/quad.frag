#version 450

struct UniformsPixel
{
    float AlphaTestReferenceValue;
    vec4 cDepthConversionParams;
    vec4 cSoftParticleParams;
    vec4 cSoftParticleControl;
    vec4 cPaletteSelectMain;
    vec4 cPaletteSrcMixerMain;
    vec4 APPLY_TEAM_COLOR_CORRECTION;
};

layout(set = 3, binding = 1) uniform UniformsPixel uniforms_pixel;

layout(set = 3, binding = 2) uniform texture2D TEXTURE_texture;
layout(set = 3, binding = 3) uniform sampler TEXTURE_sampler;
layout(set = 3, binding = 4) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 3, binding = 5) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 3, binding = 6) uniform texture2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture;
layout(set = 3, binding = 7) uniform sampler CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler;
layout(set = 3, binding = 8) uniform texture2D CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture;
layout(set = 3, binding = 9) uniform sampler CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler;

#ifdef MASKED
layout(set = 3, binding = 10) uniform texture2D NAVMESH_MASK_TEXTURE_texture;
layout(set = 3, binding = 11) uniform sampler NAVMESH_MASK_TEXTURE_sampler;
#endif
#ifdef MULT_PASS
layout(set = 3, binding = 12) uniform texture2D TEXTUREMULT_texture;
layout(set = 3, binding = 13) uniform sampler TEXTUREMULT_sampler;
#endif
#ifdef PALETTIZE_TEXTURES
layout(set = 3, binding = 14) uniform texture2D sPalettesTexture_texture;
layout(set = 3, binding = 15) uniform sampler sPalettesTexture_sampler;
#endif
#ifdef SOFT_PARTICLES
layout(set = 3, binding = 16) uniform texture2D sDepthTexture_texture;
layout(set = 3, binding = 17) uniform sampler sDepthTexture_sampler;
#endif

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec4 TEXCOORD1;
#ifdef MASKED
layout(location = 2) in vec4 TEXCOORD2;
#else
layout(location = 3) in vec2 TEXCOORD2;
#endif

layout(location = 0) out vec4 SV_Target0;

void main()
{
#ifdef FOW_IGNORE_VISIBILITY
    // 特殊情况：忽略可见性，只输出带有alpha的黑色
    float base_alpha = (texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy) * texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), TEXCOORD1.zw)).w;
    SV_Target0 = vec4(0.0, 0.0, 0.0, TEXCOORD0.a * base_alpha);
    return;
#endif

    // 1. 基础纹理采样和颜色混合
    vec4 tex_color = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);
    
#ifdef MULT_PASS
    tex_color *= texture(sampler2D(TEXTUREMULT_texture, TEXTUREMULT_sampler), TEXCOORD1.zw);
#endif

#ifdef PALETTIZE_TEXTURES
    // 调色板颜色查找
    vec2 palette_uv = vec2(
        clamp(dot(tex_color, uniforms_pixel.cPaletteSrcMixerMain), 0.0, 1.0), 
        uniforms_pixel.cPaletteSelectMain.x
    ) + vec2(uniforms_pixel.cPaletteSelectMain.z, uniforms_pixel.cPaletteSelectMain.w);
    vec4 palette_color = texture(sampler2D(sPalettesTexture_texture, sPalettesTexture_sampler), palette_uv);
    tex_color = vec4(palette_color.rgb, tex_color.a);
#endif

    vec4 particle_color = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), TEXCOORD1.zw);
    vec4 combined_color = TEXCOORD0 * tex_color * particle_color;
    
    // 2. 颜色重映射 (Color Remapping)
    vec3 final_rgb = combined_color.rgb;
    float luminance = dot(final_rgb, vec3(0.2126, 0.7152, 0.0722));
    vec4 remap_color = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(luminance, 0.5));
    if (remap_color.w > 0.0) {
        final_rgb = remap_color.xyz;
    }
    
    float final_alpha = combined_color.a;
    vec4 final_color = vec4(final_rgb, final_alpha);
    
    // 3. 色盲模式颜色校正
#ifdef COLORPALETTE_COLORBLIND
    if (uniforms_pixel.APPLY_TEAM_COLOR_CORRECTION.x != 0.0) {
        float green_minus_red = final_rgb.g - final_rgb.r;
        if (green_minus_red > 0.001) {
            final_color = vec4(0.0, final_rgb.g * 0.3, final_rgb.z + (final_rgb.g * 3.0), final_alpha * 1.75);
        } else if (green_minus_red < -0.001) {
            final_color = vec4(final_rgb.r, final_rgb.g + (final_rgb.r * 0.3), final_rgb.z, final_alpha * 2.0);
        }
    }
#endif

    // 4. 软粒子或战争迷雾效果
#ifdef SOFT_PARTICLES
    // 软粒子深度淡出计算
    float scene_depth = texelFetch(sDepthTexture, ivec3(int(gl_FragCoord.x), int(gl_FragCoord.y), 0), 0).x;
    float linear_scene_depth = 1.0 / (uniforms_pixel.cDepthConversionParams.x + scene_depth * uniforms_pixel.cDepthConversionParams.y);
    float linear_particle_depth = 1.0 / (uniforms_pixel.cDepthConversionParams.x + gl_FragCoord.z * uniforms_pixel.cDepthConversionParams.y);
    vec2 depth_diff = vec2(linear_scene_depth - linear_particle_depth);
    vec2 soft_fade = clamp((depth_diff - uniforms_pixel.cSoftParticleParams.xy) * uniforms_pixel.cSoftParticleParams.zw, 0.0, 1.0);
    soft_fade = (soft_fade * soft_fade) * (3.0 - (soft_fade * 2.0));
    float soft_particle_factor = soft_fade.x - soft_fade.y;

    vec4 soft_color_effect = final_color * soft_particle_factor;
    final_color.rgb = (final_color.rgb * uniforms_pixel.cSoftParticleControl.x) + (soft_color_effect.rgb * uniforms_pixel.cSoftParticleControl.y);
    final_color.a = (uniforms_pixel.cSoftParticleControl.z * final_alpha) + (uniforms_pixel.cSoftParticleControl.w * soft_color_effect.a);
#else
    #ifndef DISABLE_FOW
        // 应用战争迷雾
        #ifdef MASKED
            final_color.rgb *= texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2.xy).w;
        #else
            final_color.rgb *= texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2).w;
        #endif
    #endif
#endif

    // 5. 导航网格遮罩 (Navmesh Mask)
#ifdef MASKED
    float mask = texture(sampler2D(NAVMESH_MASK_TEXTURE_texture, NAVMESH_MASK_TEXTURE_sampler), TEXCOORD2.zw).x;
    final_color.a = min(final_color.a, mask);
#endif

    // 6. Alpha 测试
#ifdef ALPHA_TEST
    if (final_color.a < uniforms_pixel.AlphaTestReferenceValue) {
        discard;
    }
#endif

    SV_Target0 = final_color;
}