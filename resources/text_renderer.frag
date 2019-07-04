uniform sampler2D text_sampler;

in vec2 fs_pos_in_tex;

layout(location = 0) out vec4 frag_color;

void main() {
  frag_color = vec4(vec3(1.0), texture(text_sampler, fs_pos_in_tex).r);
}
