#version 110

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color;
attribute vec3 rotation;
varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
  float rot_radians = rotation.x;
  vec4 vertex_pos = vec4(position.xy - rotation.yz, position.z, 1.0);
  vertex_pos.xy = vec2(cos(rot_radians) * vertex_pos.x - sin(rot_radians) * vertex_pos.y,
                       sin(rot_radians) * vertex_pos.x + cos(rot_radians) * vertex_pos.y);
  vertex_pos.xy += rotation.yz;
  gl_Position = vertex_pos * projection_matrix;
  frag_texcoord = texcoord;
  frag_color = color;
}
