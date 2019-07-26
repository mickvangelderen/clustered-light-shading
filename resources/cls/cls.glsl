#include "native/PREFIX_SUM"
#include "../draw_indirect.glsl"
#include "../compute_indirect.glsl"

layout(std430, binding = 0) buffer ClusterFragmentCountsBuffer {
  uint cluster_fragment_counts[];
};

struct ClusterMeta {
  uint light_index_count;
  uint light_index_offset;
};

layout(std430, binding = 1) buffer ClusterMetaBuffer {
  ClusterMeta cluster_metas[];
};

layout(std430, binding = 2) buffer ActiveClusterIndicesBuffer {
  uint active_cluster_indices[];
};

layout(std430, binding = 3) buffer ActiveClusterLightCountsBuffer {
  uint active_cluster_light_counts[];
};

layout(std430, binding = 4) buffer ActiveClusterLightOffsetsBuffer {
  uint active_cluster_light_offsets[];
};

layout(std430, binding = 5) buffer LightXYZRBuffer {
  vec4 light_xyzr[];
};

layout(std430, binding = 6) buffer OffsetBuffer {
  uint offsets[];
};

layout(std430, binding = 7) buffer DrawCommandBuffer {
  DrawCommand draw_command;
};

layout(std430, binding = 8) buffer ComputeCommandBuffer {
  ComputeCommand compute_command;
};

