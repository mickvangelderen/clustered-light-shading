layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = NORMAL_SAMPLER_LOC) uniform sampler2D normal_sampler;
layout(location = AMBIENT_SAMPLER_LOC) uniform sampler2D ambient_sampler;
layout(location = DIFFUSE_SAMPLER_LOC) uniform sampler2D diffuse_sampler;
layout(location = SPECULAR_SAMPLER_LOC) uniform sampler2D specular_sampler;
layout(location = SHININESS_LOC) uniform float shininess;

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
flat out uint fs_instance_index;

struct InstanceMatrices {
  mat4 pos_from_obj_to_wld;
  mat4 pos_from_obj_to_lgt;
  mat4 nor_from_obj_to_lgt;
};

layout(std430, binding = INSTANCE_MATRICES_BUFFER_BINDING) buffer InstanceMatricesBuffer {
  InstanceMatrices instance_matrices_buffer[];
};

void main() {
  InstanceMatrices instance_matrices = instance_matrices_buffer[vs_instance_index];

  vec4 pos_in_obj = vec4(vs_pos_in_obj, 1.0);
  vec4 pos_in_wld = instance_matrices.pos_from_obj_to_wld * pos_in_obj;
  gl_Position = cam_to_clp * wld_to_cam * pos_in_wld;

  fs_pos_in_lgt = instance_matrices.pos_from_obj_to_lgt * pos_in_wld;
  fs_nor_in_lgt = mat3(instance_matrices.nor_from_obj_to_lgt) * vs_nor_in_obj;
  fs_bin_in_lgt = mat3(instance_matrices.pos_from_obj_to_lgt) * vs_bin_in_obj;
  fs_tan_in_lgt = mat3(instance_matrices.pos_from_obj_to_lgt) * vs_tan_in_obj;
  // NOTE(mickvangelderen): TOO LAZY TO CHANGE IMAGE ORIGIN.
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
  fs_instance_index = vs_instance_index;
}
