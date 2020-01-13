// Version preprocessor automatically added by fae, either 300 es or 330.

out vec4 out_color;
in vec2 frag_texcoord;
in vec4 frag_color;
uniform sampler2D tex;

void main(void) {
    out_color = frag_color;
    out_color.a *= texture(tex, frag_texcoord).r;
}
