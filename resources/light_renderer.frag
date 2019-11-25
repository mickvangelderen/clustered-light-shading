in vec2 fs_pos_in_tex;
in vec3 fs_tint;

layout(location = 0) out vec4 frag_color;

void main() {
  vec2 d = fs_pos_in_tex * 2.0 - vec2(1.0);
  float a = max(0.0, 1.0 - dot(d, d));

  frag_color = vec4(normalize(fs_tint), pow(a, 4.0));
}
