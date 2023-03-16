#version 410

in vec2 vert_xy;
in vec3 vert_rgb;

uniform mat3 view_matrix;

out vec3 frag_rgb;

void main() {
    vec3 pos = view_matrix * vec3(vert_xy, 1.0);
    gl_Position = vec4(pos.xy, 0.0, 1.0);

    frag_rgb = vert_rgb;
}
