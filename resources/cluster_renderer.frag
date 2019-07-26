#include "common.glsl"
#include "cluster_renderer.glsl"

in vec2 fs_pos_in_tex;
flat in uvec3 fs_idx_in_cls;
flat in uint fs_cluster_index;
flat in uint fs_active_cluster_index;

layout(location = 0) out vec4 frag_color;

void main() {
  uvec3 idx_in_cls = fs_idx_in_cls;
  uint cluster_index = fs_cluster_index;
  uint active_cluster_index = fs_active_cluster_index;

  uint frag_count = cluster_fragment_counts[cluster_index];
  uint light_count = active_cluster_light_counts[active_cluster_index];

  // COLORS
  // frag_color = vec4(vec3(idx_in_cls)/vec3(cluster_dims), 1.0);

  if (
      fs_pos_in_tex.x > 0.03 && fs_pos_in_tex.x < 0.97 &&
      fs_pos_in_tex.y > 0.03 && fs_pos_in_tex.y < 0.97
      ) {
    if (pass == 1) {
      frag_color = vec4(1.0, 0.6, 0.2, float(light_count)/10.0);
      // frag_color = vec4(vec3(idx_in_cls)/vec3(cluster_dims), 0.1);
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
