#version 400

uniform float time;
uniform int width;
uniform int height;
uniform float x0;
uniform float x1;
uniform float y0;
uniform float y1;
uniform float z0;
uniform float z1;
uniform sampler2D color_sampler;
uniform sampler2D depth_sampler;

in vec2 fs_pos_in_tex;

out vec4 frag_color;

float lerp(float x, float x0, float x1, float y0, float y1) {
  return ((x - x0) * y1 + (x1 - x) * y0) / (x1 - x0);
}

void main() {
  float z_ndc = texture(depth_sampler, fs_pos_in_tex).r * 2.0 - 1.0;
  float z_cam = (2.0 * z0 * z1) / (z_ndc * (z1 - z0) - (z0 + z1));
  float z_lin = lerp(z_cam, -z0, -z1, 1.0, 0.0);
  float fog_weight = lerp(z_cam, -z0, -z1, 0.0, 1.0);

  vec3 pos_in_cam = z_cam * vec3(lerp(fs_pos_in_tex.x, 0.0, 1.0, x0, x1), //
                                 lerp(fs_pos_in_tex.y, 0.0, 1.0, y0, y1), //
                                 1.0);

  float z_center_ndc = texture(depth_sampler, vec2(0.5, 0.5)).r * 2.0 - 1.0;
  float z_center_cam = (2.0 * z0 * z1) / (z_center_ndc * (z1 - z0) - (z0 + z1));

  vec3 d = pos_in_cam - vec3(0.0, 0.0, z_center_cam);
  if (dot(d, d) <= 1.0) {
    frag_color = vec4(1.0, 1.0, 1.0, 1.0);
  } else {
    vec4 fog_color = vec4(0.4, 0.5, 0.6, 1.0);
    frag_color =
        mix(texture(color_sampler, fs_pos_in_tex), fog_color, fog_weight);
  }
}
