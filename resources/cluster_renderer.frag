in vec2 fs_pos_in_tex;

layout(location = 0) out vec4 frag_color;

uniform uint current_light_count;
uniform uint debug_pass;

void main() {
  if (
      fs_pos_in_tex.x > 0.01 && fs_pos_in_tex.x < 0.99 &&
      fs_pos_in_tex.y > 0.01 && fs_pos_in_tex.y < 0.99
      ) {
    if (debug_pass == 1) {
      frag_color = vec4(1.0, 0.6, 0.2, float(current_light_count)/float(32.0));
    } else {
      discard;
    }
  } else {
    if (debug_pass == 0) {
      frag_color = vec4(vec3(0.3), 1.0);
    } else {
      discard;
    }
  }
}
