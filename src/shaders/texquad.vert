#version 330

in vec3 position;
in vec2 texcoord;
in vec4 color;
in vec4 rotation;
out vec2 frag_texcoord;
out vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
  gl_Position = vec4(position, 1.0) * projection_matrix + rotation;
  frag_texcoord = texcoord;
  frag_color = color;
}
