#include "common.glsl"
#include "cluster_renderer.glsl"

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

out vec2 fs_pos_in_tex;
flat out uvec3 fs_idx_in_cls;
flat out uint fs_cluster_index;
flat out uint fs_active_cluster_index;

void main() {
  uint active_cluster_index = gl_InstanceID;
  uint cluster_index = active_cluster_indices[active_cluster_index];
  uvec3 idx_in_cls = index_1_to_3(cluster_index, cluster_dims);
  vec3 pos_in_cls = vec3(idx_in_cls) + vs_pos_in_obj;
  vec4 almost_pos_in_ccam = cclp_to_ccam * to_homogeneous(pos_in_cls);
  vec3 pos_in_ccam = vec3(almost_pos_in_ccam.xy / vec2(almost_pos_in_ccam.w), pos_in_cls.z);

  gl_Position = ccam_to_clp * to_homogeneous(vec3(pos_in_ccam.xy, pos_in_cls.z*10.0));
  fs_pos_in_tex = vs_pos_in_tex;
  fs_idx_in_cls = idx_in_cls;
  fs_cluster_index = cluster_index;
  fs_active_cluster_index = active_cluster_index;
}
