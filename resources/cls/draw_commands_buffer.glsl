#include "../draw_command.glsl"

layout(std430, binding = DRAW_COMMANDS_BUFFER_BINDING) buffer DrawCommandsBuffer {
  DrawCommand draw_commands;
};
