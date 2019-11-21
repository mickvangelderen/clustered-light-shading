#include "common.glsl"
#include "instance_matrices_buffer.glsl"

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_INSTANCE_INDEX_LOC) in uint vs_instance_index;

#if defined(BASIC_PASS)
#if BASIC_PASS == BASIC_PASS_MASKED
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;
#endif
#else
#error BASIC_PASS is undefined.
#endif

invariant gl_Position;

out vec2 fs_pos_in_tex;

void main() {
  InstanceMatrices m = instance_matrices_buffer[vs_instance_index];

  vec4 pos_in_obj = to_homogeneous(vs_pos_in_obj);
  gl_Position = m.obj_to_ren_clp * pos_in_obj;

#if defined(BASIC_PASS)
#if BASIC_PASS == BASIC_PASS_MASKED
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
#endif
#else
#error BASIC_PASS is undefined.
#endif
}
