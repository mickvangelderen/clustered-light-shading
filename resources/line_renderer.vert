layout(location = OBJ_TO_CLP_LOC) uniform mat4 obj_to_clp;

layout(location = VS_POS_IN_OBJ_LOC) in vec3 vs_pos_in_obj;

void main() {
  gl_Position = obj_to_clp * vec4(vs_pos_in_obj, 1.0);
}
