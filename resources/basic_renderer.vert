#version 400 core

uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;
uniform mat4 pos_from_wld_to_clp;
uniform mat4 pos_from_wld_to_cam;
uniform float highlight;

in vec3 vs_pos_in_obj;
in vec2 vs_pos_in_tex;
in vec3 vs_nor_in_obj;
in vec3 vs_tan_in_obj;

out vec3 fs_pos_in_cam;
out vec2 fs_pos_in_tex;
out vec3 fs_nor_in_cam;
out vec3 fs_tan_in_cam;

void main() {
  mat4 pos_from_obj_to_wld = mat4(1.0, 0.0, 0.0, 0.0,              //
                                  0.0, 1.0, 0.0, 0.0,              //
                                  0.0, 0.0, 1.0, 0.0,              //
                                  0.0, -0.02 * highlight, 0.0, 1.0 //
  );

  mat3 nor_from_obj_to_cam =
      transpose(inverse(mat3(pos_from_wld_to_cam) * mat3(pos_from_obj_to_wld)));

  gl_Position =
      pos_from_wld_to_clp * pos_from_obj_to_wld * vec4(vs_pos_in_obj, 1.0);

  fs_pos_in_cam = vec3(pos_from_wld_to_cam * pos_from_obj_to_wld *
                       vec4(vs_pos_in_obj, 1.0));
  fs_pos_in_tex = vs_pos_in_tex;
  fs_nor_in_cam = nor_from_obj_to_cam * vs_nor_in_obj;
  // fs_nor_in_cam = vs_nor_in_obj;
  fs_tan_in_cam =
      mat3(pos_from_wld_to_cam) * mat3(pos_from_obj_to_wld) * vs_tan_in_obj;
}
