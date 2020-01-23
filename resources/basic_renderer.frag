#include "native/ATTENUATION_MODE"
#include "native/RENDER_TECHNIQUE"
#include "native/PROFILING"
#include "native/DEPTH_PREPASS"

#include "common.glsl"
#include "heatmap.glsl"
#include "light_buffer.glsl"
#include "pbr.glsl"

#if defined(RENDER_TECHNIQUE_CLUSTERED)
#include "cls/cluster_space_buffer.glsl"
#include "cls/cluster_maybe_active_cluster_indices_buffer.glsl"
#include "cls/active_cluster_light_counts_buffer.glsl"
#include "cls/active_cluster_light_offsets_buffer.glsl"
#include "cls/light_indices_buffer.glsl"
#endif

#if !defined(PROFILING_TIME_SENSITIVE)
#error PROFILING_TIME_SENSITIVE is not defined.
#endif

#if !defined(BASIC_PASS)
#error BASIC_PASS is not defined.
#endif

#if !defined(BASIC_PASS_OPAQUE)
#error BASIC_PASS_OPAQUE is not defined.
#endif

#if !defined(BASIC_PASS_MASKED)
#error BASIC_PASS_MASKED is not defined.
#endif

#if !defined(DEPTH_PREPASS)
#error DEPTH_PREPASS is not defined.
#endif

#if !PROFILING_TIME_SENSITIVE
layout(binding = BASIC_ATOMIC_BINDING, offset = 0) uniform atomic_uint shading_ops;
layout(binding = BASIC_ATOMIC_BINDING, offset = 4) uniform atomic_uint lighting_ops;
#endif

#if BASIC_PASS == BASIC_PASS_OPAQUE
// NOTE(mickvangelderen) Should be the default, unless using side
// effects like the atomic writes for atemporal profiling where we have
// to force this behaviour.
layout(early_fragment_tests) in;
#endif

#if BASIC_PASS == BASIC_PASS_MASKED
// NOTE(mickvangelderen) Can only force this on when we disable depth
// writes because of how early-z is implemented.
#if DEPTH_PREPASS
layout(early_fragment_tests) in;
#endif
#endif

#if BASIC_PASS == BASIC_PASS_TRANSPARENT
// NOTE(mickvangelderen) For transparent shading we should always
// have depth writes disabled?
layout(early_fragment_tests) in;
#endif

layout(binding = NORMAL_SAMPLER_BINDING) uniform sampler2D normal_sampler;
layout(binding = EMISSIVE_SAMPLER_BINDING) uniform sampler2D emissive_sampler;
layout(binding = AMBIENT_SAMPLER_BINDING) uniform sampler2D ambient_sampler;
layout(binding = DIFFUSE_SAMPLER_BINDING) uniform sampler2D diffuse_sampler;
layout(binding = SPECULAR_SAMPLER_BINDING) uniform sampler2D specular_sampler;
// layout(binding = SHADOW_SAMPLER_BINDING) uniform samplerCubeShadow shadow_sampler;
layout(binding = SHADOW_SAMPLER_BINDING) uniform samplerCube shadow_sampler;

layout(location = CAM_POS_IN_LGT_LOC) uniform vec3 cam_pos_in_lgt;

in vec3 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec3 fs_bin_in_lgt;
in vec3 fs_tan_in_lgt;
in vec2 fs_pos_in_tex;
#if defined(RENDER_TECHNIQUE_CLUSTERED)
in vec3 fs_pos_in_clu_cam;
#endif

layout(location = 0) out vec4 frag_color;

vec3 sample_nor_in_tan(vec2 pos_in_tex) {
  vec2 xy = texture(normal_sampler, pos_in_tex).xy * 2.0 - vec2(1.0);
  float z = sqrt(max(0.0, 1.0 - dot(xy, xy)));
  return vec3(xy, z);
}

