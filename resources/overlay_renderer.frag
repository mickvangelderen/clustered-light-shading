uniform sampler2D color_sampler;

in vec2 fs_pos_in_tex;

out vec4 frag_color;

void main() {
  frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rrr, 1.0);
}
