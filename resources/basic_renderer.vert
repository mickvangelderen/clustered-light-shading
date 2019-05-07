uniform float time;
uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;
uniform mat4 pos_from_obj_to_wld;
uniform mat4 pos_from_wld_to_cam;
uniform mat4 pos_from_cam_to_clp;
uniform mat4 pos_from_wld_to_lgt;
uniform float highlight;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;

out vec3 fs_pos_in_cam;
out vec2 fs_pos_in_tex;
out vec4 fs_pos_in_lgt;
out vec3 fs_nor_in_cam;
out vec3 fs_tan_in_cam;

void main() {
  mat4 pos_from_ver_to_obj = mat4(1.0, 0.0, 0.0, 0.0,              //
                                  0.0, 1.0, 0.0, 0.0,              //
                                  0.0, 0.0, 1.0, 0.0,              //
                                  0.0, -0.02 * highlight, 0.0, 1.0 //
  );

  mat4 pos_from_obj_to_cam_mat4 =
      pos_from_wld_to_cam * pos_from_obj_to_wld * pos_from_ver_to_obj;

  mat4 pos_from_obj_to_lgt_mat4 =
      pos_from_wld_to_lgt * pos_from_obj_to_wld * pos_from_ver_to_obj;

  gl_Position =
      pos_from_cam_to_clp * pos_from_obj_to_cam_mat4 * vec4(vs_pos_in_obj, 1.0);

  fs_pos_in_cam = mat4x3(pos_from_obj_to_cam_mat4) * vec4(vs_pos_in_obj, 1.0);
  fs_pos_in_tex = vs_pos_in_tex;
  fs_pos_in_lgt = pos_from_obj_to_lgt_mat4 * vec4(vs_pos_in_obj, 1.0);

  fs_nor_in_cam =
      transpose(inverse(mat3(pos_from_obj_to_cam_mat4))) * vs_nor_in_obj;
  fs_tan_in_cam = mat3(pos_from_obj_to_cam_mat4) * vs_tan_in_obj;
}
