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

float sample_z_ndc(vec2 pos) {
  return texture(depth_sampler, pos).r * 2.0 - 1.0;
}

float z_from_ndc_to_cam(float z_ndc) {
  return (2.0 * z0 * z1) / (z_ndc * (z1 - z0) - (z0 + z1));
}

vec3 sample_pos_in_cam(vec2 pos_in_tex) {
  float z_cam = z_from_ndc_to_cam(sample_z_ndc(pos_in_tex));
  return z_cam * vec3(lerp(pos_in_tex.x, 0.0, 1.0, x0, x1), //
                      lerp(pos_in_tex.y, 0.0, 1.0, y0, y1), //
                      1.0);
}

void main() {
  float pixel_dx = 1.0 / width;
  float pixel_dy = 1.0 / height;

  vec3 c_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 b_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(-pixel_dy, 0.0));
  vec3 t_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(pixel_dy, 0.0));
  vec3 l_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(-pixel_dx, 0.0));
  vec3 r_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(pixel_dx, 0.0));

  vec3 db_in_cam = b_in_cam - c_in_cam;
  vec3 dt_in_cam = t_in_cam - c_in_cam;
  vec3 dl_in_cam = l_in_cam - c_in_cam;
  vec3 dr_in_cam = r_in_cam - c_in_cam;

  vec3 n_in_cam =
      normalize(cross(db_in_cam, dl_in_cam) + cross(dl_in_cam, dt_in_cam) +
                cross(dt_in_cam, dr_in_cam) + cross(dr_in_cam, db_in_cam));

  frag_color = vec4(n_in_cam * 2.0 - vec3(1.0), 1.0);
}
