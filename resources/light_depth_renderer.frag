#include "light_buffer.glsl"

#if !defined(BASIC_PASS)
#error BASIC_PASS is undefined.
#endif

layout(binding = NORMAL_SAMPLER_BINDING) uniform sampler2D normal_sampler;
// layout(binding = EMISSIVE_SAMPLER_BINDING) uniform sampler2D emissive_sampler;
// layout(binding = AMBIENT_SAMPLER_BINDING) uniform sampler2D ambient_sampler;
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;
// layout(binding = SPECULAR_SAMPLER_BINDING) uniform sampler2D specular_sampler;

in vec3 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec3 fs_bin_in_lgt;
in vec3 fs_tan_in_lgt;
in vec2 fs_pos_in_tex;

layout(location = 0) out float frag_distance;
layout(location = 1) out vec3 frag_nor;
layout(location = 2) out vec3 frag_tint;

vec3 sample_nor_in_tan(vec2 pos_in_tex) {
  vec2 xy = texture(normal_sampler, pos_in_tex).xy * 2.0 - vec2(1.0);
  float z = sqrt(max(0.0, 1.0 - dot(xy, xy)));
  return vec3(xy, z);
}

void main() {
  vec3 frag_pos_in_lgt = fs_pos_in_lgt;
  vec3 frag_geo_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec3 frag_geo_bin_in_lgt = normalize(fs_bin_in_lgt);
  vec3 frag_geo_tan_in_lgt = normalize(fs_tan_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;
  vec3 frag_nor_in_tan = sample_nor_in_tan(frag_pos_in_tex);

  // vec4 ka = texture(ambient_sampler, frag_pos_in_tex);
  // vec4 ke = texture(emissive_sampler, frag_pos_in_tex);
  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);
  // vec4 ks = texture(specular_sampler, frag_pos_in_tex);

#if BASIC_PASS == BASIC_PASS_MASKED
  if (kd.a < 0.5) {
    discard;
  }
#endif

  mat3 tbn = mat3(frag_geo_tan_in_lgt, frag_geo_bin_in_lgt, frag_geo_nor_in_lgt);
  vec3 frag_nor_in_lgt = normalize(tbn * frag_nor_in_tan);

  PointLight light = light_buffer.point_lights[0];

  frag_distance = distance(light.position, frag_pos_in_lgt) / light.r1;
  frag_nor = frag_nor_in_lgt;
  frag_tint = light.tint * kd.rgb;
}
