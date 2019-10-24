#version 110

varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform sampler2D tex;

void main(void) {
  vec4 out_color = frag_color;
  if (frag_texcoord.x != -1.0 || frag_texcoord.y != -1.0) {
    out_color.a = texture2D(tex, frag_texcoord).r;
  }
  if (out_color.a < 0.00390625) {
    discard;
  }
  gl_FragColor = out_color;
}
