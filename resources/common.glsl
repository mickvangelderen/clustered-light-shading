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

uint uint_div_ceil(uint n, uint d) {
  uint r = n / d;
  return ((n % d) == 0) ? r : r + 1;
}
