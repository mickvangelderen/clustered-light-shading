#include "../common.glsl"
#include "cluster_space_buffer.glsl"
#include "active_cluster_cluster_indices_buffer.glsl"

layout(location = CLU_CAM_TO_REN_CLP_LOC) uniform mat4 clu_cam_to_ren_clp;
layout(location = VISIBLE_ONLY_LOC) uniform uint visible_only;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;


out vec2 fs_pos_in_tex;
flat out uvec3 fs_idx_in_cls;
flat out uint fs_cluster_index;
flat out uint fs_maybe_active_cluster_index;

void main() {
  uint cluster_index;
  uint maybe_active_cluster_index;
  if (visible_only != 0) {
    uint active_cluster_index = gl_InstanceID;
    cluster_index = active_cluster_cluster_indices[active_cluster_index];
    maybe_active_cluster_index = active_cluster_index + 1;
  } else {
    cluster_index = gl_InstanceID;
    maybe_active_cluster_index = 0;
  }
  uvec3 idx_in_cls = index_1_to_3(cluster_index, cluster_space.dimensions);
  vec3 pos_in_clu_cam = cluster_clp_to_cam(vec3(idx_in_cls) + vs_pos_in_obj);
  gl_Position = clu_cam_to_ren_clp * to_homogeneous(pos_in_clu_cam);

  fs_pos_in_tex = vs_pos_in_tex;
  fs_idx_in_cls = idx_in_cls;
  fs_cluster_index = cluster_index;
  fs_maybe_active_cluster_index = maybe_active_cluster_index;
}
