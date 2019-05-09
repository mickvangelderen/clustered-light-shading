uniform sampler2D color_sampler;
uniform vec4 channel_weights = vec4(1.0, 1.0, 1.0, 1.0);
uniform vec4 channel_defaults = vec4(0.0, 0.0, 0.0, 1.0);

in vec2 fs_pos_in_tex;

out vec4 frag_color;

void main() {
  frag_color = mix(channel_defaults, texture(color_sampler, fs_pos_in_tex),
                   channel_weights);
}
