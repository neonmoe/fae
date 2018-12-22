#version 330

// Per-vertex attributes:
in vec2 shared_position;
in vec2 shared_texcoord;
// Per-instance attributes:
in vec4 position;
in vec4 texcoord;
in vec4 color;
in vec3 rotation;
in float depth;

out vec2 frag_texcoord;
out vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
  float rot_radians = rotation.x;
  vec4 vertex_pos = vec4((shared_position - rotation.yz) * position.zw, depth, 1.0);
  vertex_pos.xy = vec2(cos(rot_radians) * vertex_pos.x - sin(rot_radians) * vertex_pos.y,
                       sin(rot_radians) * vertex_pos.x + cos(rot_radians) * vertex_pos.y);
  vertex_pos.xy += position.xy + rotation.yz * position.zw;
  gl_Position = vertex_pos * projection_matrix;
  if (texcoord == vec4(-1.0, -1.0, -2.0, -2.0)) {
    frag_texcoord = vec2(-1.0, -1.0);
  } else {
    frag_texcoord = texcoord.xy + shared_texcoord.xy * texcoord.zw;
  }
  frag_color = color;
}
