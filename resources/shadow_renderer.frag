out vec2 fs_out;

void main() {
  // Variance shadow map.
  fs_out = vec2(gl_FragCoord.z, gl_FragCoord.z * gl_FragCoord.z);
}
