uniform float time;
uniform int width;
uniform int height;
uniform sampler2D color_sampler;
uniform sampler2D depth_sampler;
uniform sampler2D nor_in_cam_sampler;
uniform sampler2D ao_sampler;
uniform mat4 pos_from_clp_to_cam;
uniform mat4 pos_from_cam_to_clp;

in vec2 fs_pos_in_tex;

out vec4 frag_color;

float lerp(float x, float x0, float x1, float y0, float y1) {
  return ((x - x0) * y1 + (x1 - x) * y0) / (x1 - x0);
}

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

uvec2 sample_ao(vec2 pos_in_tex) {
  return uvec2(texture(ao_sampler, pos_in_tex).xy);
}

vec3 sample_nor_in_cam(vec2 pos_in_tex) {
  vec3 sam = texture(nor_in_cam_sampler, pos_in_tex).xyz;
  return sam * 2.0 - vec3(1.0);
}

void main() {
  vec3 pos_in_cam = sample_pos_in_cam(fs_pos_in_tex);
  vec3 nor_in_cam = sample_nor_in_cam(fs_pos_in_tex);

  float dx = 1.0 / width;
  float dy = 1.0 / height;

  float filtered_ao = 0.0;

  float ao_r = 1.0;
  float ao_r_sq = ao_r * ao_r;

  const float[25] weights = float[25](                  //
      0.003765, 0.015019, 0.023792, 0.015019, 0.003765, //
      0.015019, 0.059912, 0.094907, 0.059912, 0.015019, //
      0.023792, 0.094907, 0.150342, 0.094907, 0.023792, //
      0.015019, 0.059912, 0.094907, 0.059912, 0.015019, //
      0.003765, 0.015019, 0.023792, 0.015019, 0.003765  //
  );

  for (int iy = -2; iy <= 2; iy += 1) {
    for (int ix = -2; ix <= 2; ix += 1) {
      vec2 sam_pos_in_tex = fs_pos_in_tex + vec2(iy * dy, ix * dx);
      // float z_diff = sample_pos_in_cam(sam_pos_in_tex).z - pos_in_cam.z;
      // float z_weight = max(0.0, 1.0 - (z_diff * z_diff / ao_r_sq));

      filtered_ao += weights[(iy + 2) * 5 + ix + 2] *
                     texture(ao_sampler, sam_pos_in_tex).r;
    }
  }

  // NORMALS
  // frag_color = vec4(nor_in_cam * 0.5 + vec3(0.5), 1.0);

  // NO AO
  // frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);

  // APPLIED AO
  frag_color =
      vec4(filtered_ao * texture(color_sampler, fs_pos_in_tex).rgb, 1.0);

  // APPLIED UNFILTERED AO
  // uvec2 ao = sample_ao(fs_pos_in_tex);
  // float ao_weight = float(ao.x) / float(ao.x + ao.y);
  // frag_color = vec4(mix(texture(color_sampler, fs_pos_in_tex).rgb, vec3(0.0),
  //                       (1.0 - ao_weight)),
  //                   1.0);

  // ANIMATED APPLIED AO
  // float ao_weight = filtered_ao.x / (filtered_ao.x + filtered_ao.y);
  // frag_color = vec4(mix(texture(color_sampler, fs_pos_in_tex).rgb, vec3(0.0),
  //                       (cos(time * 3.14) + 1.0)/2.0 * (1.0 - ao_weight)),
  //                   1.0);

  // FILTERED AO
  // frag_color = vec4(vec3(filtered_ao), 1.0);

  // UNFILTERED AO
  // frag_color = vec4(vec3(texture(ao_sampler, fs_pos_in_tex).r), 1.0);
}
