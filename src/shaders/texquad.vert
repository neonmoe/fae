#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec2 texcoord;
out vec2 frag_texcoord;
uniform mat4 projection_matrix;

void main(void) {
  gl_Position = position * projection_matrix;
  frag_texcoord = texcoord;
}
