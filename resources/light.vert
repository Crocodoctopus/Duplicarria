#version 310 es

layout(location = 0) in vec2 vert_xy;
layout(location = 1) in vec2 vert_uv;

layout(location = 0) uniform mat3 view_matrix;

out vec2 frag_uv;

void main() {
	gl_Position = vec4((view_matrix * vec3(vert_xy, 1)).xy, 0, 1);
	frag_uv = vert_uv;
}