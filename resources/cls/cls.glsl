#include "native/PREFIX_SUM"
#include "../draw_indirect.glsl"
#include "../compute_indirect.glsl"

layout(std430, binding = 0) buffer CLS_FragmentsPerClusterBuffer {
  uint fragments_per_cluster[];
};

layout(std430, binding = 1) buffer CLS_OffsetBuffer {
  uint offsets[PASS_1_THREADS];
};

layout(std430, binding = 2) buffer CLS_ActiveClusterBuffer {
  uint active_cluster_indices[];
};

layout(std430, binding = 3) buffer CLS_DrawCommandBuffer {
  DrawCommand draw_command;
};

layout(std430, binding = 4) buffer CLS_ComputeCommandBuffer {
  ComputeCommand compute_command;
};

layout(std430, binding = 5) buffer CLS_LightBuffer {
  vec4 light_xyzr[];
};

layout(std430, binding = 6) buffer CLS_LightCountBuffer {
  uint light_counts[];
};
