vec3 heatmap(float value, float minVal, float maxVal) {
  vec3 color = vec3(0.0, 0.0, 0.0);
  float range = maxVal - minVal;
  float adjustedVal = clamp(value - minVal, 0.0, range);
  float step = range / 6.0;
  if (value < step) {
    color.z = value / step;
  } else if (value < 2.0 * step) {
    color.y = (value - step) / step;
    color.z = 1.0;
  } else if (value < 3.0 * step) {
    color.y = 1.0;
    color.z = 1.0 - (value - 2.0 * step) / step;
  } else if (value < 4.0 * step) {
    color.x = (value - 3.0 * step) / step;
    color.y = 1.0;
  } else if (value < 5.0 * step) {
    color.x = 1.0;
    color.y = 1.0 - (value - 4.0 * step) / step;
  } else {
    color.x = 1.0;
    color.y = (value - 5.0 * step) / step;
    color.z = (value - 5.0 * step) / step;
  }
  return color;
}
