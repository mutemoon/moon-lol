#version 450

struct UniformsPixel
{
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
layout(location = 2) in vec2 TEXCOORD2;
layout(location = 3) in vec3 COLOR0;
layout(location = 0) out vec4 SV_Target0;

void main()
{
    vec4 _56 = texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy);
    vec4 _62 = texture(sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler), uniforms_pixel.COLOR_LOOKUP_UV);
    vec4 _63 = _56 * _62;
    vec4 _64 = TEXCOORD0 * _63;
    vec3 _65 = _64.xyz;
    vec4 _71 = texture(sampler2D(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_texture, CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip_sampler), vec2(dot(_65, vec3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875)), 0.5));
    vec3 _78 = vec3(0.0);
    if (_71.w > 0.0)
    {
        _78 = _71.xyz;
    }
    else
    {
        _78 = _65;
    }
    vec4 _83 = vec4(_78, _64.w);
    vec3 _95 = clamp(((_83.xyz + (COLOR0 * _63.w)).xyz * texture(sampler2D(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_texture, CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip_sampler), TEXCOORD2).w).xyz, vec3(0.0), vec3(1.0));
    SV_Target0 = vec4(_95.x, _95.y, _95.z, _83.w);
}
