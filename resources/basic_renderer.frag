#include "cotangent_frame.glsl"

layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = NORMAL_SAMPLER_LOC) uniform sampler2D normal_sampler;
layout(location = AMBIENT_SAMPLER_LOC) uniform sampler2D ambient_sampler;
layout(location = DIFFUSE_SAMPLER_LOC) uniform sampler2D diffuse_sampler;
layout(location = SPECULAR_SAMPLER_LOC) uniform sampler2D specular_sampler;
layout(location = SHININESS_LOC) uniform float shininess;

in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec2 fs_pos_in_tex;

layout(location = 0) out vec4 frag_color;

void main() {
  vec3 frag_pos_in_lgt = fs_pos_in_lgt.xyz/fs_pos_in_lgt.w;
  vec3 frag_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;

  vec2 n = texture(normal_sampler, frag_pos_in_tex).xy * 2.0 - 1.0;
  vec4 ka = texture(ambient_sampler, frag_pos_in_tex);
  vec4 kd = texture(diffuse_sampler, frag_pos_in_tex);
  vec4 ks = texture(specular_sampler, frag_pos_in_tex);

  mat3 tbn = cotangent_frame(frag_nor_in_lgt, frag_pos_in_lgt, frag_pos_in_tex);
  frag_nor_in_lgt = tbn * vec3(n.xy, sqrt(1.0 - dot(n, n)));

  vec3 frag_to_light_nor = normalize(vec3(0.1, 1.0, 0.2));
  vec3 frag_reflect_nor = reflect(-frag_to_light_nor, frag_nor_in_lgt);
  vec3 frag_to_cam_nor = normalize(cam_pos_in_lgt.xyz - frag_pos_in_lgt);

  float d = max(0.0, dot(frag_to_light_nor, frag_nor_in_lgt));
  float s = pow(max(0.0, dot(frag_reflect_nor, frag_to_cam_nor)), shininess);

  if (kd.a < 0.01) {
    discard;
  }

  frag_color = vec4(d * kd.rgb + s * vec3(ks.y), 1.0);
  // frag_color = vec4(ka.rgb + d * kd.rgb + s * ks.rgb, 1.0);
  // frag_color = vec4(vec3(ks.y), 1.0);

  // Normals
  // frag_color = vec4(frag_nor_in_lgt * 0.5 + 0.5, 1.0);

  // Texture coordinates
  // frag_color = vec4(frag_pos_in_tex * 0.5 + 0.5, 0.0, 1.0);
}

