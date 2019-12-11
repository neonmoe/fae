// Version preprocessor automatically added by fae, either 100 or 110.

varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform sampler2D tex;

void main(void) {
  vec4 out_color = frag_color;
  out_color.a *= texture2D(tex, frag_texcoord).r;
  if (out_color.a < 0.00390625) {
    discard;
  }
  gl_FragColor = out_color;
}
