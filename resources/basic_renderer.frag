uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;
uniform float highlight;
uniform vec3 sun_dir_in_cam;

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform sampler2D shadow_sampler;

in vec3 fs_pos_in_cam;
in vec2 fs_pos_in_tex;
in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_cam;
in vec3 fs_tan_in_cam;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec3 frag_nor_in_cam;

float compute_visibility_classic(vec3 pos_in_lgt, float bias) {
  float closest_depth_in_lgt =
      texture(shadow_sampler, pos_in_lgt.xy * 0.5 + 0.5).x;
  float frag_depth_in_lgt = pos_in_lgt.z * 0.5 + 0.5;
  return step(frag_depth_in_lgt, closest_depth_in_lgt + bias);
}

float compute_visibility_variance(vec3 pos_in_lgt) {
  float frag_depth = pos_in_lgt.z * 0.5 + 0.5;
  vec2 sam = texture(shadow_sampler, pos_in_lgt.xy * 0.5 + 0.5).xy;
  float mean_depth = sam.x;
  float mean_depth_sq = sam.y;
  float variance = mean_depth_sq - mean_depth * mean_depth;
  float diff = frag_depth - mean_depth;
  return (diff > 0.0) ? variance / (variance + diff * diff) : 1.0;
}

void main() {
  vec3 pos_in_lgt = fs_pos_in_lgt.xyz / fs_pos_in_lgt.w;

  vec3 light_dir_in_cam_normalized = normalize(sun_dir_in_cam);
  // vec3 light_pos_in_cam = vec3(0.0);
  // vec3 light_dir_in_cam_normalized =
  //     normalize(light_pos_in_cam - fs_pos_in_cam);
  vec3 cam_dir_in_cam_normalized = normalize(-fs_pos_in_cam);

  vec3 nor_in_cam = normalize(fs_nor_in_cam);

  float ambient_weight = 0.2;
  float diffuse_weight = max(dot(nor_in_cam, light_dir_in_cam_normalized), 0.0);
  float specular_angle =
      max(dot(cam_dir_in_cam_normalized,
              reflect(-light_dir_in_cam_normalized, nor_in_cam)),
          0.0);
  float specular_weight = pow(specular_angle, shininess);

  // vec3 diffuse_color = texture(diffuse_sampler, fs_pos_in_tex).rgb;
  vec3 diffuse_color;
  vec4 diffuse_sample = texture(diffuse_sampler, fs_pos_in_tex);
  if (diffuse_sample != vec4(0.0, 0.0, 0.0, 1.0)) {
    diffuse_color = diffuse_sample.rgb;
  } else {
    diffuse_color = diffuse;
  }

  // Can be computed as sqrt(1/dot(l, n)^2 - 1) but this is probably cheaper.
  // The width of a shadow map texel.
  // Bias is clamped between 0.0 and 0.1 because it can technically go to
  // infinity when dot(l, n) is 0.
  // float shadow_map_texel_width_in_cam = 0.005;
  // float bias = clamp(shadow_map_texel_width_in_cam *
  // tan(acos(diffuse_weight)),
  //                    0.0, 0.005);
  float bias = 0.005;

  float visibility = compute_visibility_variance(pos_in_lgt);
  frag_color = vec4((diffuse_color * ambient_weight) +
                        visibility * (diffuse_color * diffuse_weight +
                                      specular * specular_weight),
                    1.0);

  // if (pos_in_lgt.x < -1.0 || pos_in_lgt.x > 1.0 || //
  //     pos_in_lgt.y < -1.0 || pos_in_lgt.y > 1.0) {
  //   // Need a larger shadow xy frustrum.
  //   frag_color = vec4(1.0, 0.0, 0.0, 1.0);
  // }
  // if (pos_in_lgt.z < -1.0 || pos_in_lgt.z > 1.0) {
  //   // Need a larger shadow z frustrum.
  //   frag_color = vec4(0.0, 1.0, 0.0, 1.0);
  // }

  // frag_color = vec4(diffuse_color, 1.0);
  // frag_color = (vec4(nor_in_cam, 1.0) + vec4(1.0)) / 2.0;
  frag_nor_in_cam = nor_in_cam * 0.5 + vec3(0.5);
}
