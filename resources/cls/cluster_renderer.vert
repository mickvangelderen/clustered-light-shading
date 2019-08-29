#include "../common.glsl"
#include "cluster_space_buffer.glsl"
#include "active_cluster_indices_buffer.glsl"

layout(location = WLD_TO_CLP_LOC) uniform mat4 wld_to_clp;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

out vec2 fs_pos_in_tex;
flat out uvec3 fs_idx_in_cls;
flat out uint fs_cluster_index;
flat out uint fs_active_cluster_index;

void main() {
  uint active_cluster_index = gl_InstanceID;
  uint cluster_index = active_cluster_indices[active_cluster_index];
  uvec3 idx_in_cls = index_1_to_3(cluster_index, cluster_space.dimensions);
  vec3 pos_in_cls = vec3(idx_in_cls) + vs_pos_in_obj;

  vec3 pos_in_ccam = cluster_cls_to_cam(pos_in_cls);

  gl_Position = wld_to_clp * cluster_space.cam_to_wld * to_homogeneous(pos_in_ccam);
  fs_pos_in_tex = vs_pos_in_tex;
  fs_idx_in_cls = idx_in_cls;
  fs_cluster_index = cluster_index;
  fs_active_cluster_index = active_cluster_index;
}
