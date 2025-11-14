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


vec3 _35 = vec3(0.0);

struct UniformsVertex
{
    vec4 FOG_OF_WAR_PARAMS;
    vec4 FOG_OF_WAR_ALWAYS_BELOW_Y;
    vec4 FOW_HEIGHT_FADE;
    mat4 DECAL_WORLD_MATRIX;
    mat4 DECAL_WORLD_TO_UV_MATRIX;
    vec4 DECAL_PROJECTION_Y_RANGE;
};

layout(set = 3, binding = 0) uniform UniformsVertex uniforms_vertext;

layout(location = 0) in vec3 ATTR0;
layout(location = 0) out vec3 TEXCOORD0;
layout(location = 1) out vec2 TEXCOORD2;

void main()
{
    vec4 _9 = uniforms_vertext.DECAL_WORLD_MATRIX * vec4(ATTR0, 1.0);
    vec4 _50 = uniforms_vertext.DECAL_WORLD_TO_UV_MATRIX * _9;
    vec2 _51 = _50.xz;
    _51.y = 1.0 - _50.z;
    vec3 _55 = vec3(_51.x, _51.y, _35.z);
    float _59 = abs(ATTR0.y - uniforms_vertext.DECAL_PROJECTION_Y_RANGE.x);
    vec3 _73 = vec3(0.0);
    if (_59 <= uniforms_vertext.DECAL_PROJECTION_Y_RANGE.y)
    {
        vec3 _66 = _55;
        _66.z = 1.0;
        _73 = _66;
    }
    else
    {
        vec3 _72 = _55;
        _72.z = 1.0 - ((_59 - uniforms_vertext.DECAL_PROJECTION_Y_RANGE.y) / uniforms_vertext.DECAL_PROJECTION_Y_RANGE.z);
        _73 = _72;
    }
    gl_Position = camera_view.clip_from_world * _9;
    TEXCOORD0 = _73;
    TEXCOORD2 = ((_9.xz * uniforms_vertext.FOG_OF_WAR_PARAMS.xy) + uniforms_vertext.FOG_OF_WAR_PARAMS.zw).xy;
}
