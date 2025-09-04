#version 450

struct UniformsPixel
{
    vec2 SLICE_RANGE;
};

layout(set = 2, binding = 1) uniform UniformsPixel uniforms_pixel;
layout(set = 2, binding = 2) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 2, binding = 3) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 2, binding = 4) uniform texture2D TEXTURE_texture;
layout(set = 2, binding = 5) uniform sampler TEXTURE_sampler;
layout(set = 2, binding = 6) uniform texture2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture;
layout(set = 2, binding = 7) uniform sampler CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler;
layout(set = 2, binding = 8) uniform texture2D SAMPLER_FOW_texture;
layout(set = 2, binding = 9) uniform sampler SAMPLER_FOW_sampler;

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec4 TEXCOORD1;
layout(location = 2) in vec2 TEXCOORD2;
layout(location = 0) out vec4 SV_Target0;

void main() {
    vec4 _53 = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), TEXCOORD1.zw);
    vec4 _59 = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);
    float _61 = _59.x - _53.w;
    vec3 _73 = (TEXCOORD0 * _53).xyz;
    vec4 _79 = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(dot(_73, vec3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875)), 0.5));
    vec3 _86 = vec3(0.0);
    if (_79.w > 0.0)
    {
        _86 = _79.xyz;
    }
    else
    {
        _86 = _73;
    }
    vec4 _90 = vec4(_86, clamp(_59.w * (((_61 + uniforms_pixel.SLICE_RANGE.x) * (uniforms_pixel.SLICE_RANGE.x - _61)) * uniforms_pixel.SLICE_RANGE.y), 0.0, 1.0));
    vec3 _99 = _90.xyz * (texture(sampler2D(SAMPLER_FOW_texture, SAMPLER_FOW_sampler), TEXCOORD2).www * _59.y);
    SV_Target0 = vec4(_99.x, _99.y, _99.z, _90.w);
}
