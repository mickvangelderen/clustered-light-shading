uniform sampler2D color_sampler;
uniform vec4 default_colors;
uniform mat4 color_matrix;

in vec2 fs_pos_in_tex;

out vec4 frag_color;

void main() {
  frag_color =
      default_colors + color_matrix * texture(color_sampler, fs_pos_in_tex);
}
