#include "../frustum.glsl"
#include "../lerp_coeffs.glsl"
#include "native/CLUSTERED_LIGHT_SHADING"

layout(std140,
       binding = CLUSTER_SPACE_BUFFER_BINDING) uniform ClusterSpaceBuffer {
  uvec3 dimensions;
  uint cluster_count;
  Frustum frustum;
  LerpCoeffs cam_to_cls_coeffs;
  LerpCoeffs cls_to_cam_coeffs;
  mat4 wld_to_cam;
  mat4 cam_to_wld;
}
cluster_space;

vec3 cluster_cam_to_cls(vec3 pos_in_cam) {
  Frustum f = cluster_space.frustum;
  LerpCoeffs c = cluster_space.cam_to_cls_coeffs;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  float frac_1_neg_z_cam = -1.0 / pos_in_cam.z;
  float x_cls = frac_1_neg_z_cam * (c.xa * pos_in_cam.x) + c.xb;
  float y_cls = frac_1_neg_z_cam * (c.ya * pos_in_cam.y) + c.yb;
  float d = -f.z1 * (f.x1 - f.x0) / float(cluster_space.dimensions.x);
  float z_cls = float(cluster_space.dimensions.z) -
                log(pos_in_cam.z / f.z1) / log(1.0 + d);
  return vec3(x_cls, y_cls, z_cls);
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return vec3(                    //
      c.xa * pos_in_cam.x + c.xb, //
      c.ya * pos_in_cam.y + c.yb, //
      c.za * pos_in_cam.z + c.zb  //
  );
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
}

vec3 cluster_cls_to_cam(vec3 pos_in_cls) {
  Frustum f = cluster_space.frustum;
  LerpCoeffs c = cluster_space.cls_to_cam_coeffs;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  // NOTE: I expect dx to be roughly equal to dy.
  float d = -f.z1 * (f.x1 - f.x0) / float(cluster_space.dimensions.x);
  float z_cam =
      f.z1 * pow(1.0 + d, float(cluster_space.dimensions.z) - pos_in_cls.z);
  float x_cam = -z_cam * (c.xa * pos_in_cls.x + c.xb);
  float y_cam = -z_cam * (c.ya * pos_in_cls.y + c.yb);
  return vec3(x_cam, y_cam, z_cam);
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return vec3(                    //
      c.xa * pos_in_cls.x + c.xb, //
      c.ya * pos_in_cls.y + c.yb, //
      c.za * pos_in_cls.z + c.zb  //
  );
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
}
