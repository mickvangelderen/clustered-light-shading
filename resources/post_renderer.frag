#version 420

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
uniform sampler2D nor_in_cam_sampler;

#define HBAO_KERNEL_BINDING 0

layout(std140, binding = HBAO_KERNEL_BINDING) uniform HBAO_Kernel {
  vec4 hbao_kernel[64];
};

in vec2 fs_pos_in_tex;

out vec4 frag_color;

float lerp(float x, float x0, float x1, float y0, float y1) {
  return ((x - x0) * y1 + (x1 - x) * y0) / (x1 - x0);
}

float sample_z_ndc(vec2 pos_in_tex) {
  return texture(depth_sampler, pos_in_tex).r * 2.0 - 1.0;
}

// Reverse projection matrix.
float z_from_ndc_to_cam(float z_ndc) {
  return (2.0 * z0 * z1) / (z_ndc * (z1 - z0) - (z0 + z1));
}

vec3 sample_pos_in_cam(vec2 pos_in_tex) {
  float z_cam = z_from_ndc_to_cam(sample_z_ndc(pos_in_tex));
  return vec3(z_cam / -z0 * mix(x0, x1, pos_in_tex.x),
              z_cam / -z0 * mix(y0, y1, pos_in_tex.y), z_cam);
}

vec3 sample_nor_in_cam(vec2 pos_in_tex) {
  vec3 sam = texture(nor_in_cam_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
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
  vec3 pos_in_cam = sample_pos_in_cam(fs_pos_in_tex);

  int occlude_count = 0;
  int visible_count = 0;
  for (int i = 0; i < 64; i += 1) {
    vec3 sample_pos_in_cam = pos_in_cam + hbao_kernel[i].xyz * 0.5;
    vec2 sample_pos_in_tex = vec2(
        lerp(-z0 / sample_pos_in_cam.z * sample_pos_in_cam.x, x0, x1, 0.0, 1.0),
        lerp(-z0 / sample_pos_in_cam.z * sample_pos_in_cam.y, y0, y1, 0.0,
             1.0));
    float hit_z_in_ndc = sample_z_ndc(sample_pos_in_tex);
    float hit_z_in_cam = z_from_ndc_to_cam(hit_z_in_ndc);
    // FIXME: Don't ignore faraway
    if (hit_z_in_cam < sample_pos_in_cam.z) {
      visible_count += 1;
    } else {
      occlude_count += 1;
    }
  }

  frag_color = vec4(float(visible_count) /
                        float(visible_count + occlude_count) * vec3(1.0),
                    1.0);

  // if (pos_in_cam.x < 0.5) {
  //   frag_color = vec4(1.0, 1.0, 0.0, 1.0);
  // } else {
  //   frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);
  // }
  // if (fs_pos_in_tex.x < 0.5) {
  //   // frag_color =
  //   //     vec4((pos_in_cam.z - hit_z_in_cam) * vec3(0.5) +
  //   vec3(0.5), 1.0);
  // } else {
  //   frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);
  // }
}
