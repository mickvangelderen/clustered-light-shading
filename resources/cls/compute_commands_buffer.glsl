#include "../compute_command.glsl"

#define COMPUTE_COMMAND_INDEX_ACTIVE_CLUSTER_COUNT 0
#define COMPUTE_COMMAND_INDEX_PREFIX_SUM_LIGHT_COUNTS 1
#define COMPUTE_COMMAND_INDEX_ACTIVE_CLUSTER_HIST 2

layout(std430, binding = COMPUTE_COMMANDS_BUFFER_BINDING) buffer ComputeCommandsBuffer {
  ComputeCommand compute_commands[];
};
