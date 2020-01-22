#include "light_buffer.glsl"

#if !defined(BASIC_PASS)
#error BASIC_PASS is undefined.
#endif

#if BASIC_PASS == BASIC_PASS_MASKED
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;

in vec2 fs_pos_in_tex;
#endif

in vec4 fs_pos_in_wld;

layout(location = 0) out float frag_distance;

void main() {
#if BASIC_PASS == BASIC_PASS_MASKED
  vec2 frag_pos_in_tex = fs_pos_in_tex;

  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);

  if (kd.a < 0.5) {
    discard;
  }
#endif
  PointLight light = light_buffer.point_lights[0];
  frag_distance = (distance(light.position, fs_pos_in_wld.xyz/fs_pos_in_wld.w) - light.r0)/(light.r1 - light.r0);
  // frag_distance = gl_FragCoord.x/512.0 + gl_FragCoord.y/512.0;
}
