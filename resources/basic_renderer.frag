#version 400

uniform vec3 ambient;
uniform vec3 diffuse;
uniform vec3 specular;
uniform float shininess;
uniform sampler2D diffuse_sampler;
// uniform sampler2D normal_sampler;
uniform float highlight;

in vec3 fs_pos_in_cam;
in vec2 fs_pos_in_tex;
in vec3 fs_nor_in_cam;
in vec3 fs_tan_in_cam;

layout(location = 0) out vec4 frag_color;
layout(location = 1) out uvec2 frag_nor_in_cam;

void main() {
  vec3 light_pos_in_cam = vec3(0.0);
  vec3 light_dir_in_cam_normalized =
      normalize(light_pos_in_cam - fs_pos_in_cam);
  vec3 cam_dir_in_cam_normalized = normalize(-fs_pos_in_cam);

  vec3 nor_in_cam = normalize(fs_nor_in_cam);

  float ambient_weight = 0.1;
  float diffuse_weight = max(dot(nor_in_cam, light_dir_in_cam_normalized), 0.0);
  float specular_angle =
      max(dot(cam_dir_in_cam_normalized,
              reflect(-light_dir_in_cam_normalized, nor_in_cam)),
          0.0);
  float specular_weight = pow(specular_angle, shininess);

  vec4 diffuse_sample = texture(diffuse_sampler, fs_pos_in_tex);
  vec3 diffuse_color;
  if (diffuse_sample != vec4(0.0, 0.0, 0.0, 1.0)) {
    diffuse_color = diffuse_sample.rgb;
  } else {
    diffuse_color = diffuse;
  }

  // frag_color = vec4(ambient * ambient_weight             //
  //                       + diffuse_color * diffuse_weight //
  //                       + specular * specular_weight,    //
  //                   1.0);
  // frag_color = vec4(diffuse, 1.0);
  float cx = nor_in_cam.x;
  float cy = nor_in_cam.y;
  float cz_sign = sign(nor_in_cam.z);
  float cz = cz_sign * sqrt(1 - cx * cx - cy * cy);

  frag_color = vec4((vec3(cx, cy, cz_sign) + vec3(1.0)) / 2.0, 1.0);

  // 7 bits for x.
  uint x = min(uint(nor_in_cam.x * 64.0 + 64), 127);
  // 1 bit for z-direction.
  uint z = uint(sign(nor_in_cam.z) * 0.5 + 0.5) << 7;
  // 8 bits for y.
  uint y = min(uint(nor_in_cam.y * 128.0 + 128.0), 255);
  frag_nor_in_cam = uvec2(x | z, y);
}
