#include "native/ATTENUATION_MODE"

float point_light_attenuate(float i, float i0, float r0, float r1, float d_unclipped) {
  float d_sq_unclipped = d_unclipped*d_unclipped;

  float d = clamp(d_unclipped, r0, r1);
  float d_sq = d*d;

#if defined(ATTENUATION_MODE_STEP)
  // Use physical intensity halfway between 0 and r1
  float attenuation = 4*i/(r1*r1)*step(d_unclipped, r1);
#elif defined(ATTENUATION_MODE_LINEAR)
  // Linear doesn't go infinite so we can use the unclipped distance.
  float attenuation = max(0.0, i*(r1 - d_unclipped)/r1);
#elif defined(ATTENUATION_MODE_PHYSICAL)
  float attenuation = i / d_sq_unclipped * step(d_unclipped, r1);
#elif defined(ATTENUATION_MODE_REDUCED)
  float attenuation = i / d_sq - i0;
#elif defined(ATTENUATION_MODE_PHY_RED_1)
  float attenuation = i / d_sq - i0/r1*d;
#elif defined(ATTENUATION_MODE_PHY_RED_2)
  float attenuation = i / d_sq - (i0 / (r1*r1)) * d_sq;
#elif defined(ATTENUATION_MODE_SMOOTH)
  float attenuation = i / d_sq + (2.0 * i0 / r1) * d - 3.0 * i0;
#elif defined(ATTENUATION_MODE_PHY_SMO_1)
  float attenuation = i / d_sq + (2.0 * i0 / (r1*r1)) * d_sq - 3.0 * i0/r1 * d;
#elif defined(ATTENUATION_MODE_PHY_SMO_2)
  float attenuation = i / d_sq + (2.0 * i0 / (r1*r1*r1)) * (d_sq*d) - 3.0 * i0/(r1*r1) * d_sq;
#else
#error invalid attenuation mode!
#endif

  return attenuation;
}
