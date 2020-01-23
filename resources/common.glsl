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

uint u8x4_to_u32(uvec4 n) {
  return (n.x & 0xFF) | ((n.y & 0xFF) << 8) | ((n.z & 0xFF) << 16) | ((n.w & 0xFF) << 24);
}

uint u8x4_to_u32_unsafe(uvec4 n) {
  return n.x | (n.y << 8) | (n.z << 16) | (n.w << 24);
}

uvec4 u32_to_u8x4(uint n) {
  return uvec4(
    n & 0xFF,
    (n >> 8) & 0xFF,
    (n >> 16) & 0xFF,
    n >> 24
  );
}

uint u8x3_to_u32(uvec3 n) {
  return (n.x & 0xFF) | ((n.y & 0xFF) << 8) | ((n.z & 0xFF) << 16);
}

uint u8x3_to_u32_unsafe(uvec3 n) {
  return n.x | (n.y << 8) | (n.z << 16);
}

uvec3 u32_to_u8x3(uint n) {
  return uvec3(
    n & 0xFF,
    (n >> 8) & 0xFF,
    (n >> 16) & 0xFF
  );
}
