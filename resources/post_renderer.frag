uniform int width;
uniform int height;
uniform sampler2D color_sampler;
uniform sampler2D depth_sampler;
uniform sampler2D nor_in_cam_sampler;
uniform sampler2D ao_sampler;

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
  float ao = texture(ao_sampler, fs_pos_in_tex).r;

  // NORMALS
  // frag_color = vec4(nor_in_cam * 0.5 + vec3(0.5), 1.0);

  // NO AO
  // frag_color = vec4(texture(color_sampler, fs_pos_in_tex).rgb, 1.0);

  // APPLIED AO
  frag_color = vec4(ao * texture(color_sampler, fs_pos_in_tex).rgb, 1.0);

  // AO
  // frag_color = vec4(vec3(ao), 1.0);

  // GAMMA SANITY
  // if (fs_pos_in_tex.x < 0.5) {
  //   float x = fs_pos_in_tex.x * 2.0;
  //   if (fs_pos_in_tex.y < 0.3) {
  //     frag_color = vec4(vec3(pow(x, 2.4)), 1.0);
  //   } else if (fs_pos_in_tex.y < 0.6) {
  //     frag_color = vec4(vec3(x), 1.0);
  //   } else {
  //     frag_color = vec4(vec3(pow(x, 1.0/2.4)), 1.0);
  //   }
  // } else {
  //   if (fs_pos_in_tex.y < 0.5) {
  //     uvec2 fr = uvec2(gl_FragCoord.xy);
  //     frag_color = vec4(vec3(float((fr.x + fr.y) % 2)), 1.0);
  //   } else {
  //     frag_color = vec4(vec3(0.5), 1.0);
  //   }
  // }
}
