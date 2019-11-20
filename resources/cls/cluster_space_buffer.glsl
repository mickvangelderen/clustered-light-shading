#include "../frustum.glsl"
#include "../lerp_coeffs.glsl"
#include "native/CLUSTERED_LIGHT_SHADING"

layout(std140,
       binding = CLUSTER_SPACE_BUFFER_BINDING) uniform ClusterSpaceBuffer {
  uvec3 dimensions;
  uint cluster_count;

  // Perspective light counting/assignment.
  Frustum frustum;

  // Orthographic light counting/assignment. Might be able to just use the frustum.
  mat4 clu_clp_to_clu_cam;
}
cluster_space;
