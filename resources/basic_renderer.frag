uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;
uniform float highlight;

uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform sampler2D shadow_sampler;

in vec3 fs_pos_in_cam;
in vec2 fs_pos_in_tex;
in vec3 fs_pos_in_lgt;
in vec3 fs_nor_in_cam;
in vec3 fs_tan_in_cam;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec3 frag_nor_in_cam;

float compute_shadow_classic() {
  float frag_depth_in_lgt = fs_pos_in_lgt.z * 0.5 + 0.5;
  float closest_depth_in_lgt =
      texture(shadow_sampler, fs_pos_in_lgt.xy * 0.5 + 0.5).x;
  frag_color = vec4(10 * vec3(frag_depth_in_lgt - closest_depth_in_lgt), 1.0);
  float bias = 0.002;
  return step(frag_depth_in_lgt, closest_depth_in_lgt + bias);
}

float compute_shadow_variance() {
  float frag_depth = fs_pos_in_lgt.z * 0.5 + 0.5;
  vec2 sam = texture(shadow_sampler, fs_pos_in_lgt.xy * 0.5 + 0.5).xy;
  float mean_depth = sam.x;
  float mean_depth_sq = sam.y;
  float variance = mean_depth_sq - mean_depth * mean_depth;
  float diff = frag_depth - mean_depth;
  if (diff > 0.0) {
    return variance / (variance + diff * diff);
  } else {
    return 1.0;
  }
}

void main() {
  vec3 light_pos_in_cam = vec3(0.0);
  vec3 light_dir_in_cam_normalized =
      normalize(light_pos_in_cam - fs_pos_in_cam);
  vec3 cam_dir_in_cam_normalized = normalize(-fs_pos_in_cam);

  vec3 nor_in_cam = normalize(fs_nor_in_cam);

  float ambient_weight = 0.1;
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

  float in_shadow = compute_shadow_variance();
  // frag_color = vec4(vec3(in_shadow), 1.0);
  frag_color = vec4(
      ambient * ambient_weight //
          + mix((diffuse_color * diffuse_weight + specular * specular_weight),
                vec3(0.0), (1.0 - in_shadow) * 0.4),
      1.0);

  // frag_color = vec4(diffuse_color, 1.0);
  // frag_color = (vec4(nor_in_cam, 1.0) + vec4(1.0)) / 2.0;
  // frag_nor_in_cam = nor_in_cam * 0.5 + vec3(0.5);

  // frag_color = vec4(texture(shadow_sampler, fs_pos_in_lgt.xy).rrr *
  // 1000, 1.0);
}
