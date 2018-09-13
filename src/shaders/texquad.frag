#version 330

out vec4 out_color;
in vec2 frag_texcoord;
uniform sampler2D tex;

void main(void) {
  out_color = texture(tex, frag_texcoord);
}
