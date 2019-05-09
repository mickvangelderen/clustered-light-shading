uniform float time;
uniform int width;
uniform int height;
uniform mat4 pos_from_cam_to_clp;
uniform mat4 pos_from_clp_to_cam;
uniform sampler2D color_sampler;
uniform sampler2D depth_sampler;
uniform sampler2D nor_in_cam_sampler;
uniform sampler2D random_unit_sphere_surface_sampler;

layout(std140, binding = AO_SAMPLE_BUFFER_BINDING) buffer SphereSamples {
  vec4 ao_samples[512];
};

in vec2 fs_pos_in_tex;

layout(location = 0) out float fs_ao;

vec2 pos_from_cam_to_tex(vec3 pos_in_cam) {
  vec4 p_clp = pos_from_cam_to_clp * vec4(pos_in_cam, 1.0);
  return (p_clp.xy / p_clp.w) * 0.5 + vec2(0.5);
}

vec3 sample_pos_in_cam(vec2 pos_in_tex) {
  float z_ndc = texture(depth_sampler, pos_in_tex).r;

  float a = pos_from_cam_to_clp[2][2];
  float b = pos_from_cam_to_clp[3][2];
  float c = pos_from_cam_to_clp[2][3];
  float d = pos_from_cam_to_clp[3][3];

  float w_clp = (b * c - a * d) / (c * z_ndc - a);
  vec4 p_ndc = vec4(                   //
      pos_in_tex.xy * 2.0 - vec2(1.0), //
      z_ndc,                           //
      1.0                              //
  );

  return mat4x3(pos_from_clp_to_cam) * (w_clp * p_ndc);
}

vec3 sample_nor_in_cam(vec2 pos_in_tex) {
  vec3 sam = texture(nor_in_cam_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
}

vec3 sample_random_normal(vec2 pos_in_tex) {
  vec3 sam = texture(random_unit_sphere_surface_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
}

void main() {
  vec3 pos_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 nor_in_cam = sample_nor_in_cam(fs_pos_in_tex);

  uint ix = uint(gl_FragCoord.x);
  uint iy = uint(gl_FragCoord.y);
  // uint sample_base = ((iy % 8) * 8 + (ix % 8)) * 64;
  uint sample_base = 0;
  vec3 random_normal = sample_random_normal(gl_FragCoord.xy / 256.0);
  // vec3 random_normal = vec3(0.0, 0.0, 1.0);

  float radius = 1.0;
  float radius_sq = radius * radius;

  uint visible_count = 0;
  uint total_count = 0;

  const uint N = 16;
  for (int i = 0; i < N; i += 1) {
    // NOTE: Instead of having N random samples for every pixel, we have N
    // random samples per group of pixels, and reflect the samples along a
    // random normal of which we have 1 per pixel. I could also try a PRNG.
    vec3 ao_sample =
        reflect(ao_samples[sample_base + i].xyz, random_normal) * radius;

    // NOTE: The samples are distributed over a sphere, we use the fragment
    // normal to mirror these samples across the origin to put them in the
    // positive hemisphere.
    float mirror = dot(nor_in_cam, ao_sample) < 0.0 ? -1.0 : 1.0;

    vec3 sam_pos_in_cam = pos_in_cam + mirror * ao_sample + 0.01 * nor_in_cam;

    // PERF: Can manually do these computations but hopefully the optimizer does
    // it for us.
    vec2 sam_pos_in_tex = pos_from_cam_to_tex(sam_pos_in_cam);
    vec3 hit_pos_in_cam = sample_pos_in_cam(sam_pos_in_tex);
    vec3 hit_ray = hit_pos_in_cam - pos_in_cam;

    // NOTE: Depth values are negative.
    bool is_sample_visible = hit_pos_in_cam.z < sam_pos_in_cam.z;

    // NOTE: Test if sample position is within the AO sphere and if it is in the
    // positive hemisphere.
    bool is_hit_within_hemi = (dot(hit_ray, hit_ray) < radius_sq) &&
                              (dot(hit_ray, nor_in_cam) >= 0.0);

    // NOTE: Samples that are occluded but not in the positive hemisphere are
    // unreliable. They do not count towards neither occlusion nor visibility.
    visible_count += is_sample_visible ? 1 : 0;
    total_count += is_sample_visible || is_hit_within_hemi ? 1 : 0;
  }

  // NOTE: We assume a sample is visible if we don't have any usable samples.
  fs_ao = total_count > 0 ? float(visible_count) / float(total_count) : 1.0;
}
