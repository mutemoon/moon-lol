#version 450

struct UniformsPixel
{
    vec4 COLOR_UV;
    vec4 MODULATE_COLOR;
};

layout(set = 3, binding = 1) uniform UniformsPixel uniforms_pixel;

layout(set = 3, binding = 2) uniform texture2D DIFFUSE_MAP_texture;
layout(set = 3, binding = 3) uniform sampler DIFFUSE_MAP_sampler;
layout(set = 3, binding = 4) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 3, binding = 5) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 3, binding = 6) uniform texture2D CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture;
layout(set = 3, binding = 7) uniform sampler CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler;

layout(location = 0) in vec3 TEXCOORD0;
layout(location = 1) in vec2 TEXCOORD2;
layout(location = 0) out vec4 SV_Target0;

void main()
{
    vec4 _53 = (texture(sampler2D(DIFFUSE_MAP_texture, DIFFUSE_MAP_sampler), TEXCOORD0.xy) * texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), uniforms_pixel.COLOR_UV.xy)) * uniforms_pixel.MODULATE_COLOR;
    _53.w = _53.w * (isnan(0.0) ? TEXCOORD0.z : (isnan(TEXCOORD0.z) ? 0.0 : max(TEXCOORD0.z, 0.0)));
    vec3 _64 = _53.xyz * texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2).w;
    SV_Target0 = vec4(_64.x, _64.y, _64.z, _53.w);
}
