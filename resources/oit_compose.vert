layout(location = VS_POS_IN_TEX_LOC) in vec2 vs_pos_in_tex;

void main() {
  gl_Position = vec4(vs_pos_in_tex * 2.0 - 1.0, 0.0, 1.0);
}
