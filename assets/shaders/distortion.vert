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

struct UniformsVertex {
  float PARTICLE_DEPTH_PUSH_PULL;
  vec4 TEXTURE_INFO;
};

layout(set = 3, binding = 0) uniform UniformsVertex uniforms_vertext;

// - ATTR0 (location 0, vec4):
//   - 顶点的世界坐标位置
//   - xyz 分量用于位置，与相机位置计算深度偏移
//   - 用于 PARTICLE_DEPTH_PUSH_PULL 效果，使粒子朝向或远离相机移动
// - ATTR3 (location 3, vec4):
//   - 颜色数据
//   - 在代码中重排为 zyxw 后输出到 TEXCOORD0
//   - 传递给片段着色器用于粒子着色
// - ATTR8 (location 8, vec3):
//   - 纹理图集相关数据
//   - x, y: 纹理在图集中的网格坐标
//   - z: 纹理索引/ID
//   - 结合 TEXTURE_INFO 计算实际的纹理UV坐标
// - ATTR9 (location 9, vec2):
//   - 粒子的纹理坐标 (UV)
//   - 直接传递到 TEXCOORD1.zw
//   - 可能用于旋转、缩放等粒子特有的变换

// 输出到片段着色器：
// - TEXCOORD0: 重排后的颜色数据
// - TEXCOORD1: 计算后的纹理图集UV + 原始UV坐标
// - TEXCOORD2: 屏幕空间坐标 (用于可能的屏幕空间效果)

layout(location = 0) in vec4 ATTR0;
layout(location = 3) in vec4 ATTR3;
layout(location = 8) in vec3 ATTR8;
layout(location = 9) in vec2 ATTR9;
layout(location = 0) out vec4 TEXCOORD0;
layout(location = 1) out vec4 TEXCOORD1;
layout(location = 2) out vec2 TEXCOORD2;

void main() {
  vec4 _58 =
      camera_view.clip_from_world *
      vec4(ATTR0.xyz + (normalize(ATTR0.xyz - camera_view.world_position) *
                        uniforms_vertext.PARTICLE_DEPTH_PUSH_PULL),
           1.0);
  float _63 = floor(ATTR8.z);
  float _66 = floor(_63 * uniforms_vertext.TEXTURE_INFO.y);
  vec2 _85 = ((_58.xy / vec2(_58.w)) * vec2(0.5)) + vec2(0.5);
  _85.y = 1.0 - _85.y;
  gl_Position = _58;      // done
  TEXCOORD0 = ATTR3.zyxw; //
  TEXCOORD1 = vec4(ATTR9, ATTR9);
  TEXCOORD2 = _85;
}
