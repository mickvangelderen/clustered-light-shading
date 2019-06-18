uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;

out vec3 fs_pos_in_cam;
out vec3 fs_pos_in_cls;
out vec2 fs_pos_in_tex;
out vec4 fs_pos_in_lgt;
out vec3 fs_nor_in_cam;
out vec3 fs_tan_in_cam;

invariant gl_Position;

void main() {
  mat4 pos_from_obj_to_cam = pos_from_wld_to_cam * pos_from_obj_to_wld;
  mat3 rot_from_obj_to_cam = mat3(pos_from_obj_to_cam);
  mat4 pos_from_obj_to_cls = pos_from_wld_to_cls * pos_from_obj_to_wld;
  mat4 light_pos_from_obj_to_cam =
      light_pos_from_wld_to_cam * pos_from_obj_to_wld;

  gl_Position = pos_from_cam_to_clp * pos_from_obj_to_cam *
    vec4(vs_pos_in_obj, 1.0);

  fs_pos_in_cam = mat4x3(pos_from_obj_to_cam) * vec4(vs_pos_in_obj, 1.0);
  fs_pos_in_cls = mat4x3(pos_from_obj_to_cls) * vec4(vs_pos_in_obj, 1.0);
  fs_pos_in_tex = vs_pos_in_tex;
  fs_pos_in_lgt = light_pos_from_cam_to_clp * light_pos_from_obj_to_cam * vec4(vs_pos_in_obj, 1.0);
  fs_nor_in_cam = transpose(inverse(rot_from_obj_to_cam)) * vs_nor_in_obj;
  fs_tan_in_cam = rot_from_obj_to_cam * vs_tan_in_obj;
}
