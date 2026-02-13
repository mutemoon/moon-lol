#version 450

struct UniformsPixel {
  float AlphaTestReferenceValue;
  float DistortionPower;
  vec4 APPLY_TEAM_COLOR_CORRECTION;
};

layout(set = 3, binding = 1) uniform UniformsPixel uniforms_pixel;

layout(set = 3, binding = 2) uniform texture2D TEXTURE_texture;
layout(set = 3, binding = 3) uniform sampler TEXTURE_sampler;
layout(set = 3, binding = 4) uniform texture2D PARTICLE_COLOR_TEXTURE_texture;
layout(set = 3, binding = 5) uniform sampler PARTICLE_COLOR_TEXTURE_sampler;
layout(set = 3, binding = 6) uniform texture2D NORMAL_MAP_texture;
layout(set = 3, binding = 7) uniform sampler NORMAL_MAP_sampler;
layout(set = 3, binding = 8) uniform texture2D
    CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_texture;
layout(set = 3, binding = 9) uniform sampler
    CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_sampler;

layout(location = 0) in vec4 TEXCOORD0;
layout(location = 1) in vec4 TEXCOORD1;
layout(location = 2) in vec2 TEXCOORD2;
layout(location = 0) out vec4 SV_Target0;

void main() {
  vec4 _49 = texture(
      sampler2D(PARTICLE_COLOR_TEXTURE_texture, PARTICLE_COLOR_TEXTURE_sampler),
      TEXCOORD1.zw);
  vec4 _55 =
      texture(sampler2D(NORMAL_MAP_texture, NORMAL_MAP_sampler), TEXCOORD1.xy);
  float _62 = _49.w;
  vec4 _69 =
      texture(
          sampler2D(CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_texture,
                    CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_sampler),
          TEXCOORD2 +
              ((((_55.xy - vec2(0.5)) * 2.0) * uniforms_pixel.DistortionPower) *
               _62)) *
      ((TEXCOORD0 *
        texture(sampler2D(TEXTURE_texture, TEXTURE_sampler), TEXCOORD1.xy)) *
       _49);
  _69.w = _55.w * _62;
  SV_Target0 = _69;
  //   SV_Target0 = texture(
  //       sampler2D(CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_texture,
  //                 CMB_TEX_SAMPLER_BACK_BUFFER_COPY_SMP_Clamp_No_Mip_sampler),
  //       TEXCOORD2);
}
