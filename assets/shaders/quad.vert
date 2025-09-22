#version 450

struct CameraView {
    mat4 clip_from_world;
    mat4 unjittered_clip_from_world;
    mat4 world_from_clip;
    mat4 world_from_view;
    mat4 view_from_world;
    mat4 clip_from_view;
    mat4 view_from_clip;
    vec3 world_position;
};

layout(set = 0, binding = 0) uniform CameraView camera_view;
struct UniformsVertex
{
    vec4 FOG_OF_WAR_PARAMS;
    vec4 FOG_OF_WAR_ALWAYS_BELOW_Y;
    vec4 FOW_HEIGHT_FADE;
    vec4 NAV_GRID_XFORM;
    float PARTICLE_DEPTH_PUSH_PULL;
    vec4 TEXTURE_INFO;
    vec4 TEXTURE_INFO_2;
};
layout(set = 2, binding = 0) uniform UniformsVertex uniforms_vertext;

layout(location = 0) in vec3 ATTR0;
layout(location = 3) in vec4 ATTR3;
layout(location = 8) in vec4 ATTR8;
layout(location = 9) in vec2 ATTR9;

layout(location = 0) out vec4 TEXCOORD0;
layout(location = 1) out vec4 TEXCOORD1;
#ifdef MASKED
layout(location = 2) out vec4 TEXCOORD2;
#else
layout(location = 3) out vec2 TEXCOORD2;
#endif
#ifdef ALPHA_EROSION
layout(location = 4) out float TEXCOORD3;
#endif

void main()
{
    // --- 通用计算 ---
    // 计算最终的顶点位置（包含粒子推拉效果）
    gl_Position = camera_view.clip_from_world * vec4(ATTR0 + (normalize(ATTR0 - camera_view.world_position) * uniforms_vertext.PARTICLE_DEPTH_PUSH_PULL), 1.0);
    // 传递粒子颜色
    TEXCOORD0 = ATTR3.zyxw;

    // --- 计算 TEXCOORD1 (主纹理和多重纹理的 UV) ---
    float frame = floor(ATTR8.z);

    #ifdef MULT_PASS
    // MULT_PASS 模式：为两张纹理图集计算 UV
    // 计算主纹理 UV
    float u_offset = floor(frame * uniforms_vertext.TEXTURE_INFO.y);
    vec2 uv1 = vec2(
        (ATTR8.x + (frame - (u_offset * uniforms_vertext.TEXTURE_INFO.x))) * uniforms_vertext.TEXTURE_INFO.y,
        (ATTR8.y + u_offset) * uniforms_vertext.TEXTURE_INFO.z
    );

    // 计算第二张（多重）纹理 UV
    float u_offset_2 = floor(frame * uniforms_vertext.TEXTURE_INFO_2.y);
    vec2 uv2 = vec2(
        (ATTR9.x + (frame - (u_offset_2 * uniforms_vertext.TEXTURE_INFO_2.x))) * uniforms_vertext.TEXTURE_INFO_2.y,
        (ATTR9.y + u_offset_2) * uniforms_vertext.TEXTURE_INFO_2.z
    );

    TEXCOORD1 = vec4(uv1, uv2);
    #else
    // 单 Pass 模式：只为主纹理图集计算 UV，第二组 UV 直接使用输入
    float u_offset = floor(frame * uniforms_vertext.TEXTURE_INFO.y);
    vec2 uv1 = vec2(
        (ATTR8.x + (frame - (u_offset * uniforms_vertext.TEXTURE_INFO.x))) * uniforms_vertext.TEXTURE_INFO.y,
        (ATTR8.y + u_offset) * uniforms_vertext.TEXTURE_INFO.z
    );

    TEXCOORD1 = vec4(uv1, ATTR9.xy);
    #endif


    // --- 计算 TEXCOORD2 (战争迷雾和导航网格的 UV) ---
    #ifndef DISABLE_FOW
    // 战争迷雾已启用
    vec2 fow_uv = (ATTR0.xz * uniforms_vertext.FOG_OF_WAR_PARAMS.xy) + uniforms_vertext.FOG_OF_WAR_PARAMS.zw;
    #ifdef MASKED
        // 同时启用 MASKED
        vec2 nav_grid_uv = (uniforms_vertext.NAV_GRID_XFORM.xy * ATTR0.xz) + uniforms_vertext.NAV_GRID_XFORM.zw;
        TEXCOORD2 = vec4(fow_uv, nav_grid_uv);
    #else
        // 未启用 MASKED
        TEXCOORD2 = fow_uv;
    #endif
    #else
    // 战争迷雾已禁用 (DISABLE_FOW)
    #ifdef MASKED
        // 但启用了 MASKED
        vec2 nav_grid_uv = (uniforms_vertext.NAV_GRID_XFORM.xy * ATTR0.xz) + uniforms_vertext.NAV_GRID_XFORM.zw;
        TEXCOORD2 = vec4(vec2(0.0), nav_grid_uv);
    #else
        // 两者都未启用
        TEXCOORD2 = vec2(0.0);
    #endif
    #endif


    // --- 计算 TEXCOORD3 (Alpha 侵蚀) ---
    #ifdef ALPHA_EROSION
    TEXCOORD3 = ATTR8.w;
    #endif
}