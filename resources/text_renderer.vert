uniform vec2 dimensions;
uniform vec2 text_dimensions;

layout(location = VS_POS_IN_OBJ_LOC) in vec2 vs_pos_in_obj;
layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

out vec2 fs_pos_in_tex;

void main() {
  gl_Position = vec4((vs_pos_in_obj / dimensions) * 2.0 - vec2(1.0), 0.0, 1.0);
  fs_pos_in_tex = vs_pos_in_tex / text_dimensions;
}

