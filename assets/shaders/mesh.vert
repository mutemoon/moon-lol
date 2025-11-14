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


vec4 _44 = vec4(0.0);

struct UniformsVertex
{
    vec4 FOG_OF_WAR_PARAMS;
    vec4 FOG_OF_WAR_ALWAYS_BELOW_Y;
    vec4 FOW_HEIGHT_FADE;
    mat4 mWorld;
    float PARTICLE_DEPTH_PUSH_PULL;
    vec4 vFresnel;
    mat4x3 vParticleUVTransform;
    mat4x3 vParticleUVTransformMult;
    vec4 kColorFactor;
};

layout(set = 3, binding = 0) uniform UniformsVertex uniforms_vertext;

layout(location = 0) in vec3 ATTR0;
layout(location = 2) in vec3 ATTR2;
layout(location = 8) in vec2 ATTR8;
layout(location = 0) out vec4 TEXCOORD0;
layout(location = 1) out vec3 TEXCOORD1;
layout(location = 2) out vec3 TEXCOORD2;
layout(location = 3) out vec3 TEXCOORD6;

void main()
{
    vec4 _56 = uniforms_vertext.mWorld * vec4(ATTR0, 1.0);
    vec3 _57 = _56.xyz;
    vec3 _61 = normalize(_57 - camera_view.world_position);
    vec3 _65 = _57 + (_61 * uniforms_vertext.PARTICLE_DEPTH_PUSH_PULL);
    vec4 _69 = camera_view.clip_from_world * vec4(_65.x, _65.y, _65.z, _56.w);
    vec2 _104 = (_56.xz * uniforms_vertext.FOG_OF_WAR_PARAMS.xy) + uniforms_vertext.FOG_OF_WAR_PARAMS.zw;
    vec4 _105 = vec4(_104.x, _104.y, _44.z, _44.w);
    _105.w = clamp((_56.y * uniforms_vertext.FOG_OF_WAR_ALWAYS_BELOW_Y.z) + uniforms_vertext.FOG_OF_WAR_ALWAYS_BELOW_Y.w, 0.0, 1.0);
    gl_Position = _69;
    TEXCOORD0 = uniforms_vertext.kColorFactor;
    TEXCOORD1 = vec3((uniforms_vertext.vParticleUVTransform * vec4(ATTR8, 1.0, 0.0)).xy, _69.w);
    TEXCOORD2 = _105.xyw;
    TEXCOORD6 = uniforms_vertext.vFresnel.xyz * (1.0 - pow(clamp(dot(-_61, normalize(mat3(uniforms_vertext.mWorld[0].xyz, uniforms_vertext.mWorld[1].xyz, uniforms_vertext.mWorld[2].xyz) * ATTR2)), 0.0, 1.0), uniforms_vertext.vFresnel.w));
}
