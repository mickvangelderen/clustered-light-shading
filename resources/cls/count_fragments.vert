#include "../common.glsl"
#include "../light_buffer.glsl"
#include "../instance_matrices_buffer.glsl"
#include "cluster_space_buffer.glsl"

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
#if BASIC_PASS == BASIC_PASS_MASKED || BASIC_PASS == BASIC_PASS_TRANSPARENT
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
out vec2 fs_pos_in_tex;
#endif

layout(location = VS_INSTANCE_INDEX_LOC) in uint vs_instance_index;

invariant gl_Position;

out vec3 fs_pos_in_clu_cam;

void main() {
  InstanceMatrices m = instance_matrices_buffer[vs_instance_index];

  vec4 pos_in_obj = to_homogeneous(vs_pos_in_obj);
  gl_Position = m.obj_to_ren_clp * pos_in_obj;

#if BASIC_PASS == BASIC_PASS_MASKED || BASIC_PASS == BASIC_PASS_TRANSPARENT
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
#endif

  fs_pos_in_clu_cam = mat4x3(m.obj_to_clu_cam) * pos_in_obj;
}
