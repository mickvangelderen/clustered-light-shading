struct ProfilingCluster {
  uint active_cluster_count;
  uint light_indices_count;
  uint _pad[254];
  uint fragments_per_cluster_hist[256];
  uint lights_per_cluster_hist[256];
  uint lights_per_fragment_hist[256];
};

layout(std430, binding = PROFILING_CLUSTER_BUFFER_BINDING) buffer ProfilingClusterBuffer {
  ProfilingCluster profiling_cluster_buffer;
};

