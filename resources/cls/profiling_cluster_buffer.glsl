struct ProfilingClusterBuffer {
  uint active_cluster_count;
  uint light_indices_count;
  uint shade_count;
  uint _pad;
  uint frag_count_hist[32];
  uint light_count_hist[32];
};

layout(std430, binding = PROFILING_CLUSTER_BUFFER_BINDING) buffer ProfilingClusterBuffer {
  ProfilingClusterBuffer profiling_cluster_buffer;
};
