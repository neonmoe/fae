#version 330

layout(location = 0) in vec4 position;
layout(location = 1) in vec2 texcoord;
layout(location = 2) in vec4 color;
out vec2 frag_texcoord;
out vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
  gl_Position = position * projection_matrix;
  frag_texcoord = texcoord;
  frag_color = color;
}
