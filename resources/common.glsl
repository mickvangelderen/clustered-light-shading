vec3 from_homogeneous(vec4 p) { return p.xyz / p.w; }

vec4 to_homogeneous(vec3 p) { return vec4(p, 1.0); }

uint index_3_to_1(uvec3 indices, uvec3 dimensions) {
  return (((indices.z * dimensions.y) + indices.y) * dimensions.x) + indices.x;
}

uvec3 index_1_to_3(uint index_1, uvec3 dimensions) {
  uint x = index_1 % dimensions.x;
  index_1 /= dimensions.x;
  uint y = index_1 % dimensions.y;
  index_1 /= dimensions.y;
  uint z = index_1;
  return uvec3(x, y, z);
}

uint ceiled_div_u32(uint n, uint d) {
  return (n + (d - 1))/d;
}

float lerp_f32_f32(float x, float x0, float x1, float y0, float y1) {
  return (y1*(x - x0) + y0*(x1 - x))/(x1 - x0);
}

float lerp_u32_f32(uint x, uint x0, uint x1, float y0, float y1) {
  return (y1*float(x - x0) + y0*float(x1 - x))/float(x1 - x0);
}
