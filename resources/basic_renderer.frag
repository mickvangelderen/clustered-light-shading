uniform float highlight;
uniform vec3 sun_dir_in_cam;

uniform sampler2D shadow_sampler;
uniform sampler2D diffuse_sampler;
uniform sampler2D normal_sampler;
uniform sampler2D specular_sampler;
uniform float shininess;

uniform vec2 shadow_dimensions;
uniform vec2 diffuse_dimensions;
uniform vec2 normal_dimensions;
uniform vec2 specular_dimensions;

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
  float frag_depth_in_lgt = 1.0 - pos_in_lgt.z;
  return step(frag_depth_in_lgt, closest_depth_in_lgt + bias);
}

float compute_visibility_variance(vec3 pos_in_lgt) {
  float frag_depth = 1.0 - pos_in_lgt.z;
  vec2 sam = texture(shadow_sampler, pos_in_lgt.xy * 0.5 + 0.5).xy;
  float mean_depth = sam.x;
  float mean_depth_sq = sam.y;
  float variance = mean_depth_sq - mean_depth * mean_depth;
  float diff = frag_depth - mean_depth;
  return (diff > 0.0) ? variance / (variance + diff * diff) : 1.0;
}

// FIXME
vec3 sample_nor_in_tan(vec2 pos_in_tex) {
  float dx = 1.0 / normal_dimensions.x;
  float dy = 1.0 / normal_dimensions.y;

  float v00 = texture(normal_sampler, pos_in_tex + vec2(-dx, -dy)).x;
  float v01 = texture(normal_sampler, pos_in_tex + vec2(0.0, -dy)).x;
  float v02 = texture(normal_sampler, pos_in_tex + vec2(dx, -dy)).x;
  float v10 = texture(normal_sampler, pos_in_tex + vec2(-dx, 0.0)).x;
  // v11
  float v12 = texture(normal_sampler, pos_in_tex + vec2(dx, 0.0)).x;
  float v20 = texture(normal_sampler, pos_in_tex + vec2(-dx, dy)).x;
  float v21 = texture(normal_sampler, pos_in_tex + vec2(0.0, dy)).x;
  float v22 = texture(normal_sampler, pos_in_tex + vec2(dx, dy)).x;

  float x = (v02 - v00) + 2.0 * (v12 - v10) + (v22 - v20);
  float y = (v20 - v00) + 2.0 * (v21 - v01) + (v22 - v02);

  return normalize(vec3(-x, -y, 1.0));
}

void main() {
  // Perspective divide after interpolation.
  vec3 pos_in_lgt = fs_pos_in_lgt.xyz / fs_pos_in_lgt.w;

  // Common intermediates.
  vec3 light_dir_in_cam_normalized = normalize(sun_dir_in_cam);
  vec3 cam_dir_in_cam_normalized = normalize(-fs_pos_in_cam);

  // Perturbed normal in camera space.
  // TODO: Consider https://github.com/mickvangelderen/vr-lab/issues/3
  vec3 fs_nor_in_cam_norm = normalize(fs_nor_in_cam);
  vec3 fs_bitan_in_cam_norm =
      cross(fs_nor_in_cam_norm, normalize(fs_tan_in_cam));
  vec3 fs_tan_in_cam_norm = cross(fs_bitan_in_cam_norm, fs_nor_in_cam_norm);
  mat3 dir_from_tan_to_cam =
      mat3(fs_tan_in_cam_norm, fs_bitan_in_cam_norm, fs_nor_in_cam_norm);
  vec3 nor_in_cam = dir_from_tan_to_cam * sample_nor_in_tan(fs_pos_in_tex);

  // Ambient.
  float ambient_weight = 0.2;

  // Diffuse.
  float diffuse_weight = max(dot(nor_in_cam, light_dir_in_cam_normalized), 0.0);
  vec4 diffuse_sample = texture(diffuse_sampler, fs_pos_in_tex);
  if (diffuse_sample.a < 0.5) {
    discard;
  }
  vec3 diffuse_color = diffuse_sample.rgb;

  // Specular.
  float specular_angle =
      max(dot(cam_dir_in_cam_normalized,
              reflect(-light_dir_in_cam_normalized, nor_in_cam)),
          0.0);
  float specular_weight = pow(specular_angle, shininess);
  vec3 specular_color = texture(specular_sampler, fs_pos_in_tex).rgb;

  // Can make the bias depend on the angle between the light direction and the
  // surface normal but does that really help? The worst case scenario is not
  // improved. For now I just use a constant so at least I don't have to look
  // for errors here.
  float bias = 0.005;

  float visibility = compute_visibility_variance(pos_in_lgt);
  frag_color = vec4((diffuse_color * ambient_weight) +
                        visibility * (diffuse_color * diffuse_weight +
                                      specular_color * specular_weight),
                    1.0);

  // DIFFUSE TEXTURE
  // frag_color = texture(diffuse_sampler, fs_pos_in_tex);

  // NORMAL TEXURE
  // frag_color = texture(normal_sampler, fs_pos_in_tex);

  // SPECULAR_TEXTURE
  // frag_color = texture(specular_sampler, fs_pos_in_tex);

  // NORMAL IN CAMERA SPACE
  // frag_color = vec4(nor_in_cam, 1.0);

  frag_nor_in_cam = nor_in_cam * 0.5 + vec3(0.5);
}
