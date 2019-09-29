#include "native/CLUSTERED_LIGHT_SHADING"
#include "../frustum.glsl"
#include "../lerp_coeffs.glsl"

layout(std140, binding = CLUSTER_SPACE_BUFFER_BINDING) uniform ClusterSpaceBuffer {
  uvec3 dimensions;
  uint cluster_count;
  Frustum frustum;
  LerpCoeffs cam_to_clp_coeffs;
  LerpCoeffs clp_to_cam_coeffs;
  mat4 wld_to_cam;
  mat4 cam_to_wld;
} cluster_space;

vec4 cluster_cam_to_clp(vec3 pos_in_cam) {
  LerpCoeffs c = cluster_space.cam_to_clp_coeffs;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  return vec4(                                   //
      c.xa * pos_in_cam.x - c.xb * pos_in_cam.z, //
      c.ya * pos_in_cam.y - c.yb * pos_in_cam.z, //
      c.za * pos_in_cam.z + c.zb,                //
      -pos_in_cam.z                              //
  );
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return vec4(                    //
      c.xa * pos_in_cam.x + c.xb, //
      c.ya * pos_in_cam.y + c.yb, //
      c.za * pos_in_cam.z + c.zb, //
      1.0                         //
  );
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
}

vec3 cluster_clp_to_cls(vec4 pos_in_clp) {
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  return vec3(                     //
      pos_in_clp.x / pos_in_clp.w, //
      pos_in_clp.y / pos_in_clp.w, //
      pos_in_clp.z                 //
  );
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return pos_in_clp.xyz;
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
}

// NOTE: Unused
// vec3 cluster_cam_to_cls(vec3 pos_in_cam) {
//   LerpCoeffs c = cluster_space.cam_to_clp_coeffs;
// #if !defined(CLUSTERING_PROJECTION)
// #error CLUSTERING_PROJECTION is not defined.
// #endif
// #if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
//   float frac_1_neg_z_cam = -1.0 / pos_in_cam.z;
//   return vec3(                                         //
//       frac_1_neg_z_cam * (c.xa * pos_in_cam.x) + c.xb, //
//       frac_1_neg_z_cam * (c.ya * pos_in_cam.y) + c.yb, //
//       c.za * pos_in_cam.z + c.zb                       //
//   );
// #elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
//   return vec3(                    //
//       c.xa * pos_in_cam.x + c.xb, //
//       c.ya * pos_in_cam.y + c.yb, //
//       c.za * pos_in_cam.z + c.zb  //
//   );
// #else
// #error Unknown CLUSTERING_PROJECTION.
// #endif
// }

vec3 cluster_cls_to_cam(vec3 pos_in_cls) {
  LerpCoeffs c = cluster_space.clp_to_cam_coeffs;
#if !defined(CLUSTERING_PROJECTION)
#error CLUSTERING_PROJECTION is not defined.
#endif
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  float z_cam = c.za * pos_in_cls.z + c.zb;
  return vec3(                               //
      -z_cam * (c.xa * pos_in_cls.x + c.xb), //
      -z_cam * (c.ya * pos_in_cls.y + c.yb), //
      z_cam                                  //
  );
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
