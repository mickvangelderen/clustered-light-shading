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
uniform sampler2D random_unit_sphere_surface_sampler;

#define HBAO_KERNEL_BINDING 0

layout(std140, binding = HBAO_KERNEL_BINDING) uniform HBAO_Kernel {
  vec4 hbao_kernel[128];
};

in vec2 fs_pos_in_tex;

layout(location = 0) out uvec2 fs_ao;

float sample_z_ndc(vec2 pos_in_tex) {
  return texture(depth_sampler, pos_in_tex).r * 2.0 - 1.0;
}

vec2 pos_from_cam_to_tex(vec3 pos_in_cam) {
  float x = pos_in_cam.x;
  float y = pos_in_cam.y;
  float z = pos_in_cam.z;
  float s = -z0 / z;

  return vec2((s * x - x0) / (x1 - x0), (s * y - y0) / (y1 - y0));
}

// Reverse projection matrix.
float z_from_ndc_to_cam(float z_ndc) {
  return (2.0 * z0 * z1) / (z_ndc * (z1 - z0) - (z0 + z1));
}

vec3 sample_pos_in_cam(vec2 pos_in_tex) {
  float z_ndc = sample_z_ndc(pos_in_tex);
  // This is z_from_ndc_to_cam(z_ndc) / -z0
  float s = (-2.0 * z1) / (z_ndc * (z1 - z0) - (z0 + z1));
  return s * vec3(mix(x0, x1, pos_in_tex.x), mix(y0, y1, pos_in_tex.y), -z0);
}

vec3 sample_nor_in_cam(vec2 pos_in_tex) {
  vec3 sam = texture(nor_in_cam_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
}

vec3 sample_random_normal() {
  return texture(random_unit_sphere_surface_sampler, gl_FragCoord.xy / 256.0)
                 .xyz *
             2.0 -
         vec3(1.0);
}

void main() {
  vec3 pos_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 nor_in_cam = sample_nor_in_cam(fs_pos_in_tex);

  uint occlude_count = 0;
  uint visible_count = 0;

  uint ix = uint(gl_FragCoord.x);
  uint iy = uint(gl_FragCoord.y);
  uint kernel_offset = ((iy % 2) * 2 + (ix % 2)) * 32;
  // uint kernel_offset = 0;
  vec3 random_normal = sample_random_normal();

  float radius = 1.0;
  float radius_sq = radius * radius;

  for (int i = 0; i < 32; i += 1) {
    // vec3 kernel_sample = hbao_kernel[i].xyz * radius;
    vec3 kernel_sample =
        reflect(hbao_kernel[kernel_offset + i].xyz, random_normal) * radius;

    // k/dot(nor_in_cam, nor_in_cam) = k is the signed distance from
    // kernel_sample to the plane defined by nor_in_cam.
    float k = dot(nor_in_cam, kernel_sample);
    // Mirror kernel_sample across (0, 0, 0) to move it to the positive
    // hemisphere.
    vec3 sam_pos_in_cam = pos_in_cam +
                          (k < 0.0 ? -kernel_sample : kernel_sample) +
                          0.01 * nor_in_cam;

    vec2 sam_pos_in_tex = pos_from_cam_to_tex(sam_pos_in_cam);
    vec3 hit_pos_in_cam = sample_pos_in_cam(sam_pos_in_tex);
    vec3 hit_ray = hit_pos_in_cam - pos_in_cam;

    // NOTE: z are negative!!!
    if (hit_pos_in_cam.z > sam_pos_in_cam.z) {
      // Sample is occluded, but the occluder might not be in range of the
      // hemisphere.
      if ((dot(hit_ray, hit_ray) < radius_sq) &&
          (dot(hit_ray, nor_in_cam) >= 0.0)) {
        // Hit is within positive hemisphere.
        occlude_count += 1;
      } else {
        // Hit is not within positive hemisphere.
        // Sample is not trustworthy.
      }
    } else {
      // Sample must be visible.
      visible_count += 1;
    }
  }

  fs_ao = uvec2(visible_count, occlude_count);
}
