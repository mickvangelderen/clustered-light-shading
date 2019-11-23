in vec2 fs_pos_in_tex;
in vec3 fs_tint;

layout(location = 0) out vec4 frag_color;

void main() {
  vec2 d = fs_pos_in_tex - vec2(0.5, 0.5);
  float a = max(0.0, 0.25 - dot(d, d));

  frag_color = vec4(fs_tint, a);
}
