#include "native/SAMPLE_COUNT"

#if SAMPLE_COUNT == 0
layout(binding = ACCUM_SAMPLER_BINDING) uniform sampler2D accum_sampler;
layout(binding = REVEAL_SAMPLER_BINDING) uniform sampler2D reveal_sampler;
#else
layout(binding = ACCUM_SAMPLER_BINDING) uniform sampler2DMS accum_sampler;
layout(binding = REVEAL_SAMPLER_BINDING) uniform sampler2DMS reveal_sampler;
#endif

layout(location = 0) out vec4 frag_color;

void main() {
  // FIXME(mickvangelderen): NOT DEALING WITH MULTI-SAMPLING. I don't know how to with OIT.
  ivec2 pos_in_tex = ivec2(gl_FragCoord.xy);
  vec4 accum = texelFetch(accum_sampler, pos_in_tex, 0);
  float reveal = texelFetch(reveal_sampler, pos_in_tex, 0).r;
  frag_color = vec4(accum.rgb / clamp(accum.a, 1e-4, 5e4), reveal);
}
