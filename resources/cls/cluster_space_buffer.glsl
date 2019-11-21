#include "../frustum.glsl"
#include "native/CLUSTERED_LIGHT_SHADING"

layout(std140,
       binding = CLUSTER_SPACE_BUFFER_BINDING) uniform ClusterSpaceBuffer {
  uvec3 dimensions;
  uint cluster_count;

  // Perspective light counting/assignment.
  Frustum frustum;

  float cam_to_clp_ax;
  float cam_to_clp_bx;
  float cam_to_clp_ay;
  float cam_to_clp_by;
  float cam_to_clp_az;
  float cam_to_clp_bz;
  float _pad0;
  float _pad1;

  float clp_to_cam_ax;
  float clp_to_cam_bx;
  float clp_to_cam_ay;
  float clp_to_cam_by;
  float clp_to_cam_az;
  float clp_to_cam_bz;
  float _pad2;
  float _pad3;
}
cluster_space;

vec3 cluster_cam_to_clp(vec3 pos_in_cam) {
  float ax = cluster_space.cam_to_clp_ax;
  float bx = cluster_space.cam_to_clp_bx;
  float ay = cluster_space.cam_to_clp_ay;
  float by = cluster_space.cam_to_clp_by;
  float az = cluster_space.cam_to_clp_az;
  float bz = cluster_space.cam_to_clp_bz;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  float frac_1_neg_z_cam = -1.0 / pos_in_cam.z;
  float x_cls = frac_1_neg_z_cam * (ax * pos_in_cam.x) + bx;
  float y_cls = frac_1_neg_z_cam * (ay * pos_in_cam.y) + by;
  float z_cls = float(cluster_space.dimensions.z) - log(pos_in_cam.z * az) * bz;
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  float x_cls = ax * pos_in_cam.x + bx;
  float y_cls = ay * pos_in_cam.y + by;
  float z_cls = az * pos_in_cam.z + bz;
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
  return vec3(x_cls, y_cls, z_cls);
}

vec3 cluster_clp_to_cam(vec3 pos_in_clp) {
  float ax = cluster_space.clp_to_cam_ax;
  float bx = cluster_space.clp_to_cam_bx;
  float ay = cluster_space.clp_to_cam_ay;
  float by = cluster_space.clp_to_cam_by;
  float az = cluster_space.clp_to_cam_az;
  float bz = cluster_space.clp_to_cam_bz;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  float z_cam = az * pow(bz, float(cluster_space.dimensions.z) - pos_in_clp.z);
  float x_cam = -z_cam * (ax * pos_in_clp.x + bx);
  float y_cam = -z_cam * (ay * pos_in_clp.y + by);
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  float x_cam = ax * pos_in_clp.x + bx;
  float y_cam = ay * pos_in_clp.y + by;
  float z_cam = az * pos_in_clp.z + bz;
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
  return vec3(x_cam, y_cam, z_cam);
}
