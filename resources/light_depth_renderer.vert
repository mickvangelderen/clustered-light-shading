#include "common.glsl"
#include "instance_matrices_buffer.glsl"

#if !defined(BASIC_PASS)
#error BASIC_PASS is undefined.
#endif

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_INSTANCE_INDEX_LOC) in uint vs_instance_index;

#if BASIC_PASS == BASIC_PASS_MASKED
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
out vec2 ge_pos_in_tex;
#endif

void main() {
  InstanceMatrices m = instance_matrices_buffer[vs_instance_index];

  vec4 pos_in_obj = to_homogeneous(vs_pos_in_obj);
  // FIXME: assuming lgt == wld
  gl_Position = m.obj_to_lgt * pos_in_obj;

#if BASIC_PASS == BASIC_PASS_MASKED
  ge_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
#endif
}
