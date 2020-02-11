#include "native/RENDER_TECHNIQUE"
#include "native/PROFILING"
#include "native/DEPTH_PREPASS"

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

#include "common.glsl"
#include "heatmap.glsl"
#include "light_buffer.glsl"
#include "point_light_attenuate.glsl"
#include "pbr.glsl"

#if defined(RENDER_TECHNIQUE_CLUSTERED)
#include "cls/cluster_space_buffer.glsl"
#include "cls/cluster_maybe_active_cluster_indices_buffer.glsl"
#include "cls/active_cluster_light_counts_buffer.glsl"
#include "cls/active_cluster_light_offsets_buffer.glsl"
#include "cls/light_indices_buffer.glsl"
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
layout(binding = SHADOW_SAMPLER_BINDING_2) uniform samplerCube shadow_sampler_2;
layout(binding = SHADOW_SAMPLER_BINDING_3) uniform samplerCube shadow_sampler_3;

layout(location = CAM_POS_IN_LGT_LOC) uniform vec3 cam_pos_in_lgt;

#if defined(RENDER_TECHNIQUE_CLUSTERED)
layout(location = VIEWPORT_LOC) uniform vec4 viewport;
layout(location = REN_CLP_TO_CLU_CAM_LOC) uniform mat4 ren_clp_to_clu_cam;
#endif

in vec3 fs_pos_in_lgt;
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

  vec3 color_accumulator = vec3(0.0);
#if defined(RENDER_TECHNIQUE_NAIVE)
  // Indirect
  for (uint i = 1; i < light_buffer.light_count.x; i += 1) {
    PointLight light = light_buffer.point_lights[i];
    vec3 f_to_l = light.position - frag_pos_in_lgt;
    float f_to_l_mag = length(f_to_l);

    color_accumulator +=
      point_light_attenuate(light.i, light.i0, light.r0, light.r1, f_to_l_mag) *
      light.tint *
      cook_torrance(f_to_l/f_to_l_mag, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);

#if !PROFILING_TIME_SENSITIVE
    atomicCounterIncrement(lighting_ops);
#endif
  }
  // Direct
  // {
  //   PointLight light = light_buffer.point_lights[0];
  //   vec3 l_to_f = frag_pos_in_lgt - light.position;
  //   float l_to_f_mag = length(l_to_f);

  //   color_accumulator +=
  //   (light.i/(l_to_f_mag*l_to_f_mag)) *
  //   light.tint *
  //   cook_torrance(l_to_f/(-l_to_f_mag), frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);
  // }
#elif defined(RENDER_TECHNIQUE_CLUSTERED)
  vec3 frag_pos_in_clu_cam = from_homogeneous(ren_clp_to_clu_cam * vec4(
    gl_FragCoord.xy/viewport.zw * 2.0 - 1.0,
    gl_FragCoord.z,
    1.0
  ));
  vec3 pos_in_cls = cluster_cam_to_clp(frag_pos_in_clu_cam);
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
  // color_accumulator = vec3(float(active_cluster_index) / 100.0);

  // CLUSTER LENGHTS
  // color_accumulator = vec3(float(cluster_light_count) / 400.0);
  // color_accumulator = heatmap(float(cluster_light_count), 0.0, 100.0);

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
    vec3 f_to_l = light.position - frag_pos_in_lgt;
    float f_to_l_mag = length(f_to_l);

    if (light_index == 0) {
      continue;
    } else if (light_index < light_buffer.virtual_light_count) {
      // float roughness = mix(0.0, light.r1, light._pad1)*ks.y;
      float roughness = ks.y;
      float metalness = ks.z;
      color_accumulator +=
        min(light.i, point_light_attenuate(light.i, light.i0, light.r0, light.r1, f_to_l_mag)) *
        light.tint *
        cook_torrance(f_to_l/f_to_l_mag, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, roughness, metalness);

    } else {
      color_accumulator +=
        point_light_attenuate(light.i, light.i0, light.r0, light.r1, f_to_l_mag) *
        light.tint *
        cook_torrance(f_to_l/f_to_l_mag, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);
    }

#if !PROFILING_TIME_SENSITIVE
    atomicCounterIncrement(lighting_ops);
#endif
  }

  // SHADOW MAP
  PointLight light = light_buffer.point_lights[0];
  vec3 l_to_f = frag_pos_in_lgt - light.position;
  float d_l_to_f_closest = texture(shadow_sampler, l_to_f).r;
  float d_l_to_f = length(l_to_f);
  vec3 f_to_l_norm = l_to_f/-d_l_to_f;
  vec3 triangle_normal = normalize(cross(dFdx(frag_pos_in_lgt), dFdy(frag_pos_in_lgt)));

  float dot_n_lo = dot(f_to_l_norm, triangle_normal);
  float bias = 0.1*sqrt(1.0 - dot_n_lo*dot_n_lo);

  uint DISPLAY = 1;

  if (dot_n_lo > 0.0 && d_l_to_f < d_l_to_f_closest + bias) {
    if (DISPLAY == 1) {
      color_accumulator +=
        point_light_attenuate(light.i, light.i0, light.r0, light.r1, d_l_to_f) *
        light.tint *
        cook_torrance(l_to_f/-d_l_to_f, frag_nor_in_lgt, frag_to_cam_nor, kd.xyz, ks.y, ks.z);
    } else if (DISPLAY == 2) {
      color_accumulator = step(d_l_to_f, light.r1) * (texture(shadow_sampler_2, l_to_f).rgb * 0.5 + 0.5);
    } else if (DISPLAY == 3) {
      color_accumulator = step(d_l_to_f, light.r1) * texture(shadow_sampler_3, l_to_f).rgb;
    }
  } else {
    if (DISPLAY == 2 || DISPLAY == 3) {
      color_accumulator = vec3(0.0);
    }
  }

  // DIFFUSE
  // color_accumulator = normalize(kd.rgb);
#else
#error Unimplemented render technique!
#endif

#if BASIC_PASS == BASIC_PASS_TRANSPARENT
  frag_color = vec4(color_accumulator, kd.a);
#else
  frag_color = vec4(color_accumulator, 1.0);
#endif
}