float point_light_attenuate(PointLight point_light, vec3 frag_pos) {
  vec3 pos_from_frag_to_light = point_light.position - frag_pos;

  float I = point_light.i;
  float I0 = point_light.i0;
  float R0 = point_light.r0;
  float R1 = point_light.r1; // sqrt(I/I0)

  // Attenuation.
  float d_sq_unclipped = dot(pos_from_frag_to_light, pos_from_frag_to_light);
  float d_unclipped = sqrt(d_sq_unclipped);

  float d_sq = max(d_sq_unclipped, R0*R0);
  float d = max(d_unclipped, R0);

#if defined(ATTENUATION_MODE_STEP)
  // Use physical intensity halfway between 0 and R1
  float attenuation = d_sq_unclipped < R1*R1 ? 4*I/(R1*R1) : 0.0;
#elif defined(ATTENUATION_MODE_LINEAR)
  // Linear doesn't go infinite so we can use the unclipped distance.
  float attenuation = d_unclipped < R1 ? I/2.0*(R1 - d_unclipped)/(R1 - 1.0) : 0.0;
#elif defined(ATTENUATION_MODE_PHYSICAL)
  float attenuation = I / d_sq;
#elif defined(ATTENUATION_MODE_REDUCED)
  float attenuation = d_sq < R1*R1 ? I / d_sq - I0 : 0.0;
#elif defined(ATTENUATION_MODE_PHY_RED_1)
  float attenuation = d_sq < R1*R1 ? I / d_sq - I0/R1*d : 0.0;
#elif defined(ATTENUATION_MODE_PHY_RED_2)
  float attenuation = d_sq < R1*R1 ? I / d_sq - (I0 / (R1*R1)) * d_sq : 0.0;
#elif defined(ATTENUATION_MODE_SMOOTH)
  float attenuation = d_sq < R1*R1 ? I / d_sq + (2.0 * I0 / R1) * d - 3.0 * I0 : 0.0;
#elif defined(ATTENUATION_MODE_PHY_SMO_1)
  float attenuation = d_sq < R1*R1 ? I / d_sq + (2.0 * I0 / (R1*R1)) * d_sq - 3.0 * I0/R1 * d : 0.0;
#elif defined(ATTENUATION_MODE_PHY_SMO_2)
  float attenuation = d_sq < R1*R1 ? I / d_sq + (2.0 * I0 / (R1*R1*R1)) * (d_sq*d) - 3.0 * I0/(R1*R1) * d_sq : 0.0;
#else
#error invalid attenuation mode!
#endif

  return attenuation;
}

