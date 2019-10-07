struct ProfilingCluster {
  uint active_cluster_count;
  uint light_indices_count;
  uint shade_count;
  uint _pad;
  uint fragments_per_cluster_hist[32];
  uint lights_per_cluster_hist[32];
  uint lights_per_fragment_hist[32];
};

layout(std430, binding = PROFILING_CLUSTER_BUFFER_BINDING) buffer ProfilingClusterBuffer {
  ProfilingCluster profiling_cluster_buffer;
};

