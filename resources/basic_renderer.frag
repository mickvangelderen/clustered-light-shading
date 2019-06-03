uniform float highlight;

uniform sampler2D shadow_sampler;
uniform sampler2D diffuse_sampler;
uniform sampler2D normal_sampler;
uniform sampler2D specular_sampler;

uniform vec2 shadow_dimensions;
uniform vec2 diffuse_dimensions;
uniform vec2 normal_dimensions;
uniform vec2 specular_dimensions;

in vec3 fs_pos_in_cam;
in vec3 fs_pos_in_cls;
in vec2 fs_pos_in_tex;
in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_cam;
in vec3 fs_tan_in_cam;

struct PointLight {
  vec4 ambient;
  vec4 diffuse;
  vec4 specular;
  vec4 pos_in_cam;
  vec4 att;
};

layout(std140, binding = LIGHTING_BUFFER_BINDING) uniform LightingBuffer {
  PointLight point_lights[POINT_LIGHT_CAPACITY];
};

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec3 frag_nor_in_cam;

uvec3 separate_bits_by_2(uvec3 x) {
  // x       = 0b??????????????????????jihgfedcba
  // mask    = 0b00000000000000000000001111111111
  x &= 0x000003ff;
  // x       = 0b0000000000000000000000jihgfedcba
  // x << 10 = 0b000000000000jihgfedcba0000000000
  // mask    = 0b00000000000011111000000000011111
  x = (x | (x << 10)) & 0x00f8001f;
  // x       = 0b000000000000jihgf0000000000edcba
  // x << 04 = 0b00000000jihgf0000000000edcba0000
  // mask    = 0b00000000111000011000000111000011
  x = (x | (x << 04)) & 0x001e81c3;
  // x       = 0b00000000jih0000gf000000edc0000ba
  // x << 02 = 0b000000jih0000gf000000edc0000ba00
  // mask    = 0b00000011001001001000011001001001
  x = (x | (x << 02)) & 0x03248649;
  // x       = 0b000000ji00h00g00f0000ed00c00b00a
  // x << 02 = 0b0000ji00h00g00f0000ed00c00b00a00
  // mask    = 0b00001001001001001001001001001001
  x = (x | (x << 02)) & 0x09249249;
  // x       = 0b0000j00i00h00g00f00e00d00c00b00a
  return x;
}

uint to_morton_3(uvec3 p) {
  uvec3 q = separate_bits_by_2(p);
  return (q.z << 2) | (q.y << 1) | q.x;
}

