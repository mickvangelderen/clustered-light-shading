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
uniform usampler2D ao_sampler;

#define HBAO_KERNEL_BINDING 0

layout(std140, binding = HBAO_KERNEL_BINDING) uniform HBAO_Kernel {
  vec4 hbao_kernel[1024];
};

in vec2 fs_pos_in_tex;

out vec4 frag_color;

float lerp(float x, float x0, float x1, float y0, float y1) {
  return ((x - x0) * y1 + (x1 - x) * y0) / (x1 - x0);
}

float sample_z_ndc(vec2 pos_in_tex) {
  return texture(depth_sampler, pos_in_tex).r * 2.0 - 1.0;
}

vec2 pos_from_cam_to_tex(vec3 pos_in_cam) {
  float x = pos_in_cam.x;
  float y = pos_in_cam.y;
  float z = pos_in_cam.z;
  float s = z0 / z;

  return vec2((s * x - x0) / (x1 - x0), (s * y - y0) / (y1 - y0));
}

// Reverse projection matrix.
// float z_from_ndc_to_cam(float z_ndc) {
//   return (2.0 * z0 * z1) / (z_ndc * (z1 - z0) + z0 + z1);
// }

// Reverse infinite far perspective projection matrix.
float z_from_ndc_to_cam(float z_ndc) { return 2.0 * z0 / (1.0 - z_ndc); }

vec3 sample_pos_in_cam(vec2 pos_in_tex) {
  float z_ndc = sample_z_ndc(pos_in_tex);
  // This is z_from_ndc_to_cam(z_ndc) / z0
  float s = 2.0 / (1.0 - z_ndc);
  return vec3(s * mix(x0, x1, pos_in_tex.x), s * mix(y0, y1, pos_in_tex.y),
              s * z0);
}

uvec2 sample_ao(vec2 pos_in_tex) { return texture(ao_sampler, pos_in_tex).xy; }

vec3 sample_nor_in_cam(vec2 pos_in_tex) {
  vec3 sam = texture(nor_in_cam_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
}

void main() {
  vec3 pos_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 nor_in_cam = sample_nor_in_cam(fs_pos_in_tex);

  float dx = 1.0 / width;
  float dy = 1.0 / width;

  vec2 filtered_ao = vec2(0.0);

  float ao_r = 1.0;
  float ao_r_sq = ao_r * ao_r;

  float sum = 0.0;
  for (int iy = -5; iy <= 5; iy += 1) {
    for (int ix = -5; ix <= 5; ix += 1) {
      vec2 sam_pos_in_tex = fs_pos_in_tex + vec2(iy * dy, ix * dx);

      vec3 frag_to_sam_in_cam = sample_pos_in_cam(sam_pos_in_tex) - pos_in_cam;
      float weight =
          max(0.0, 1.0 - dot(frag_to_sam_in_cam, frag_to_sam_in_cam) / ao_r_sq);

      filtered_ao += weight * sample_ao(sam_pos_in_tex);
    }
  }

  // NORMALS
  // frag_color = vec4(nor_in_cam * 0.5 + vec3(0.5), 1.0);

  // NO AO
  // frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);

  // APPLIED AO
  float ao_weight = filtered_ao.x / (filtered_ao.x + filtered_ao.y);
  frag_color = vec4(mix(texture(color_sampler, fs_pos_in_tex).rgb, vec3(0.0),
                        (1.0 - ao_weight)),
                    1.0);

  // ANIMATED APPLIED AO
  // float ao_weight = filtered_ao.x / (filtered_ao.x + filtered_ao.y);
  // frag_color = vec4(mix(texture(color_sampler, fs_pos_in_tex).rgb, vec3(0.0),
  //                       (cos(time * 3.14) + 1.0)/2.0 * (1.0 - ao_weight)),
  //                   1.0);

  // FILTERED AO
  // float ao_weight = filtered_ao.x / (filtered_ao.x + filtered_ao.y);
  // frag_color = vec4(vec3(ao_weight), 1.0);

  // UNFILTERED AO
  // uvec2 ao = sample_ao(fs_pos_in_tex);
  // float ao_weight = float(ao.x) / float(ao.x + ao.y);
  // // float ao_weight = 1 - float(ao.y) / 64.0;
  // frag_color = vec4(vec3(ao_weight), 1.0);
}
