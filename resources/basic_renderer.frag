#version 400

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform float highlight;

in vec3 fs_pos_in_obj;
in vec2 fs_pos_in_tex;
in vec3 fs_nor_in_obj;
in vec3 fs_tan_in_obj;

out vec4 frag_color;

void main() {
  vec4 d = texture(diffuse_sampler, fs_pos_in_tex);
  // frag_color = vec4(mix(d.rgb, vec3(1.0, 1.0, 1.0), highlight), d.a);
  frag_color = vec4((fs_tan_in_obj + vec3(1.0)) / 2.0, 1.0);
}
