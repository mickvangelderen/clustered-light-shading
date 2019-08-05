#include "native/PREFIX_SUM"
#include "native/CLUSTERED_LIGHT_SHADING"
#include "../draw_indirect.glsl"
#include "../compute_indirect.glsl"

layout(std430, binding = 0) buffer ClusterFragmentCountsBuffer {
  uint cluster_fragment_counts[];
};

layout(std430, binding = 1) buffer ActiveClusterIndicesBuffer {
  uint active_cluster_indices[];
};

layout(std430, binding = 2) buffer ActiveClusterLightCountsBuffer {
  uint active_cluster_light_counts[];
};

layout(std430, binding = 3) buffer ActiveClusterLightOffsetsBuffer {
  uint active_cluster_light_offsets[];
};

layout(std430, binding = 4) buffer LightXYZRBuffer {
  vec4 light_xyzr[];
};

layout(std430, binding = 5) buffer OffsetBuffer {
  uint offsets[];
};

layout(std430, binding = 6) buffer DrawCommandBuffer {
  DrawCommand draw_command;
};

#define COMPUTE_COMMAND_INDEX_ACTIVE_CLUSTER_COUNT 0
#define COMPUTE_COMMAND_INDEX_PREFIX_SUM_LIGHT_COUNTS 1
layout(std430, binding = 7) buffer ComputeCommandBuffer {
  ComputeCommand compute_commands[];
};

layout(std430, binding = 8) buffer LightIndicesBuffer {
  uint light_indices[];
};
