uniform mat4 obj_to_wld;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

invariant gl_Position;
out vec2 fs_pos_in_tex;

void main() {
  mat4 obj_to_cam = wld_to_cam * obj_to_wld;
  gl_Position = cam_to_clp * (obj_to_cam * vec4(vs_pos_in_obj, 1.0));
  fs_pos_in_tex = vs_pos_in_tex;
}
