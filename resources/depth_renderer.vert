uniform mat4 pos_from_obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

invariant gl_Position;

void main() {
  mat4 pos_from_obj_to_cam = pos_from_wld_to_cam * pos_from_obj_to_wld;

  gl_Position = pos_from_cam_to_clp * pos_from_obj_to_cam *
    vec4(vs_pos_in_obj, 1.0);
}
