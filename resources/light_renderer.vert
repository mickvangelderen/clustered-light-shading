#include "common.glsl"
#include "light_buffer.glsl"

layout(location = WLD_TO_CAM_LOC) uniform mat4 wld_to_cam;
layout(location = CAM_TO_CLP_LOC) uniform mat4 cam_to_clp;

layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

out vec2 fs_pos_in_tex;
out vec3 fs_tint;

void main() {
  PointLight light = light_buffer.point_lights[gl_InstanceID];

  vec3 pos_in_cam = mat4x3(wld_to_cam) * to_homogeneous(light.position);
  pos_in_cam.xy += (vs_pos_in_tex - vec2(0.5)) * light.r1 * 0.20;
  gl_Position = cam_to_clp * to_homogeneous(pos_in_cam);

  fs_pos_in_tex = vs_pos_in_tex;
  fs_tint = light.tint;
}
