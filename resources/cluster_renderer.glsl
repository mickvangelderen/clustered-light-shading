layout(location = 0) uniform mat4 cls_to_clp;
layout(location = 1) uniform uvec3 cluster_dims;
layout(location = 2) uniform uint pass;

#include "cls/cls.glsl"
