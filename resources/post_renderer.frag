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
uniform usampler2D nor_in_cam_sampler;

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

vec4 sample_nor_in_cam(vec2 pos_in_tex) {
  uvec2 sam = texture(nor_in_cam_sampler, pos_in_tex).xy;
  float x = float(sam.x & 127) * (2.0 / 127.0) - 1.0;
  float y = float(sam.y & 255) * (2.0 / 255.0) - 1.0;
  float z_sign = float(int((sam.x & 128) >> 6) - 1);
  float z_mag = sqrt(max(1 - x * x - y * y, 0.0));
  float z = z_sign * z_mag;
  return vec4(x, y, z, z_sign);
}

vec3 compute_nor_in_cam() {
  float pixel_dx = 1.0 / width;
  float pixel_dy = 1.0 / height;

  vec3 c_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 l_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(-pixel_dx, 0.0));
  vec3 r_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(pixel_dx, 0.0));
  vec3 b_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(0.0, -pixel_dy));
  vec3 t_in_cam = sample_pos_in_cam(fs_pos_in_tex + vec2(0.0, pixel_dy));

  vec3 db_in_cam = b_in_cam - c_in_cam;
  vec3 dt_in_cam = t_in_cam - c_in_cam;
  vec3 dl_in_cam = l_in_cam - c_in_cam;
  vec3 dr_in_cam = r_in_cam - c_in_cam;

  // TODO: Don't know why we need the z-coordinate flip but don't care to find
  // out.
  return vec3(1.0, 1.0, -1.0) *
         normalize(cross(db_in_cam, dl_in_cam) + cross(dl_in_cam, dt_in_cam) +
                   cross(dt_in_cam, dr_in_cam) + cross(dr_in_cam, db_in_cam));
}

void main() {
  vec4 nor_in_cam = sample_nor_in_cam(fs_pos_in_tex);
  if ((int(gl_FragCoord.x) / 128 + int(gl_FragCoord.y) / 128) % 2 == 0) {
    frag_color = vec4((nor_in_cam.rgb + vec3(1.0)) / 2.0, 1.0);
  } else {
    frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);
  }
}
