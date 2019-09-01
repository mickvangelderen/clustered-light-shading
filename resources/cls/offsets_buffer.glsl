// Used in prefix sums.

layout(std430, binding = OFFSETS_BUFFER_BINDING) buffer OffsetsBuffer {
  uint offsets[];
};
