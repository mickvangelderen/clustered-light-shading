struct InstanceMatrices {
  mat4 obj_to_ren_clp; // = wld_to_ren_clp * obj_to_wld
  mat4 obj_to_clu_clp; // = wld_to_clu_clp * obj_to_wld
  mat4 obj_to_lgt; // = obj_to_wld
  mat4 obj_to_lgt_inv_tra; // = transpose(inverse(obj_to_lgt))
};

layout(std430, binding = INSTANCE_MATRICES_BUFFER_BINDING) buffer InstanceMatricesBuffer {
  InstanceMatrices instance_matrices_buffer[];
};
