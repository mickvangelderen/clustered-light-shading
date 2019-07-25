#include "common.glsl"
#include "cluster_renderer.glsl"

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

out vec2 fs_pos_in_tex;
flat out uvec4 fs_indices;

void main() {
  uint cluster_index = active_cluster_indices[gl_InstanceID];
  uvec3 idx_in_cls = index_1_to_3(cluster_index, cluster_dims);
  vec3 pos_in_cls = vec3(idx_in_cls) + vs_pos_in_obj;

  gl_Position = cls_to_clp * to_homogeneous(pos_in_cls);
  fs_pos_in_tex = vs_pos_in_tex;
  fs_indices = uvec4(idx_in_cls, cluster_index);
}
