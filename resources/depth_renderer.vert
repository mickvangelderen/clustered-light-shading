uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

invariant gl_Position;

void main() {
  mat4x3 pos_from_obj_to_cam = mat4x3(pos_from_wld_to_cam * pos_from_obj_to_wld);
  pos_in_cam = pos_from_obj_to_cam * vec4(vs_pos_in_obj, 1.0);
  gl_Position = pos_from_obj_to_clp * vec4(pos_in_cam, 1.0);
}
