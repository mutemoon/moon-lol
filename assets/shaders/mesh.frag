#version 450

struct UniformsPixel
{
    vec4 FOW_EDGE_CONTROL;
    vec2 COLOR_LOOKUP_UV;
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

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec3 TEXCOORD1;
layout(location = 2) in vec3 TEXCOORD2;
layout(location = 3) in vec3 TEXCOORD6;
layout(location = 0) out vec4 SV_Target0;

void main()
{
    vec4 _58 = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);
    vec4 _64 = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), uniforms_pixel.COLOR_LOOKUP_UV);
    vec4 _66 = TEXCOORD0 * (_58 * _64);
    vec3 _67 = _66.xyz;
    vec4 _73 = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(dot(_67, vec3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875)), 0.5));
    vec3 _80 = vec3(0.0);
    if (_73.w > 0.0)
    {
        _80 = _73.xyz;
    }
    else
    {
        _80 = _67;
    }
    float _81 = _66.w;
    vec4 _85 = vec4(_80, _81);
    vec3 _101 = clamp(((_85.xyz + (TEXCOORD6 * _81)).xyz * mix(uniforms_pixel.FOW_EDGE_CONTROL.w, texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2.xy).w, TEXCOORD2.z)).xyz, vec3(0.0), vec3(1.0));
    SV_Target0 = vec4(_101.x, _101.y, _101.z, _85.w);
}
