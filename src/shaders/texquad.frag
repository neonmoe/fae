#version 330

out vec4 out_color;
in vec2 frag_texcoord;
in vec4 frag_color;
uniform sampler2D tex;

void main(void) {
  if (frag_texcoord.x == -1.0 && frag_texcoord.y == -1.0) {
    out_color = frag_color;
  } else {
    out_color = frag_color * texture(tex, frag_texcoord.xy);
  }
  if (out_color.a < 0.00390625) {
    discard;
  }
}
