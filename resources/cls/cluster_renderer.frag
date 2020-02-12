#include "../common.glsl"
#include "../heatmap.glsl"
#include "cluster_space_buffer.glsl"
#include "cluster_fragment_counts_buffer.glsl"
#include "active_cluster_light_counts_buffer.glsl"
#include "active_cluster_light_offsets_buffer.glsl"

layout(location = VISUALISATION_LOC) uniform uint visualisation;
layout(location = PASS_LOC) uniform uint pass;

in vec2 fs_pos_in_tex;
flat in uvec3 fs_idx_in_cls;
flat in uint fs_cluster_index;
flat in uint fs_maybe_active_cluster_index;

layout(location = 0) out vec4 frag_color;

#define CLUSTER_INDICES 1
#define LIGHT_COUNT_HEATMAP 8
#define LIGHT_COUNT_VOLUMETRIC 9
#define FRAGMENT_COUNT_HEATMAP 16
#define FRAGMENT_COUNT_VOLUMETRIC 17

void main() {
  float border_width = 0.02;
  bool inside = fs_pos_in_tex.x > border_width &&
  fs_pos_in_tex.x < (1.0 - border_width) &&
  fs_pos_in_tex.y > border_width &&
  fs_pos_in_tex.y < (1.0 - border_width);

  vec4 border_color = vec4(vec3(0.5), 1.0);

  if (visualisation == CLUSTER_INDICES) {
    if (inside) {
      frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_space.dimensions), 1.0);
    } else {
      frag_color = border_color;
    }
    return;
  }

  uint frag_count = cluster_fragment_counts[fs_cluster_index];
  uint light_count = 0;
  uint light_offset = 0;
  if (fs_maybe_active_cluster_index > 0) {
    uint active_cluster_index = fs_maybe_active_cluster_index - 1;
    light_count = active_cluster_light_counts[active_cluster_index];
    light_offset = active_cluster_light_offsets[active_cluster_index];
  }

  float MAX_LIGHTS = 25.0;
  float MAX_FRAGMENTS = 2000.0;

  float value = 0.0;
  if (visualisation == LIGHT_COUNT_HEATMAP || visualisation == LIGHT_COUNT_VOLUMETRIC) {
    value = float(light_count)/MAX_LIGHTS;
  }
  if (visualisation == FRAGMENT_COUNT_HEATMAP || visualisation == FRAGMENT_COUNT_VOLUMETRIC) {
    value = float(frag_count)/MAX_FRAGMENTS;
  }

  bool is_heatmap = (visualisation == LIGHT_COUNT_HEATMAP || visualisation == FRAGMENT_COUNT_HEATMAP);
  if (is_heatmap) {
    if (inside) {
      frag_color = vec4(heatmap(value, 0.0, 1.0), 1.0);
    } else {
      frag_color = border_color;
    }
    return;
  }

  bool is_volumetric = (visualisation == LIGHT_COUNT_VOLUMETRIC || visualisation == FRAGMENT_COUNT_VOLUMETRIC);
  if (is_volumetric) {
    if (pass == 0 && !inside) {
      frag_color = border_color;
    } else if (pass == 1 && inside) {
      frag_color = vec4(1.0, 0.6, 0.2, value);
    } else {
      discard;
    }
    return;
  }
}
