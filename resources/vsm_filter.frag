uniform vec2 delta;
uniform sampler2D sampler;

in vec2 fs_pos_in_tex;

out vec2 fs_out;

void main() {
  const int WEIGHT_COUNT = 7;

  vec2 offsets[WEIGHT_COUNT] = vec2[]( //
      -3.0 * delta,                    //
      -2.0 * delta,                    //
      -1.0 * delta,                    //
      vec2(0.0),                       //
      1.0 * delta,                     //
      2.0 * delta,                     //
      3.0 * delta                      //
  );

  float weights[WEIGHT_COUNT] = float[]( //
      0.015625,                          //
      0.093750,                          //
      0.234375,                          //
      0.312500,                          //
      0.234375,                          //
      0.093750,                          //
      0.015625                           //
  );

  vec2 accumulator = vec2(0.5);
  for (int i = 0; i < WEIGHT_COUNT; i++) {
    accumulator = weights[i] * texture(sampler, fs_pos_in_tex + offsets[i]).rg;
  }
  fs_out = accumulator;
}
