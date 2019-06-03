uniform uint light_count;

in vec3 fs_pos_in_cam;
in vec2 fs_pos_in_tex;
in vec3 fs_nor_in_cam;
in vec3 fs_tan_in_cam;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out vec3 frag_nor_in_cam;

void main() {
  // Common intermediates.
  vec3 light_dir_in_cam = normalize(light_dir_in_cam);
  vec3 cam_dir_in_cam = normalize(-fs_pos_in_cam);
  vec3 nor_in_cam = normalize(fs_nor_in_cam);

  // Diffuse.
  float diffuse_weight = max(0.2, dot(nor_in_cam, light_dir_in_cam));
  float fraction = float(light_count)/float(cluster_dims.w);

  // Specular.
  float specular_angle =
      max(dot(cam_dir_in_cam, reflect(-light_dir_in_cam, nor_in_cam)),
          0.0);
  float specular_weight = pow(specular_angle, 64.0);

  if (
      fs_pos_in_tex.x > 0.05 && fs_pos_in_tex.x < 0.95 &&
      fs_pos_in_tex.y > 0.05 && fs_pos_in_tex.y < 0.95
      ) {
    frag_color = vec4(vec3(fraction), 1.0);
  } else {
    // frag_color = vec4(diffuse_weight * vec3(fraction) + specular_weight * vec3(1.0), 1.0);
    frag_color = vec4((diffuse_weight * 1.0 + specular_weight) * vec3(1.0, 0.5, 0.0), 1.0);
  }
}
