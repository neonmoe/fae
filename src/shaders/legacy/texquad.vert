#version 110

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color;
varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
  gl_Position = vec4(position, 1.0) * projection_matrix;
  frag_texcoord = texcoord;
  frag_color = color;
}
