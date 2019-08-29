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

struct Frustum {
  float x0;
  float x1;
  float y0;
  float y1;
  float z0;
  float z1;
  float _pad0;
  float _pad1;
};

struct LerpCoeffs {
  float xa;
  float xb;
  float ya;
  float yb;
  float za;
  float zb;
  float _pad0;
  float _pad1;
};

layout(std140, binding = 9) uniform ClusterSpaceBuffer {
  uvec3 dimensions;
  uint _pad0;
  Frustum frustum;
  LerpCoeffs cam_to_clp_coeffs;
  LerpCoeffs clp_to_cam_coeffs;
  mat4 wld_to_cam;
  mat4 cam_to_wld;
} cluster_space;

vec4 cluster_cam_to_clp(vec3 pos_in_cam) {
  LerpCoeffs c = cluster_space.cam_to_clp_coeffs;
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  return vec4(
    c.xa * pos_in_cam.x - c.xb * pos_in_cam.z, //
    c.ya * pos_in_cam.y - c.yb * pos_in_cam.z, //
    c.za * pos_in_cam.z + c.zb, //
    -pos_in_cam.z //
  );
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return vec4(
    c.xa * pos_in_cam.x + c.xb, //
    c.ya * pos_in_cam.y + c.yb, //
    c.za * pos_in_cam.z + c.zb, //
    1.0 //
  );
#else
  #error Unknown CLUSTERING_PROJECTION.
#endif
}

vec3 cluster_cls_to_cam(vec3 pos_in_cls) {
  LerpCoeffs c = cluster_space.cam_to_clp_coeffs;
#if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
  float z_cam = pos_in_cls.z * c.za + c.zb;
  return vec3(
    z_cam*(c.xa * pos_in_cls.x + c.xb), //
    z_cam*(c.ya * pos_in_cls.y + c.yb), //
    z_cam //
  );
#elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
  return vec3(
    c.xa * pos_in_cls.x + c.xb, //
    c.ya * pos_in_cls.y + c.yb, //
    c.za * pos_in_cls.z + c.zb //
  );
#else
#error Unknown CLUSTERING_PROJECTION.
#endif
}

// NOTE: Template
// #if CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_PERSPECTIVE
// #elif CLUSTERING_PROJECTION == CLUSTERING_PROJECTION_ORTHOGRAPHIC
// #else
// #error Unknown CLUSTERING_PROJECTION.
// #endif
