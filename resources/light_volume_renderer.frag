in vec3 fs_tint;

layout(location = OPACITY_LOC) uniform float opacity;
layout(location = 0) out vec4 frag_color;

void main() {
  frag_color = vec4(fs_tint, opacity);
}
