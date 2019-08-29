// Carries information about the light sources in the scene. Used during light counting and assignment.

layout(std430, binding = LIGHT_XYZR_BUFFER_BINDING) buffer LightXYZRBuffer {
  vec4 light_xyzr[];
};
