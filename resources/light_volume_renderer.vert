#include "common.glsl"
#include "light_buffer.glsl"

layout(location = WLD_TO_CLP_LOC) uniform mat4 wld_to_clp;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

out vec3 fs_tint;

void main() {
  PointLight light = light_buffer.point_lights[gl_InstanceID];

  vec3 pos_in_wld = light.position + vs_pos_in_obj * light.r1;
  gl_Position = wld_to_clp * to_homogeneous(pos_in_wld);

  fs_tint = light.tint;
}
