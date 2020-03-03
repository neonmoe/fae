// Version preprocessor automatically added by fae, either 100 or 110.

varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform sampler2D tex;
uniform int gamma_correct;

void main(void) {
    vec4 out_color;
    if (frag_texcoord.x == -1.0 && frag_texcoord.y == -1.0) {
        out_color = frag_color;
    } else {
        out_color = frag_color * texture2D(tex, frag_texcoord);
    }
    if (out_color.a < 0.00390625) {
        discard;
    }
    if (gamma_correct != 0) {
	// Gamma correction: applying it to the alpha channel (as well
	// as the others) looks closer to having GL_FRAMEBUFFER_SRGB
	// enabled, even though apparently the alpha channel should be
	// linear? Don't know.
	out_color.rgba = pow(out_color.rgba, vec4(1.0 / 2.2));
    }
    gl_FragColor = out_color;
}
