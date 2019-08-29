#include "../common.glsl"
#include "cluster_space_buffer.glsl"
#include "cluster_fragment_counts_buffer.glsl"
#include "active_cluster_light_counts_buffer.glsl"
#include "active_cluster_light_offsets_buffer.glsl"

layout(location = PASS_LOC) uniform uint pass;

in vec2 fs_pos_in_tex;
flat in uvec3 fs_idx_in_cls;
flat in uint fs_cluster_index;
flat in uint fs_active_cluster_index;

layout(location = 0) out vec4 frag_color;

void main() {
  uint frag_count = cluster_fragment_counts[fs_cluster_index];
  uint light_count = active_cluster_light_counts[fs_active_cluster_index];
  uint light_offset = active_cluster_light_offsets[fs_active_cluster_index];

  // COLORS
  // frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_space.dimensions), 1.0);

  float border_width = 0.02;

  if (
      fs_pos_in_tex.x > border_width && fs_pos_in_tex.x < (1.0 - border_width) &&
      fs_pos_in_tex.y > border_width && fs_pos_in_tex.y < (1.0 - border_width)
      ) {
    if (pass == 1) {
      // frag_color = vec4(vec3(float(frag_count) / 150.0), 1.0);
      frag_color = vec4(1.0, 0.6, 0.2, float(light_count)/1500.0);
      // frag_color = vec4(1.0, 1.0, 1.0, float(light_offset)/1500.0);
      // frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_space.dimensions), 1.0);
    } else {
      discard;
    }
  } else {
    if (pass == 0) {
      frag_color = vec4(vec3(0.3), 1.0);
    } else {
      discard;
    }
  }
}
