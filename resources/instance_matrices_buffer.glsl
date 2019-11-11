struct InstanceMatrices {
  mat4 pos_from_obj_to_wld;
  mat4 pos_from_obj_to_lgt;
  mat4 nor_from_obj_to_lgt;
};

layout(std430, binding = INSTANCE_MATRICES_BUFFER_BINDING) buffer InstanceMatricesBuffer {
  InstanceMatrices instance_matrices_buffer[];
};
