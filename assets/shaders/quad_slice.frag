#version 450

// 统一的 Uniform 结构体
// 根据启用的功能包含不同的成员
struct UniformsPixel
{
    float AlphaTestReferenceValue;
    vec2 SLICE_RANGE; // 用于切片效果的参数
    vec4 APPLY_TEAM_COLOR_CORRECTION; // 用于色盲模式的参数
};

layout(set = 2, binding = 1) uniform UniformsPixel uniforms_pixel;

// 统一的纹理采样器
layout(set = 2, binding = 2) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 2, binding = 3) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 2, binding = 4) uniform texture2D TEXTURE_texture;
layout(set = 2, binding = 5) uniform sampler TEXTURE_sampler;
layout(set = 2, binding = 6) uniform texture2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 7) uniform sampler CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler;

#ifndef DISABLE_FOW
layout(set = 2, binding = 8) uniform texture2D SAMPLER_FOW_texture;
layout(set = 2, binding = 9) uniform sampler SAMPLER_FOW_sampler;
#endif

// 输入的顶点属性 (Varyings)
layout(location = 0) in vec4 TEXCOORD0; // 顶点颜色
layout(location = 1) in vec4 TEXCOORD1; // .xy: 主纹理 UV, .zw: 粒子颜色纹理 UV

// 根据 MASKED 和 DISABLE_FOW 宏决定 TEXCOORD2 的类型
#ifndef DISABLE_FOW
    #ifdef MASKED
layout(location = 2)         in vec4 TEXCOORD2; // .xy: FOW UV, .zw: Mask UV
    #else
layout(location = 3)         in vec2 TEXCOORD2; // FOW UV
    #endif
#endif

layout(location = 0) out vec4 SV_Target0; // 输出的像素颜色

void main()
{
    // 1. 基础纹理采样
    vec4 particle_color = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), TEXCOORD1.zw);
    vec4 tex_data = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);

    // 2. 计算切片效果的 Alpha
    // 使用 tex_data.x (距离场) 和 particle_color.w (偏移) 来计算
    // 这是一个二次函数，用于在 SLICE_RANGE.x 定义的范围内创建平滑的 Alpha 过渡
    float slice_dist = tex_data.x - particle_color.w;
    float slice_alpha_factor = ((slice_dist + uniforms_pixel.SLICE_RANGE.x) * (uniforms_pixel.SLICE_RANGE.x - slice_dist)) * uniforms_pixel.SLICE_RANGE.y;
    float base_alpha = clamp(tex_data.w * slice_alpha_factor, 0.0, 1.0);

    // 3. 计算基础颜色并进行颜色重映射
    vec3 base_rgb = (TEXCOORD0 * particle_color).xyz;
    float luminance = dot(base_rgb, vec3(0.2126, 0.7152, 0.0722));
    vec4 remap_color = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(luminance, 0.5));
    if (remap_color.w > 0.0) {
        base_rgb = remap_color.xyz;
    }

    // 4. 应用战争迷雾 (FOW) 和颜色乘数
    float fow_factor = 1.0;
#ifndef DISABLE_FOW
    #ifdef MASKED
        // MASKED 模式下，FOW 的 UV 在 TEXCOORD2.xy
        fow_factor = texture(sampler2D(SAMPLER_FOW_texture, SAMPLER_FOW_sampler), TEXCOORD2.xy).w;
    #else
        // 普通模式下，FOW 的 UV 在 TEXCOORD2
        fow_factor = texture(sampler2D(SAMPLER_FOW_texture, SAMPLER_FOW_sampler), TEXCOORD2).w;
    #endif
#endif

    float color_multiplier = tex_data.y;
    vec3 final_rgb = base_rgb * color_multiplier * fow_factor;

    // 5. 组合成最终颜色向量
    vec4 final_color = vec4(final_rgb, base_alpha);

    // 6. 应用色盲模式颜色校正
#ifdef COLORPALETTE_COLORBLIND
    if (uniforms_pixel.APPLY_TEAM_COLOR_CORRECTION.x != 0.0) {
        float green_minus_red = final_color.g - final_color.r;
        if (green_minus_red > 0.001) { // 偏绿色，校正为友方颜色
            final_color = vec4(0.0, final_color.g * 0.3, final_color.b + (final_color.g * 3.0), base_alpha * 1.75);
        } else if (green_minus_red < -0.001) { // 偏红色，校正为敌方颜色
            vec4 temp_color = final_color;
            temp_color.g += final_color.r * 0.3;
            temp_color.a = base_alpha * 2.0;
            final_color = temp_color;
        }
    }
#endif

    // 7. Alpha 测试 (使用可能被色盲模式修改过的 Alpha 值)
#ifdef ALPHA_TEST
    if (final_color.a < uniforms_pixel.AlphaTestReferenceValue) {
        discard;
    }
#endif

    // 8. 输出最终颜色
    SV_Target0 = final_color;
}