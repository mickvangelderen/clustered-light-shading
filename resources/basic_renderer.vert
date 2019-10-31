layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = NORMAL_SAMPLER_LOC) uniform sampler2D normal_sampler;
layout(location = AMBIENT_SAMPLER_LOC) uniform sampler2D ambient_sampler;
layout(location = DIFFUSE_SAMPLER_LOC) uniform sampler2D diffuse_sampler;
layout(location = SPECULAR_SAMPLER_LOC) uniform sampler2D specular_sampler;
layout(location = SHININESS_LOC) uniform float shininess;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

invariant gl_Position;

out vec4 fs_pos_in_lgt;
out vec3 fs_nor_in_obj;
out vec2 fs_pos_in_tex;

void main() {
  vec4 pos_in_obj = vec4(vs_pos_in_obj, 1.0);
  vec4 pos_in_wld = obj_to_wld * pos_in_obj;
  gl_Position = cam_to_clp * wld_to_cam * pos_in_wld;

  fs_pos_in_lgt = pos_in_wld;
  fs_nor_in_obj = vs_nor_in_obj;
  // fs_nor_in_lgt = transpose(inverse(mat3(obj_to_wld))) * vs_nor_in_obj;
  // NOTE(mickvangelderen): TOO LAZY TO CHANGE IMAGE ORIGIN.
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
}
