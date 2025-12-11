#version 410

#ifdef ALPHA_TEST
struct UniformsPixel
{
    float AlphaTestReferenceValue;
};

uniform UniformsPixel _UniformsPixel;
#endif

uniform sampler2D TEXTURE;
uniform sampler2D PARTICLE_COLOR_TEXTURE;
uniform sampler2D CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip;
uniform sampler2D CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip;

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec4 TEXCOORD1;
layout(location = 2) in vec2 TEXCOORD2;
layout(location = 0) out vec4 SV_Target0;

void main()
{
    vec4 _43 = texture(TEXTURE, TEXCOORD1.xy);
    vec4 _47 = texture(PARTICLE_COLOR_TEXTURE, TEXCOORD1.zw);
    vec4 _49 = TEXCOORD0 * (_43 * _47);
    vec3 _50 = _49.xyz;
    vec4 _56 = texture(CMB_TEX_PIXEL_COLOR_REMAP_RAMP_SMP_Clamp_No_Mip, vec2(dot(_50, vec3(0.2125999927520751953125, 0.715200006961822509765625, 0.072200000286102294921875)), 0.5));
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
    vec3 _74 = _68.xyz * texture(CMB_TEX_FOW_MAP_SMP_Clamp_No_Mip, TEXCOORD2).w;
#ifdef ALPHA_TEST
    if ((_49.w - _UniformsPixel.AlphaTestReferenceValue) < 0.0)
    {
        discard;
    }
#endif
    SV_Target0 = vec4(_74.x, _74.y, _74.z, _68.w);
}

