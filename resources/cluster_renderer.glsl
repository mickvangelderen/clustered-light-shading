layout(location = 0) uniform mat4 cls_to_clp;
layout(location = 1) uniform uvec3 cluster_dims;
layout(location = 2) uniform uint pass;

layout(binding = 0) buffer FragmentsPerClusterBuffer {
  uint fragments_per_cluster[];
};

layout(binding = 1) buffer ActiveClusterBuffer {
  uint active_cluster[];
};
