layout(location = 0) out vec4 frag_color;

void main() {
  vec3 p = gl_FragCoord.xyz/gl_FragCoord.w;
  p = normalize(p);

  frag_color = vec4(p * 0.5 + 0.5, 1.0);
}

