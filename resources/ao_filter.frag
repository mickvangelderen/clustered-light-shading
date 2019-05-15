uniform vec2 delta;
uniform sampler2D color_sampler;
uniform sampler2D depth_sampler;

in vec2 fs_pos_in_tex;

out float fs_out;

void main() {
  float ao_r = 1.0;
  float ao_r_sq = ao_r * ao_r;

  const int WEIGHT_COUNT = 11;

  vec2 offsets[WEIGHT_COUNT] = vec2[]( //
      -5.0 * delta,                    //
      -4.0 * delta,                    //
      -3.0 * delta,                    //
      -2.0 * delta,                    //
      -1.0 * delta,                    //
      vec2(0.0),                       //
      1.0 * delta,                     //
      2.0 * delta,                     //
      3.0 * delta,                     //
      4.0 * delta,                     //
      5.0 * delta                      //
  );

  float weights[WEIGHT_COUNT] = float[]( //
      0.000003,                          //
      0.000229,                          //
      0.005977,                          //
      0.060598,                          //
      0.24173,                           //
      0.382925,                          //
      0.24173,                           //
      0.060598,                          //
      0.005977,                          //
      0.000229,                          //
      0.000003                           //
  );

  float center_depth = texture(depth_sampler, fs_pos_in_tex).r;

  float accumulator = 0.0;

  for (int i = 0; i < WEIGHT_COUNT; i++) {
    vec2 sample_pos_in_tex = fs_pos_in_tex + offsets[i];
    float sample_depth = texture(depth_sampler, sample_pos_in_tex).r;
    float delta_depth = sample_depth - center_depth;
    float sample_color = texture(color_sampler, sample_pos_in_tex).x;
    accumulator +=
        smoothstep(ao_r, 0.0, abs(delta_depth)) * weights[i] * sample_color;
  }

  // fs_out = accumulator;
  fs_out = texture(color_sampler, fs_pos_in_tex).x;
}
