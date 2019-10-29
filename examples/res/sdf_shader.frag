#version 330

out vec4 out_color;
in vec2 frag_texcoord;
in vec4 frag_color;
uniform sampler2D tex;

void main(void) {
    float alpha = texture(tex, frag_texcoord.xy).a;
    float threshold = 0.675;
    if (alpha > threshold) {
	out_color = frag_color * 0.001 + vec4(0.0, 0.0, 0.0, 1.0);
    } else {
	out_color = frag_color * 0.001 + vec4(0.0, 0.0, 0.0, pow(alpha / threshold, 32.0));
    }
}
