uniform mat4 obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

invariant gl_Position;

void main() {
  vec4 pos_in_obj = vec4(vs_pos_in_obj, 1.0);
  vec4 pos_in_wld = obj_to_wld * pos_in_obj;
  gl_Position = cam_to_clp * wld_to_cam * pos_in_wld;
}
