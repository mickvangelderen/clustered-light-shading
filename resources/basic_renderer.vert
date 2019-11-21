#include "native/RENDER_TECHNIQUE"

#include "common.glsl"
#include "light_buffer.glsl"
#include "instance_matrices_buffer.glsl"
#if defined(RENDER_TECHNIQUE_CLUSTERED)
#include "cls/cluster_space_buffer.glsl"
#endif

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_BIN_IN_OBJ_LOC) in vec3 vs_bin_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_INSTANCE_INDEX_LOC) in uint vs_instance_index;

invariant gl_Position;

out vec3 fs_pos_in_lgt;
out vec3 fs_nor_in_lgt;
out vec3 fs_bin_in_lgt;
out vec3 fs_tan_in_lgt;
out vec2 fs_pos_in_tex;
#if defined(RENDER_TECHNIQUE_CLUSTERED)
out vec3 fs_pos_in_clu_cam;
#endif

void main() {
  InstanceMatrices m = instance_matrices_buffer[vs_instance_index];

  vec4 pos_in_obj = to_homogeneous(vs_pos_in_obj);
  gl_Position = m.obj_to_ren_clp * pos_in_obj;
  fs_pos_in_lgt = mat4x3(m.obj_to_lgt) * pos_in_obj;
  fs_nor_in_lgt = normalize(mat3(m.obj_to_lgt_inv_tra) * vs_nor_in_obj);
  fs_bin_in_lgt = normalize(mat3(m.obj_to_lgt) * vs_bin_in_obj);
  fs_tan_in_lgt = normalize(mat3(m.obj_to_lgt) * vs_tan_in_obj);
  // NOTE(mickvangelderen): TOO LAZY TO CHANGE IMAGE ORIGIN.
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
#if defined(RENDER_TECHNIQUE_CLUSTERED)
  fs_pos_in_clu_cam = mat4x3(m.obj_to_clu_cam) * pos_in_obj;
#endif
}
