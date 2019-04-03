#version 400 core

uniform mat4 pos_from_wld_to_clp;
uniform float highlight;

in vec3 vs_pos_in_obj;
in vec2 vs_pos_in_tex;
in vec3 vs_nor_in_obj;
in vec3 vs_tan_in_obj;

out vec3 fs_pos_in_obj;
out vec2 fs_pos_in_tex;
out vec3 fs_nor_in_obj;
out vec3 fs_tan_in_obj;

void main() {
  mat4 pos_from_obj_to_wld = mat4(1.0, 0.0, 0.0, 0.0,              //
                                  0.0, 1.0, 0.0, 0.0,              //
                                  0.0, 0.0, 1.0, 0.0,              //
                                  0.0, -0.02 * highlight, 0.0, 1.0 //
  );

  gl_Position =
      pos_from_wld_to_clp * pos_from_obj_to_wld * vec4(vs_pos_in_obj, 1.0);

  fs_pos_in_obj = vs_pos_in_obj;
  fs_pos_in_tex = vs_pos_in_tex;
  fs_nor_in_obj = vs_nor_in_obj;
  fs_tan_in_obj = vs_tan_in_obj;
}
