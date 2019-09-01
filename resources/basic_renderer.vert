#include "native/RENDER_TECHNIQUE"
#include "light_buffer.glsl"

uniform mat4 obj_to_wld;

layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_NOR_IN_OBJ_LOC) in vec3 vs_nor_in_obj;
layout(location = VS_TAN_IN_OBJ_LOC) in vec3 vs_tan_in_obj;

invariant gl_Position;
out vec2 fs_pos_in_tex;

#if defined(RENDER_TECHNIQUE_CLUSTERED)
#include "cls/cluster_space_buffer.glsl"
out vec4 fs_pos_in_cclp;
#endif

out vec3 fs_pos_in_lgt;
out vec3 fs_nor_in_lgt;
out vec3 fs_tan_in_lgt;

void main() {
  vec4 pos_in_obj = vec4(vs_pos_in_obj, 1.0);
  vec4 pos_in_wld = obj_to_wld * pos_in_obj;
  gl_Position = cam_to_clp * wld_to_cam * pos_in_wld;
  fs_pos_in_tex = vs_pos_in_tex;

#if defined(RENDER_TECHNIQUE_CLUSTERED)
  vec3 pos_in_ccam = mat4x3(cluster_space.wld_to_cam) * pos_in_wld;
  fs_pos_in_cclp = cluster_cam_to_clp(pos_in_ccam);
#endif
  mat4 obj_to_lgt = light_buffer.wld_to_lgt * obj_to_wld;
  fs_pos_in_lgt = mat4x3(obj_to_lgt) * pos_in_obj;
  fs_nor_in_lgt = transpose(inverse(mat3(obj_to_lgt))) * vs_nor_in_obj;
  fs_tan_in_lgt = mat3(obj_to_lgt) * vs_tan_in_obj;
}
