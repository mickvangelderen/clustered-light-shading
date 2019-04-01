#version 400 core

uniform mat4 pos_from_wld_to_clp;
uniform float highlight;

in vec3 vs_ver_pos;
in vec3 vs_ver_nor;
in vec2 vs_tex_pos;
out vec3 fs_ver_nor;
out vec2 fs_tex_pos;

void main() {
  mat4 pos_from_obj_to_wld = mat4(
                                  1.0, 0.0, 0.0, 0.0,
                                  0.0, 1.0, 0.0, 0.0,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.0, -0.02*highlight, 0.0, 1.0
                                  );

  gl_Position =
      pos_from_wld_to_clp * pos_from_obj_to_wld * vec4(vs_ver_pos, 1.0);
  fs_ver_nor = vs_ver_nor;
  fs_tex_pos = vs_tex_pos;
}
