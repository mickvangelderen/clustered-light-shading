layout(location = OBJ_TO_WLD_LOC) uniform mat4 obj_to_wld;
layout(location = AMBIENT_COLOR_LOC) uniform vec3 ambient_color;
layout(location = DIFFUSE_COLOR_LOC) uniform vec3 diffuse_color;
layout(location = SPECULAR_COLOR_LOC) uniform vec3 specular_color;
layout(location = SHININESS_LOC) uniform float shininess;

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
  float s = pow(max(0.0, dot(frag_reflect_nor, frag_to_cam_nor)), shininess);

  vec3 ambi = ambient_color;
  vec3 diff = d * diffuse_color;
  vec3 spec = s * specular_color;

  frag_color = vec4(ambi + diff + spec, 1.0);

  // Normals
  // frag_color = vec4(frag_nor_in_lgt * 0.5 + 0.5, 1.0);

  // Texture coordinates
  // frag_color = vec4(frag_pos_in_tex * 0.5 + 0.5, 0.0, 1.0);
}

