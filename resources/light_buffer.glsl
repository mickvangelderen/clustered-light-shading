#include "point_light.glsl"

layout(std140, binding = LIGHT_BUFFER_BINDING) buffer LightBuffer {
  mat4 wld_to_lgt;
  mat4 lgt_to_wld;

  uvec4 light_count;

  PointLight point_lights[];
} light_buffer;
