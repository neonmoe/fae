#version 330

out vec4 out_color;
in vec2 frag_texcoord;
in vec4 frag_color;
uniform sampler2D tex;

void main(void) {
  out_color = frag_color * texture(tex, frag_texcoord);
  if (out_color.a < 0.01) {
    discard;
  }
}
