#include "instance_matrices_buffer.glsl"

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_BIN_IN_OBJ_LOC) in vec3 vs_bin_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_INSTANCE_INDEX_LOC) in uint vs_instance_index;

invariant gl_Position;

out vec4 fs_pos_in_lgt;
out vec3 fs_nor_in_lgt;
out vec3 fs_bin_in_lgt;
out vec3 fs_tan_in_lgt;
out vec2 fs_pos_in_tex;

void main() {
  InstanceMatrices instance_matrices = instance_matrices_buffer[vs_instance_index];
  mat4 pos_from_obj_to_wld = instance_matrices.pos_from_obj_to_wld;
  mat4 pos_from_obj_to_lgt = instance_matrices.pos_from_obj_to_lgt;
  mat4 nor_from_obj_to_lgt = instance_matrices.nor_from_obj_to_lgt;

  gl_Position = cam_to_clp * wld_to_cam * pos_from_obj_to_wld * vec4(vs_pos_in_obj, 1.0);
  fs_pos_in_lgt = pos_from_obj_to_lgt * vec4(vs_pos_in_obj, 1.0);
  fs_nor_in_lgt = normalize(mat3(nor_from_obj_to_lgt) * vs_nor_in_obj);
  fs_bin_in_lgt = normalize(mat3(pos_from_obj_to_lgt) * vs_bin_in_obj);
  fs_tan_in_lgt = normalize(mat3(pos_from_obj_to_lgt) * vs_tan_in_obj);
  // NOTE(mickvangelderen): TOO LAZY TO CHANGE IMAGE ORIGIN.
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
}
