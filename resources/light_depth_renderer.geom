#if !defined(BASIC_PASS)
#error BASIC_PASS is undefined.
#endif

layout(triangles) in;
layout(triangle_strip, max_vertices = 18) out;

layout(location = WLD_TO_CLP_ARRAY_LOC) uniform mat4 wld_to_clp_array[6];

#if BASIC_PASS == BASIC_PASS_MASKED
in vec2 ge_pos_in_tex[3];
out vec2 fs_pos_in_tex;
#endif

out vec4 fs_pos_in_wld;

void main() {
  for(int face = 0; face < 6; face++) {
    gl_Layer = face;
    for(int vertex = 0; vertex < 3; vertex++) {
#if BASIC_PASS == BASIC_PASS_MASKED
      fs_pos_in_tex = ge_pos_in_tex[vertex];
#endif
      fs_pos_in_wld = gl_in[vertex].gl_Position;
      vec4 pos_in_wld = gl_in[vertex].gl_Position;
      gl_Position = wld_to_clp_array[face] * pos_in_wld;
      EmitVertex();
    }
    EndPrimitive();
  }
}
