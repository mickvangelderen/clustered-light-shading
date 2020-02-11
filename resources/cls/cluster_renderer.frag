#include "../common.glsl"
#include "../heatmap.glsl"
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

#define VIS 1

void main() {
  uint frag_count = cluster_fragment_counts[fs_cluster_index];
  uint light_count = active_cluster_light_counts[fs_active_cluster_index];
  uint light_offset = active_cluster_light_offsets[fs_active_cluster_index];

  // COLORS
  // frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_space.dimensions), 1.0);
  // return;


  // CLUSTER INDEX
  // if (pass == 0) {
  //   frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_space.dimensions), 1.0);
  // } else {
  //   discard;
  // }

  float MAX_LIGHTS = 20.0;

  float border_width = 0.02;
  bool inside = fs_pos_in_tex.x > border_width &&
    fs_pos_in_tex.x < (1.0 - border_width) &&
    fs_pos_in_tex.y > border_width &&
    fs_pos_in_tex.y < (1.0 - border_width);

#if VIS == 0
  if (inside) {
    frag_color = vec4(heatmap(float(light_count), 0.0, MAX_LIGHTS), 1.0);
  } else {
    frag_color = vec4(vec3(0.5), 1.0);
  }
#endif
#if VIS == 1
  if (inside) {
  if (pass == 1) {
    // frag_color = vec4(vec3(float(frag_count) / 150.0), 1.0);
    frag_color = vec4(1.0, 0.6, 0.2, float(light_count)/MAX_LIGHTS);
  } else {
    discard;
  }
 } else {
  if (pass == 0) {
    frag_color = vec4(vec3(0.5), 1.0);
  } else {
    discard;
  }
 }
#endif
}
