#include "point_light.glsl"

layout(std430, binding = LIGHT_BUFFER_BINDING) buffer LightBuffer {
  uint light_count;
  uint virtual_light_count;
  uint _pad0[14];

  PointLight point_lights[];
} light_buffer;
