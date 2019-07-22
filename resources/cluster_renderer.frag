#include "common.glsl"
#include "cluster_renderer.glsl"

layout(binding = 0) buffer OutputBuffer {
  uint fragments_per_cluster[];
};

in vec2 fs_pos_in_tex;
flat in uvec4 fs_indices;

layout(location = 0) out vec4 frag_color;

uniform uint debug_pass;

void main() {
  uvec3 idx_in_cls = fs_indices.xyz;
  uint cluster_index = fs_indices.w;

  // frag_color = vec4(vec3(idx_in_cls)/vec3(cluster_dims), 1.0);
  uint frag_count = fragments_per_cluster[cluster_index];

  if (frag_count == 0) {
    discard;
  }

  frag_color = vec4(vec3(float(frag_count) / 4.0), 1.0);

  // if (
  //     fs_pos_in_tex.x > 0.01 && fs_pos_in_tex.x < 0.99 &&
  //     fs_pos_in_tex.y > 0.01 && fs_pos_in_tex.y < 0.99
  //     ) {
  //   if (debug_pass == 1) {
  //     frag_color = vec4(1.0, 0.6, 0.2, float(current_light_count)/float(32.0));
  //   } else {
  //     discard;
  //   }
  // } else {
  //   if (debug_pass == 0) {
  //     frag_color = vec4(vec3(0.3), 1.0);
  //   } else {
  //     discard;
  //   }
  // }
}
