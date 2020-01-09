#include "native/PROFILING"
#include "native/DEPTH_PREPASS"

#include "../common.glsl"
#include "../light_buffer.glsl"

#include "cluster_space_buffer.glsl"
#include "cluster_fragment_counts_buffer.glsl"

#if !defined(PROFILING_TIME_SENSITIVE)
#error PROFILING_TIME_SENSITIVE is not defined.
#endif

#if !defined(BASIC_PASS)
#error BASIC_PASS is not defined.
#endif

#if !defined(BASIC_PASS_OPAQUE)
#error BASIC_PASS_OPAQUE is not defined.
#endif

#if !defined(BASIC_PASS_MASKED)
#error BASIC_PASS_MASKED is not defined.
#endif

#if !defined(DEPTH_PREPASS)
#error DEPTH_PREPASS is not defined.
#endif

#if BASIC_PASS == BASIC_PASS_OPAQUE
// NOTE(mickvangelderen) Should be the default, unless using side
// effects like the atomic writes for atemporal profiling where we have
// to force this behaviour.
layout(early_fragment_tests) in;
#endif

#if BASIC_PASS == BASIC_PASS_MASKED
// NOTE(mickvangelderen) Can only force this on when we disable depth
// writes because of how early-z is implemented.
#if DEPTH_PREPASS
layout(early_fragment_tests) in;
#endif
#endif

#if BASIC_PASS == BASIC_PASS_TRANSPARENT
// NOTE(mickvangelderen) For transparent shading we should always
// have depth writes disabled?
layout(early_fragment_tests) in;
#endif


#if BASIC_PASS == BASIC_PASS_MASKED || BASIC_PASS == BASIC_PASS_TRANSPARENT
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;
in vec2 fs_pos_in_tex;
#endif

in vec3 fs_pos_in_clu_cam;

// layout(location = 0) out vec4 frag_color;

void main() {
#if BASIC_PASS == BASIC_PASS_MASKED
  vec4 kd = texture(diffuse_sampler, fs_pos_in_tex);
  if (kd.a < 0.5) {
    discard;
  }
#elif BASIC_PASS == BASIC_PASS_TRANSPARENT
  vec4 kd = texture(diffuse_sampler, fs_pos_in_tex);
  if (kd.a < 0.001) {
    discard;
  }
#endif

  vec3 pos_in_cls = cluster_cam_to_clp(fs_pos_in_clu_cam);

  if (all(greaterThanEqual(pos_in_cls, vec3(0.0))) &&
      all(lessThan(pos_in_cls, vec3(cluster_space.dimensions)))) {
    uvec3 idx_in_cls = uvec3(pos_in_cls);
    uint cluster_index = index_3_to_1(idx_in_cls, cluster_space.dimensions);
    atomicAdd(cluster_fragment_counts[cluster_index], 1);
  }
}
