#version 400

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform float highlight;

in vec3 fs_ver_nor;
in vec2 fs_tex_pos;
out vec4 frag_color;

void main() {
  vec4 d = texture(diffuse_sampler, fs_tex_pos);
  // frag_color = vec4(mix(d.rgb, vec3(1.0, 1.0, 1.0), highlight), d.a);
  frag_color = vec4((fs_ver_nor + vec3(1.0)) / 2.0, 1.0);
}
