#version 330

out vec4 out_color;
in vec2 frag_texcoord;
uniform sampler2D tex;

void main(void) {
  out_color = vec4(0.0, 0.0, 0.0, texture(tex, frag_texcoord).r);
}
