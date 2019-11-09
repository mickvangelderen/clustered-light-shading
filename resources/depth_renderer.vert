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
  InstanceMatrices instance_matrices = instance_matrices_buffer[vs_instance_index];
  mat4 pos_from_obj_to_wld = instance_matrices.pos_from_obj_to_wld;

  gl_Position = cam_to_clp * wld_to_cam * pos_from_obj_to_wld * vec4(vs_pos_in_obj, 1.0);

#if defined(BASIC_PASS)
#if BASIC_PASS == BASIC_PASS_MASKED
  fs_pos_in_tex = vec2(vs_pos_in_tex.x, 1.0 - vs_pos_in_tex.y);
#endif
#else
#error BASIC_PASS is undefined.
#endif
}
