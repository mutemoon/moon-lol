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
    mat4x3 BONES[68];
    float PARTICLE_DEPTH_PUSH_PULL;
    vec4 vFresnel;
    mat4x3 vParticleUVTransform;
    mat4x3 vParticleUVTransformMult;
    vec4 kColorFactor;
};

struct Mesh {
    mat3x4 world_from_local;
    mat3x4 previous_world_from_local;
    mat2x4 local_from_world_transpose_a;
    float local_from_world_transpose_b;
    uint flags;
    vec2 lightmap_uv_rect;
    uint first_vertex_index;
    uint current_skin_index;
};

layout(set = 3, binding = 0) uniform UniformsVertex uniforms_vertext;

#ifdef PER_OBJECT_BUFFER_BATCH_SIZE
layout(set = 2, binding = 0) uniform Mesh Meshes[#{PER_OBJECT_BUFFER_BATCH_SIZE}];
#else
layout(set = 2, binding = 0) readonly buffer _Meshes {
    Mesh Meshes[];
};
#endif // PER_OBJECT_BUFFER_BATCH_SIZE

layout(set = 1, binding = 1) readonly buffer JointMatrices { mat4 joint_matrices[]; };

layout(location = 0) in vec3 ATTR0;
layout(location = 1) in vec4 ATTR1;
layout(location = 2) in vec3 ATTR2;
layout(location = 7) in uvec4 ATTR7;
layout(location = 8) in vec2 ATTR8;
layout(location = 0) out vec4 TEXCOORD0;
layout(location = 1) out vec3 TEXCOORD1;
layout(location = 2) out vec2 TEXCOORD2;
layout(location = 3) out vec3 COLOR0;

void main()
{
    vec3 _63 = vec3(0.0);
    vec3 _66 = vec3(0.0);
    _63 = vec3(0.0);
    _66 = vec3(0.0);
    for (int _68 = 0; _68 < 4; )
    {
        uint _72 = uint(_68);
        _63 += ((uniforms_vertext.BONES[ATTR7[_72]] * vec4(ATTR0, 1.0)) * ATTR1[_72]);
        _66 += ((uniforms_vertext.BONES[ATTR7[_72]] * vec4(ATTR2, 0.0)) * ATTR1[_72]);
        _68++;
        continue;
    }
    vec3 _99 = _63 + (normalize(_63 - camera_view.world_position) * uniforms_vertext.PARTICLE_DEPTH_PUSH_PULL);
    vec4 _106 = camera_view.clip_from_world * vec4(_99, 1.0);
    gl_Position = _106;
    TEXCOORD0 = uniforms_vertext.kColorFactor;
    TEXCOORD1 = vec3((uniforms_vertext.vParticleUVTransform * vec4(ATTR8, 1.0, 0.0)).xy, _106.w);
    TEXCOORD2 = ((_99.xz * uniforms_vertext.FOG_OF_WAR_PARAMS.xy) + uniforms_vertext.FOG_OF_WAR_PARAMS.zw).xy;
    COLOR0 = uniforms_vertext.vFresnel.xyz * (1.0 - pow(clamp(dot(-normalize(_99 - camera_view.world_position), normalize(_66)), 0.0, 1.0), uniforms_vertext.vFresnel.w));
}
