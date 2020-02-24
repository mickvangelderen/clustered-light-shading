layout(triangles) in;
layout(triangle_strip, max_vertices = 18) out;

layout(location = WLD_TO_CLP_ARRAY_LOC) uniform mat4 wld_to_clp_array[6];

in vec3 ge_pos_in_lgt[3];
in vec3 ge_nor_in_lgt[3];
in vec3 ge_bin_in_lgt[3];
in vec3 ge_tan_in_lgt[3];
in vec2 ge_pos_in_tex[3];

out vec3 fs_pos_in_lgt;
out vec3 fs_nor_in_lgt;
out vec3 fs_bin_in_lgt;
out vec3 fs_tan_in_lgt;
out vec2 fs_pos_in_tex;

void main() {
  for(int face = 0; face < 6; face++) {
    gl_Layer = face;
    for(int vertex = 0; vertex < 3; vertex++) {
      fs_pos_in_lgt = ge_pos_in_lgt[vertex];
      fs_nor_in_lgt = ge_nor_in_lgt[vertex];
      fs_bin_in_lgt = ge_bin_in_lgt[vertex];
      fs_tan_in_lgt = ge_tan_in_lgt[vertex];
      fs_pos_in_tex = ge_pos_in_tex[vertex];
      gl_Position = wld_to_clp_array[face] * vec4(fs_pos_in_lgt, 1.0);
      EmitVertex();
    }
    EndPrimitive();
  }
}
