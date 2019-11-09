#include "cotangent_frame.glsl"
#include "pbr.glsl"

layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = SHININESS_LOC) uniform float shininess;

layout(binding = NORMAL_SAMPLER_BINDING) uniform sampler2D normal_sampler;
layout(binding = EMISSIVE_SAMPLER_BINDING) uniform sampler2D emissive_sampler;
layout(binding = AMBIENT_SAMPLER_BINDING) uniform sampler2D ambient_sampler;
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;
layout(binding = SPECULAR_SAMPLER_BINDING) uniform sampler2D specular_sampler;

in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec3 fs_bin_in_lgt;
in vec3 fs_tan_in_lgt;
in vec2 fs_pos_in_tex;

layout(location = 0) out vec4 frag_color;

vec3 sample_nor_in_tan(vec2 pos_in_tex) {
  vec2 xy = texture(normal_sampler, pos_in_tex).xy * 2.0 - vec2(1.0);
  float z = sqrt(max(0.0, 1.0 - dot(xy, xy)));
  return vec3(xy, z);
}

vec3 cook_torrance(vec3 kd, float roughness, float metalness, vec3 N, vec3 V, vec3 L) {
  vec3 F0 = vec3(0.04);
  F0 = mix(F0, kd, metalness);

  // calculate per-light radiance
  vec3 H = normalize(V + L);
  // float distance = length(lightPositions[i] - WorldPos);
  // float attenuation = 1.0 / (distance * distance);
  // vec3 radiance = lightColors[i] * attenuation;
  vec3 radiance = vec3(3.0);

  // Cook-Torrance BRDF
  float NDF = DistributionGGX(N, H, roughness);
  float G   = GeometrySmith(N, V, L, roughness);
  vec3 F    = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);

  float NdotV = max(dot(N, V), 0.0);
  float NdotL = max(dot(N, L), 0.0);
  vec3 specular = NDF * G * F / max(4 * NdotV * NdotL, 0.001); // prevent divide by zero for NdotV=0.0 or NdotL=0.0

  vec3 kD = (vec3(1.0) - F) * (1.0 - metalness);

  return (kD * kd / PI + specular) * radiance * NdotL;
}

void main() {
  vec3 frag_pos_in_lgt = fs_pos_in_lgt.xyz/fs_pos_in_lgt.w;
  vec3 frag_geo_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec3 frag_geo_bin_in_lgt = normalize(fs_bin_in_lgt);
  vec3 frag_geo_tan_in_lgt = normalize(fs_tan_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;
  vec3 frag_nor_in_tan = sample_nor_in_tan(frag_pos_in_tex);

  vec4 ka = texture(ambient_sampler, frag_pos_in_tex);
  vec4 ke = texture(emissive_sampler, frag_pos_in_tex);
  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);
  vec4 ks = texture(specular_sampler, frag_pos_in_tex);

  #if defined(BASIC_PASS)
  #if BASIC_PASS == BASIC_PASS_MASKED
  if (kd.a < 0.5) {
    discard;
  }
  #endif
  #else
  #error BASIC_PASS is undefined.
  #endif

  mat3 tbn = mat3(frag_geo_tan_in_lgt, frag_geo_bin_in_lgt, frag_geo_nor_in_lgt);
  vec3 frag_nor_in_lgt = normalize(tbn * frag_nor_in_tan);
  vec3 frag_to_cam_nor = normalize(cam_pos_in_lgt.xyz - frag_pos_in_lgt);

  vec3 light_pos_in_lgt = (cam_to_wld * vec4(0.0, 0.5, -1.5, 1.0)).xyz;
  vec3 frag_to_light_nor = normalize(light_pos_in_lgt - frag_pos_in_lgt);

  vec3 Lo = vec3(ke.xyz);
  Lo += cook_torrance(kd.xyz, ks.y, ks.z, frag_nor_in_lgt, frag_to_cam_nor, frag_to_light_nor);

  frag_color = vec4(Lo, 1.0);

  // frag_color = vec4(frag_nor_in_lgt * 0.5 + 0.5, 1.0);
  // frag_color = vec4(frag_bin_in_lgt * 0.5 + 0.5, 1.0);
  // frag_color = vec4(frag_tan_in_lgt * 0.5 + 0.5, 1.0);
  // frag_color = vec4(frag_nor_in_tan * 0.5 + 0.5, 1.0);
  // frag_color = vec4(frag_nor_in_lgt * 0.5 + 0.5, 1.0);
  // frag_color = vec4(frag_pos_in_tex, 0.0, 1.0);

  // frag_color = vec4(ke.xyz, 1.0);
  // frag_color = vec4(ka.xyz, 1.0);
  // frag_color = vec4(kd.xyz, 1.0);
  // frag_color = vec4(ks.xyz, 1.0);
}

