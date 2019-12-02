#include "native/SAMPLE_COUNT"

#if !defined(BASIC_PASS)
#error BASIC_PASS is undefined.
#endif

#if !defined(SAMPLE_COUNT)
#error SAMPLE_COUNT is undefined.
#endif

#if SAMPLE_COUNT > 0
layout(location = 1) out float frag_depth;
#endif

#if BASIC_PASS == BASIC_PASS_MASKED
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;

in vec2 fs_pos_in_tex;
#endif

void main() {
#if SAMPLE_COUNT > 0
  // frag_depth = interpolateAtCentroid(gl_FragCoord).z;
  frag_depth = gl_FragCoord.z;
#endif

#if BASIC_PASS == BASIC_PASS_MASKED
  vec2 frag_pos_in_tex = fs_pos_in_tex;

  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);

  if (kd.a < 0.5) {
    discard;
  }
#endif
}
