uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

void main() {
  gl_Position =
      pos_from_wld_to_clp * pos_from_obj_to_wld * vec4(vs_pos_in_obj, 1.0);
}
