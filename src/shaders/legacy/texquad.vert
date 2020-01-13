// Version preprocessor automatically added by fae, either 100 or 110.

attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color;
attribute vec3 rotation;
varying vec2 frag_texcoord;
varying vec4 frag_color;
uniform mat4 projection_matrix;

void main(void) {
    float rot_radians = rotation.x;
    vec4 vertex_pos = vec4(position.xy - rotation.yz, position.z, 1.0);
    float cos_r = cos(rot_radians);
    float sin_r = sin(rot_radians);
    vertex_pos.xy = vec2(cos_r * vertex_pos.x - sin_r * vertex_pos.y,
                         sin_r * vertex_pos.x + cos_r * vertex_pos.y);
    vertex_pos.xy += rotation.yz;
    gl_Position = vertex_pos * projection_matrix;
    frag_texcoord = texcoord;
    frag_color = color;
}
