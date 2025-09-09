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
    float PARTICLE_DEPTH_PUSH_PULL;
    vec4 TEXTURE_INFO;
};

layout(set = 2, binding = 0) uniform UniformsVertex uniforms_vertext;

layout(location = 0) in vec3 ATTR0;
layout(location = 3) in vec4 ATTR3;
layout(location = 8) in vec4 ATTR8;
layout(location = 9) in vec2 ATTR9;
layout(location = 0) out vec4 TEXCOORD0;
layout(location = 1) out vec4 TEXCOORD1;
layout(location = 2) out vec2 TEXCOORD2;

void main()
{
    float _62 = floor(ATTR8.z);
    float _65 = floor(_62 * uniforms_vertext.TEXTURE_INFO.y);
    vec2 _76 = vec2((ATTR8.x + (_62 - (_65 * uniforms_vertext.TEXTURE_INFO.x))) * uniforms_vertext.TEXTURE_INFO.y, (ATTR8.y + _65) * uniforms_vertext.TEXTURE_INFO.z);
    gl_Position = camera_view.clip_from_world * vec4(ATTR0 + (normalize(ATTR0 - camera_view.world_position) * uniforms_vertext.PARTICLE_DEPTH_PUSH_PULL), 1.0);
    TEXCOORD0 = ATTR3.zyxw;
    TEXCOORD1 = vec4(_76.x, _76.y, ATTR9.x, ATTR9.y);
    TEXCOORD2 = ((ATTR0.xz * uniforms_vertext.FOG_OF_WAR_PARAMS.xy) + uniforms_vertext.FOG_OF_WAR_PARAMS.zw).xy;
}
