in vec4 fs_pos_in_lgt;
in vec3 fs_nor_in_lgt;
in vec2 fs_pos_in_tex;

layout(location = 0) out vec4 frag_color;

void main() {
  vec3 frag_pos_in_lgt = fs_pos_in_lgt.xyz/fs_pos_in_lgt.w;
  vec3 frag_nor_in_lgt = normalize(fs_nor_in_lgt);
  vec2 frag_pos_in_tex = fs_pos_in_tex;

  vec3 frag_to_light_nor = normalize(vec3(0.1, 1.0, 0.2));
  vec3 frag_reflect_nor = reflect(-frag_to_light_nor, frag_nor_in_lgt);
  vec3 frag_to_cam_nor = normalize(cam_pos_in_lgt.xyz - frag_pos_in_lgt);

  float d = max(0.0, dot(frag_to_light_nor, frag_nor_in_lgt));
  float s = pow(max(0.0, dot(frag_reflect_nor, frag_to_cam_nor)), 16.0);

  vec3 kd = frag_nor_in_lgt * 0.5 + 0.5;
  frag_color = vec4(vec3(0.02) + d * kd + s * vec3(0.2), 1.0);

  // Normals
  // frag_color = vec4(frag_nor_in_lgt * 0.5 + 0.5, 1.0);

  // Texture coordinates
  // frag_color = vec4(frag_pos_in_tex * 0.5 + 0.5, 0.0, 1.0);
}

