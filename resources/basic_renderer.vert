uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;

invariant gl_Position;
out vec2 fs_pos_in_tex;

out vec3 fs_pos_in_lgt;
out vec3 fs_nor_in_lgt;
out vec3 fs_tan_in_lgt;

void main() {
  mat4x3 pos_from_obj_to_cam =
      mat4x3(pos_from_wld_to_cam * pos_from_obj_to_wld);
  vec3 pos_in_cam = pos_from_obj_to_cam * vec4(vs_pos_in_obj, 1.0);
  gl_Position = pos_from_cam_to_clp * vec4(pos_in_cam, 1.0);
  fs_pos_in_tex = vs_pos_in_tex;

#if defined(RENDER_TECHNIQUE_CLUSTERED)
  mat4x3 pos_from_obj_to_lgt =
      mat4x3(pos_from_wld_to_cls * pos_from_obj_to_wld);
  fs_pos_in_lgt = pos_from_obj_to_lgt * vec4(vs_pos_in_obj, 1.0);
#else
#if defined(LIGHT_SPACE_WLD)
  mat4x3 pos_from_obj_to_lgt = mat4x3(pos_from_obj_to_wld);
  fs_pos_in_lgt = pos_from_obj_to_lgt * vec4(vs_pos_in_obj, 1.0);
#elif defined(LIGHT_SPACE_HMD)
  mat4x3 pos_from_obj_to_lgt =
      mat4x3(pos_from_wld_to_hmd * pos_from_obj_to_wld);
  fs_pos_in_lgt = pos_from_obj_to_lgt * vec4(vs_pos_in_obj, 1.0);
#elif defined(LIGHT_SPACE_CAM)
  mat4x3 pos_from_obj_to_lgt = pos_from_obj_to_cam;
  fs_pos_in_lgt = pos_in_cam;
#endif // LIGHT_SPACE
#endif // RENDER_TECHNIQUE
  mat3 rot_from_obj_to_lgt = mat3(pos_from_obj_to_lgt);
  fs_nor_in_lgt = transpose(inverse(rot_from_obj_to_lgt)) * vs_nor_in_obj;
  fs_tan_in_lgt = rot_from_obj_to_lgt * vs_tan_in_obj;
}
