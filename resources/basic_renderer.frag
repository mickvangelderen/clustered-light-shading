#include "cotangent_frame.glsl"
#include "pbr.glsl"

layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = NORMAL_SAMPLER_LOC) uniform sampler2D normal_sampler;
layout(location = EMISSIVE_SAMPLER_LOC) uniform sampler2D emissive_sampler;
layout(location = AMBIENT_SAMPLER_LOC) uniform sampler2D ambient_sampler;
layout(location = DIFFUSE_SAMPLER_LOC) uniform sampler2D diffuse_sampler;
layout(location = SPECULAR_SAMPLER_LOC) uniform sampler2D specular_sampler;
layout(location = SHININESS_LOC) uniform float shininess;

in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec3 fs_bin_in_lgt;
in vec3 fs_tan_in_lgt;
in vec2 fs_pos_in_tex;
flat in uint fs_instance_index;

layout(location = 0) out vec4 frag_color;

vec3 sample_nor_in_tan(vec2 pos_in_tex) {
  vec2 xy = texture(normal_sampler, pos_in_tex).xy * 2.0 - vec2(1.0);
  float z = sqrt(max(0.0, 1.0 - dot(xy, xy)));
  return vec3(xy, z);
}

void main() {
  vec3 frag_pos_in_lgt = fs_pos_in_lgt.xyz/fs_pos_in_lgt.w;
  vec3 frag_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec3 frag_bin_in_lgt = normalize(fs_bin_in_lgt);
  vec3 frag_tan_in_lgt = normalize(fs_tan_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;
  vec3 frag_nor_in_tan = sample_nor_in_tan(frag_pos_in_tex);

  vec4 ka = texture(ambient_sampler, frag_pos_in_tex);
  vec4 ke = texture(emissive_sampler, frag_pos_in_tex);
  vec4 kd = pow(texture(diffuse_sampler, frag_pos_in_tex), vec4(2.2));
  vec4 ks = texture(specular_sampler, frag_pos_in_tex);

  if (kd.a < 0.5) {
    discard;
  }

  // n = vec2(0.0);

  mat3 tbn = mat3(frag_tan_in_obj, frag_bin_in_obj, frag_nor_in_obj);
  vec3 frag_nor_in_lgt = normalize(tbn * frag_nor_in_tan);

  vec3 light_pos_in_lgt = (cam_to_wld * vec4(0.0, 0.5, -1.5, 1.0)).xyz;
  vec3 frag_to_light_nor = normalize(light_pos_in_lgt - frag_pos_in_lgt);
  vec3 frag_reflect_nor = reflect(-frag_to_light_nor, frag_nor_in_lgt);
  vec3 frag_to_cam_nor = normalize(cam_pos_in_lgt.xyz - frag_pos_in_lgt);

  vec3 F0 = vec3(0.04);
  F0 = mix(F0, kd.xyz, ks.z);

  // calculate per-light radiance
  vec3 N = frag_nor_in_lgt;
  vec3 V = frag_to_cam_nor;
  vec3 L = frag_to_light_nor;
  vec3 H = normalize(V + L);
  // float distance = length(lightPositions[i] - WorldPos);
  // float attenuation = 1.0 / (distance * distance);
  // vec3 radiance = lightColors[i] * attenuation;
  vec3 radiance = vec3(3.0);

  // Cook-Torrance BRDF
  float roughness = ks.y;
  float NDF = DistributionGGX(N, H, roughness);
  float G   = GeometrySmith(N, V, L, roughness);
  vec3 F    = fresnelSchlick(clamp(dot(H, V), 0.0, 1.0), F0);

  vec3 nominator    = NDF * G * F;
  float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0);
  vec3 specular = nominator / max(denominator, 0.001); // prevent divide by zero for NdotV=0.0 or NdotL=0.0

  // kS is equal to Fresnel
  vec3 kS = F;
  // for energy conservation, the diffuse and specular light can't
  // be above 1.0 (unless the surface emits light); to preserve this
  // relationship the diffuse component (kD) should equal 1.0 - kS.
  vec3 kD = vec3(1.0) - kS;
  // multiply kD by the inverse metalness such that only non-metals 
  // have diffuse lighting, or a linear blend if partly metal (pure metals
  // have no diffuse light).
  kD *= 1.0 - ks.z;

  // scale light by NdotL
  float NdotL = max(dot(N, L), 0.0);

  // add to outgoing radiance Lo
  vec3 Lo = vec3(ke.xyz);
  Lo += (kD * kd.xyz / PI + specular) * radiance * NdotL;  // note that we already multiplied the BRDF by the Fresnel (kS) so we won't multiply by kS again

  // Hacky tone-map
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

