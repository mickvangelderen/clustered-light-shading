layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

void main() {
  gl_Position =
      cam_to_clp * wld_to_cam * obj_to_wld * vec4(vs_pos_in_obj, 1.0);
}