vec3 cook_torrance(PointLight point_light, vec3 P, vec3 N, vec3 V, vec3 kd, float roughness, float metalness) {
  // roughness *= 0.6;
  // metalness *= 2.0;

  vec3 frag_to_light = point_light.position - P;
  vec3 L = normalize(frag_to_light);

  vec3 F0 = vec3(0.04);
  F0 = mix(F0, kd, metalness);

  // calculate per-light radiance
  vec3 H = normalize(V + L);
  vec3 radiance = point_light.tint * point_light_attenuate(point_light, P);

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
  vec3 frag_pos_in_lgt = fs_pos_in_lgt;
  vec3 frag_geo_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec3 frag_geo_bin_in_lgt = normalize(fs_bin_in_lgt);
  vec3 frag_geo_tan_in_lgt = normalize(fs_tan_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;
  vec3 frag_nor_in_tan = sample_nor_in_tan(frag_pos_in_tex);

  vec4 ka = texture(ambient_sampler, frag_pos_in_tex);
  vec4 ke = texture(emissive_sampler, frag_pos_in_tex);
  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);
  vec4 ks = texture(specular_sampler, frag_pos_in_tex);

#if BASIC_PASS == BASIC_PASS_MASKED
  if (kd.a < 0.5) {
    discard;
  }
#endif
#if BASIC_PASS == BASIC_PASS_TRANSPARENT
  if (kd.a < 0.001) {
    discard;
  }
#endif

#if !PROFILING_TIME_SENSITIVE
  atomicCounterIncrement(shading_ops);
#endif

  // FIXME: Make sure transparent materials are
  // actually somewhat transparent. Asset problem.
  kd.a *= 0.95;

  mat3 tbn = mat3(frag_geo_tan_in_lgt, frag_geo_bin_in_lgt, frag_geo_nor_in_lgt);
  vec3 frag_nor_in_lgt = normalize(tbn * frag_nor_in_tan);
  vec3 frag_to_cam_nor = normalize(cam_pos_in_lgt - frag_pos_in_lgt);

  vec3 color_accumulator = vec3(ke.xyz);
#if defined(RENDER_TECHNIQUE_NAIVE)
  for (uint i = 0; i < light_buffer.light_count.x; i++) {
    PointLight light = light_buffer.point_lights[i];
    float visibility_factor = 1.0;
    if (i == 0) {
      vec3 d = frag_pos_in_lgt - light.position;
      float b = (length(d) - light.r0)/(light.r1 - light.r0);
      visibility_factor = b < (texture(shadow_sampler, d).r + 0.05) ? 1.0 : 0.0;
    }
    color_accumulator += visibility_factor * cook_torrance(light, frag_pos_in_lgt, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);

#if !PROFILING_TIME_SENSITIVE
    atomicCounterIncrement(lighting_ops);
#endif
  }
#elif defined(RENDER_TECHNIQUE_CLUSTERED)
  vec3 pos_in_cls = cluster_cam_to_clp(fs_pos_in_clu_cam);
  uvec3 idx_in_cls = uvec3(pos_in_cls);
  // frag_color = vec4(pos_in_cls / vec3(cluster_space.dimensions.xyz), 1.0);

  // CLUSTER INDICES X, Y, Z
  // frag_color = vec4(vec3(idx_in_cls)/vec3(cluster_space.dimensions), 1.0);

  // CLUSTER INDICES X, Y, Z mod 3
  // vec3 cluster_index_colors = vec3((idx_in_cls % 3) + 1)/4.0;
  // frag_color = vec4(cluster_index_colors.xyz, 1.0);

  // CLUSTER MORTON INDEX
  // uint cluster_morton_index = to_morton_3(idx_in_cls);
  // frag_color = vec4(                              //
  //     float((cluster_morton_index >> 16) & 0xff) / 255.0, //
  //     float((cluster_morton_index >> 8) & 0xff) / 255.0,  //
  //     float((cluster_morton_index >> 0) & 0xff) / 255.0, 1.0);

  uint cluster_index = index_3_to_1(idx_in_cls, cluster_space.dimensions);
  uint maybe_active_cluster_index =
      cluster_maybe_active_cluster_indices[cluster_index];

  if (maybe_active_cluster_index == 0) {
    // We generally shouldn't see clusters that don't have any fragments.
    frag_color = vec4(1.0, 0.0, 1.0, 1.0);
    return;
  }

  uint active_cluster_index = maybe_active_cluster_index - 1;
  uint cluster_light_count = active_cluster_light_counts[active_cluster_index];
  uint cluster_light_offset = active_cluster_light_offsets[active_cluster_index];

  // ACTIVE CLUSTERINDEX
  // frag_color = vec4(vec3(float(active_cluster_index) / 100.0), 1.0);

  // CLUSTER LENGHTS
  // frag_color = vec4(vec3(float(cluster_light_count) / 1000.0), 1.0);
  // frag_color = vec4(heatmap(float(cluster_light_count), 0.0, 1000.0), 1.0);

  // COLORED CLUSTER LENGTHS
  // if (cluster_light_count == 0) {
  //   discard;
  // }
  // frag_color = vec4(vec3(float(cluster_light_count)/2.0) *
  // cluster_index_colors, 1.0);

  // HASH LIGHT INDICES
  // uint hash = 0;
  // uint p_pow = 1;
  // for (uint i = 0; i < cluster_light_count; i++) {
  //   uint light_index = light_indices[cluster_light_offset + i];
  //   hash = (hash + light_index * p_pow) % 0xffff;
  //   p_pow = (p_pow * 31) % 0xffff;
  // }
  // hash = cluster_light_offset;
  // frag_color = vec4(float(hash & 0xff)/255.0, float((hash >> 8) & 0xff)/255.0, float((hash >> 16) & 0xff)/255.0, 1.0);

  // ACTUAL SHADING
  for (uint i = 0; i < cluster_light_count; i++) {
    uint light_index = light_indices[cluster_light_offset + i];
    PointLight light = light_buffer.point_lights[light_index];
    color_accumulator += cook_torrance(light, frag_pos_in_lgt, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);

#if !PROFILING_TIME_SENSITIVE
    atomicCounterIncrement(lighting_ops);
#endif
  }

  // color_accumulator = texture(shadow_sampler, frag_pos_in_lgt - light_buffer.point_lights[0].position).rrr;
#else
#error Unimplemented render technique!
#endif

#if BASIC_PASS == BASIC_PASS_TRANSPARENT
  frag_color = vec4(color_accumulator, kd.a);
#else
  frag_color = vec4(color_accumulator, 1.0);
#endif
}