float compute_visibility_classic(vec3 pos_in_lgt) {
  // Can make the bias depend on the angle between the light direction and the
  // surface normal but does that really help? The worst case scenario is not
  // improved. For now I just use a constant so at least I don't have to look
  // for errors here.
  float bias = 0.005;

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

vec3 point_light_contribution(PointLight point_light, vec3 nor_in_cam,
                              vec3 frag_pos_in_cam, vec3 cam_dir_in_cam_norm) {
  vec3 pos_from_frag_to_light = point_light.pos_in_cam.xyz - frag_pos_in_cam;
  vec3 light_dir_in_cam_norm = normalize(pos_from_frag_to_light);

  float I = point_light.att[0];
  float C = point_light.att[1];
  float R0 = point_light.att[2];
  float R1 = point_light.att[3]; // R1^2 = I/C

  // Attenuation.
  float d_sq_unclipped = dot(pos_from_frag_to_light, pos_from_frag_to_light);
  float d_unclipped = sqrt(d_sq_unclipped);
  float d_sq = max(R0, d_sq_unclipped);
  float d = max(R0, d_unclipped);

  float diffuse_attenuation;
  float specular_attenuation;

  if (d_unclipped < R1) {
    if (attenuation_mode == ATTENUATION_MODE_STEP) {
      diffuse_attenuation = I*(1.0/R0 + R0 - 1.0/R1)/R1;
    } else if (attenuation_mode == ATTENUATION_MODE_LINEAR) {
      // Linear doesn't go infinite so we can use the unclipped distance.
      diffuse_attenuation = I - (I/R1) * d_unclipped;
    } else if (attenuation_mode == ATTENUATION_MODE_PHYSICAL) {
      diffuse_attenuation = I / d_sq;
    } else if (attenuation_mode == ATTENUATION_MODE_INTERPOLATED) {
      diffuse_attenuation = I / d_sq - (C / R1) * d;
      // diffuse_attenuation = I / (d_sq + 1) - C * pow(d_sq / (R1 * R1), 1);
    } else if (attenuation_mode == ATTENUATION_MODE_REDUCED) {
      diffuse_attenuation = I / d_sq - C;
    } else if (attenuation_mode == ATTENUATION_MODE_SMOOTH) {
      diffuse_attenuation = I / d_sq - 3.0*C + (2.0*C/R1) * d;
    } else {
      // ERROR: Invalid attenuation_mode.
      diffuse_attenuation = 0.0;
    }
    specular_attenuation = I*(1.0 - d_sq/(R1*R1));
  } else {
    diffuse_attenuation = 0.0;
    specular_attenuation = 0.0;
  }

  // Diffuse.
  float diffuse_weight = max(0.0, dot(nor_in_cam, light_dir_in_cam_norm));

  // Specular.
  float specular_angle =
      max(0.0, dot(cam_dir_in_cam_norm,
                   reflect(-light_dir_in_cam_norm, nor_in_cam)));
  float specular_weight = pow(specular_angle, shininess);

  // LIGHT ATTENUATION.
  return vec3(diffuse_attenuation);

  // LIGHT CONTRIBUTION.
  return
      // Diffuse
      (diffuse_attenuation * diffuse_weight) * point_light.diffuse.rgb *
          texture(diffuse_sampler, fs_pos_in_tex).rgb +
      // Specular
      (specular_attenuation * specular_weight) * point_light.specular.rgb *
          texture(specular_sampler, fs_pos_in_tex).rgb;
}

void main() {
  // Perspective divide after interpolation.
  vec3 pos_in_lgt = fs_pos_in_lgt.xyz / fs_pos_in_lgt.w;

  // Common intermediates.
  vec3 light_dir_in_cam_norm = normalize(light_dir_in_cam);
  vec3 cam_dir_in_cam_norm = normalize(-fs_pos_in_cam);

  // Perturbed normal in camera space.
  // TODO: Consider https://github.com/mickvangelderen/vr-lab/issues/3
  vec3 fs_nor_in_cam_norm = normalize(fs_nor_in_cam);
  vec3 fs_bitan_in_cam_norm =
      cross(fs_nor_in_cam_norm, normalize(fs_tan_in_cam));
  vec3 fs_tan_in_cam_norm = cross(fs_bitan_in_cam_norm, fs_nor_in_cam_norm);
  mat3 dir_from_tan_to_cam =
      mat3(fs_tan_in_cam_norm, fs_bitan_in_cam_norm, fs_nor_in_cam_norm);
  vec3 nor_in_cam = dir_from_tan_to_cam * sample_nor_in_tan(fs_pos_in_tex);

  // Ambient. Fake light bouncing from sun light.
  float ambient_weight =
      max(0.0, dot(mat3(pos_from_wld_to_cam) * vec3(0.0, 1.0, 0.0),
                   light_dir_in_cam) *
                   0.3);

  // Diffuse.
  float diffuse_weight = max(dot(nor_in_cam, light_dir_in_cam_norm), 0.0);
  vec4 diffuse_sample = texture(diffuse_sampler, fs_pos_in_tex);
  if (diffuse_sample.a < 0.5) {
    discard;
  }
  vec3 diffuse_color = diffuse_sample.rgb;

  // Specular.
  float specular_angle =
      max(dot(cam_dir_in_cam_norm, reflect(-light_dir_in_cam_norm, nor_in_cam)),
          0.0);
  float specular_weight = pow(specular_angle, shininess);
  vec3 specular_color = texture(specular_sampler, fs_pos_in_tex).rgb;

  float visibility = compute_visibility_variance(pos_in_lgt);

  vec3 color_accumulator = vec3(0.0);
  // color_accumulator += (diffuse_color * ambient_weight) +
  //                      visibility * (diffuse_color * diffuse_weight +
  //                                    specular_color * specular_weight);

  uvec3 fs_idx_in_cls = uvec3(fs_pos_in_cls);

  // CLUSTER INDICES X, Y, Z
  // frag_color = vec4(vec3(fs_idx_in_cls)/vec3(cluster_dims), 1.0);

  // CLUSTER INDICES X, Y, Z mod 3
  // vec3 cluster_index_colors = vec3((fs_idx_in_cls % 3) + 1)/4.0;
  // frag_color = vec4(cluster_index_colors.xyz, 1.0);

  // CLUSTER INDICES X + Y + Z mod 2
  // frag_color = vec4(
  //     vec3(float((fs_idx_in_cls.x + fs_idx_in_cls.y + fs_idx_in_cls.z) & 1)),
  //     1.0);

  // CLUSTER MORTON INDEX
  // uint cluster_morton_index = to_morton_3(fs_idx_in_cls);
  // frag_color = vec4(                              //
  //     float((cluster_morton_index >> 16) & 0xff) / 255.0, //
  //     float((cluster_morton_index >> 8) & 0xff) / 255.0,  //
  //     float((cluster_morton_index >> 0) & 0xff) / 255.0, 1.0);

  uint cluster_index =
      (((fs_idx_in_cls.z * cluster_dims.y) + fs_idx_in_cls.y) * cluster_dims.x +
       fs_idx_in_cls.x) *
      cluster_dims.w;

  uint cluster_length = clusters[cluster_index];
  // uint cluster_length = 8;

  // CLUSTER LENGHTS
  frag_color = vec4(vec3(float(cluster_length) / 8.0), 1.0);

  // CLUSTERED SHADING
  for (uint i = 0; i < cluster_length; i++) {
    // uint light_index = clusters[cluster_index + 1 + i];
    uint light_index = i;

    float t_on = 5.0;
    float t = float(cluster_length);
    float tf = fract((float(time)*point_lights[i].att.x*1.0 - float(i))/t)*t/t_on;
    float anim = step(tf, 1.0) * (cos((tf*2.0 - 1.0)*3.1415)*0.5 + 0.5);

    color_accumulator += 
    // color_accumulator += anim *
        point_light_contribution(point_lights[light_index], nor_in_cam,
                                 fs_pos_in_cam, cam_dir_in_cam_norm);
  }
  frag_color = vec4(color_accumulator, 1.0);

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
