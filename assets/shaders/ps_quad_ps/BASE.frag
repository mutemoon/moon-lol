#version 450

layout(set = 2, binding = 1) uniform texture2D TEXTURE_texture;
layout(set = 2, binding = 2) uniform sampler TEXTURE_sampler;
layout(set = 2, binding = 3) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 2, binding = 4) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 2, binding = 5) uniform texture2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 6) uniform sampler CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler;
layout(set = 2, binding = 7) uniform texture2D CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 8) uniform sampler CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler;

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
