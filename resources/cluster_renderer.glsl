layout(location = 0) uniform mat4 cclp_to_ccam;
layout(location = 1) uniform mat4 ccam_to_clp;
layout(location = 2) uniform uvec3 cluster_dims;
layout(location = 3) uniform uint pass;

#include "cls/cls.glsl"
