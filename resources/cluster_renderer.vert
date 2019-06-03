uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;

out vec3 fs_pos_in_cam;
out vec2 fs_pos_in_tex;
out vec3 fs_nor_in_cam;
out vec3 fs_tan_in_cam;

void main() {
  mat4 pos_from_obj_to_cam = pos_from_wld_to_cam * pos_from_obj_to_wld;
  vec4 fs_pos_in_cam_vec4 = pos_from_obj_to_cam * vec4(vs_pos_in_obj, 1.0);
  gl_Position = pos_from_cam_to_clp * fs_pos_in_cam_vec4;
  fs_pos_in_cam = vec3(fs_pos_in_cam_vec4);
  fs_pos_in_tex = vs_pos_in_tex;
  fs_nor_in_cam =
    transpose(inverse(mat3(pos_from_obj_to_cam))) * vs_nor_in_obj;
  fs_tan_in_cam = mat3(pos_from_obj_to_cam) * vs_tan_in_obj;
}
