#version 330

out vec4 out_color;
in vec2 frag_texcoord;
in vec4 frag_color;
uniform sampler2D tex;

void main(void) {
  vec4 color = frag_color;
  color.a = texture(tex, frag_texcoord).r;
  out_color = color;
}
