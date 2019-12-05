#include "point_light.glsl"

layout(std430, binding = LIGHT_BUFFER_BINDING) buffer LightBuffer {
  uint light_count;
  uint _pad0[15];

  PointLight point_lights[];
} light_buffer;
