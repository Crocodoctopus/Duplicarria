#version 410

in vec2 vert_xy;
in vec2 vert_uv;

uniform mat3 view_matrix;

out vec2 frag_uv;

void main() {
	gl_Position = vec4((view_matrix * vec3(vert_xy, 1)).xy, 0, 1);
	frag_uv = vert_uv;
}