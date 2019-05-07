out vec2 fs_out;

void main() {
  // Variance shadow map.
  float z = 1 - gl_FragCoord.z;
  fs_out = vec2(z, z * z);
}
